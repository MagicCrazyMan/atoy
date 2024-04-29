use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{
    js_sys::{Function, Promise},
    WebGl2RenderingContext,
};

use crate::window;

use super::{conversion::ToGlEnum, error::Error};

/// Available client wait flags mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FenceSyncFlag {
    SyncGpuCommandsComplete,
}

/// Available client wait flags mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClientWaitFlag {
    SyncFlushCommandsBit,
}

pub enum ClientWaitStatus {
    AlreadySignaled,
    TimeoutExpired,
    ConditionSatisfied,
    WaitFailed,
}

impl ClientWaitStatus {
    fn from_u32(value: u32) -> Self {
        match value {
            WebGl2RenderingContext::ALREADY_SIGNALED => ClientWaitStatus::AlreadySignaled,
            WebGl2RenderingContext::TIMEOUT_EXPIRED => ClientWaitStatus::TimeoutExpired,
            WebGl2RenderingContext::CONDITION_SATISFIED => ClientWaitStatus::ConditionSatisfied,
            WebGl2RenderingContext::WAIT_FAILED => ClientWaitStatus::WaitFailed,
            _ => unreachable!(),
        }
    }
}

pub struct ClientWait {
    gl: WebGl2RenderingContext,
    flag: FenceSyncFlag,
    flags: usize,
    timeout_ns: usize,
}

impl ClientWait {
    pub fn new(gl: WebGl2RenderingContext, timeout_ns: usize) -> Self {
        Self {
            gl,
            flag: FenceSyncFlag::SyncGpuCommandsComplete,
            flags: ClientWaitFlag::SyncFlushCommandsBit.gl_enum() as usize,
            timeout_ns,
        }
    }

    pub fn wait(&self) -> Result<ClientWaitStatus, Error> {
        let fence_sync = self
            .gl
            .fence_sync(self.flag.gl_enum(), 0)
            .ok_or(Error::CreateFenceSyncFailure)?;
        self.gl.flush();

        let status = self.gl.client_wait_sync_with_u32(
            &fence_sync,
            self.flags as u32,
            self.timeout_ns as u32,
        );
        self.gl.delete_sync(Some(&fence_sync));

        Ok(ClientWaitStatus::from_u32(status))
    }
}

pub struct ClientWaitAsync {
    gl: WebGl2RenderingContext,
    flag: FenceSyncFlag,
    flags: usize,
    timeout_ns: usize,
    interval_ms: usize,
    max_retries: Option<usize>,

    set_timeout_handler: Rc<RefCell<Option<Closure<dyn FnMut()>>>>,
}

impl ClientWaitAsync {
    pub fn new(
        gl: WebGl2RenderingContext,
        timeout_ns: usize,
        interval_ms: usize,
        max_retries: Option<usize>,
    ) -> Self {
        Self {
            gl,
            flag: FenceSyncFlag::SyncGpuCommandsComplete,
            flags: ClientWaitFlag::SyncFlushCommandsBit.gl_enum() as usize,
            timeout_ns,
            interval_ms,
            max_retries,

            set_timeout_handler: Rc::new(RefCell::new(None)),
        }
    }

    pub async fn wait(&self) -> Result<(), Error> {
        let fence_sync = self
            .gl
            .fence_sync(self.flag.gl_enum(), 0)
            .ok_or(Error::CreateFenceSyncFailure)?;
        self.gl.flush();

        for _ in 0..self.max_retries.unwrap_or(usize::MAX) {
            let fence_sync_cloned = fence_sync.clone();
            let gl = self.gl.clone();
            let flags = self.flags;
            let timeout = self.timeout_ns;
            let mut cb = move |resolve: Function, _: Function| {
                let status =
                    gl.client_wait_sync_with_u32(&fence_sync_cloned, flags as u32, timeout as u32);
                resolve
                    .call1(&JsValue::undefined(), &JsValue::from_f64(status as f64))
                    .unwrap();
            };
            let promise = Promise::new(&mut cb);
            let status = wasm_bindgen_futures::JsFuture::from(promise)
                .await
                .unwrap()
                .as_f64()
                .unwrap() as u32;

            match ClientWaitStatus::from_u32(status) {
                ClientWaitStatus::AlreadySignaled | ClientWaitStatus::ConditionSatisfied => {
                    self.gl.delete_sync(Some(&fence_sync));
                    return Ok(());
                }
                ClientWaitStatus::TimeoutExpired => {
                    let set_timeout_handler = self.set_timeout_handler.clone();
                    let mut cb = |resolve: Function, _: Function| {
                        *set_timeout_handler.borrow_mut() = Some(Closure::once(move || {
                            resolve.call0(&JsValue::undefined()).unwrap();
                        }));
                        window()
                            .set_timeout_with_callback_and_timeout_and_arguments_0(
                                set_timeout_handler
                                    .borrow()
                                    .as_ref()
                                    .unwrap()
                                    .as_ref()
                                    .unchecked_ref(),
                                self.interval_ms as i32,
                            )
                            .unwrap();
                    };
                    let promise = Promise::new(&mut cb);
                    wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
                }
                ClientWaitStatus::WaitFailed => {
                    self.gl.delete_sync(Some(&fence_sync));
                    return Err(Error::ClientWaitFailure);
                }
            }
        }

        self.gl.delete_sync(Some(&fence_sync));
        Err(Error::ClientWaitTimeout)
    }
}

// pub async fn client_wait_async(
//     gl: &WebGl2RenderingContext,
//     flags: Option<ClientWaitFlags>,
//     wait_timeout_nanoseconds: usize,
//     retry_interval_milliseconds: Option<usize>,
// ) -> Result<(), Error> {
//     let sync = gl
//         .fence_sync(WebGl2RenderingContext::SYNC_GPU_COMMANDS_COMPLETE, 0)
//         .ok_or(Error::CreateFenceSyncFailure)?;
//     gl.flush();

//     let timeout_callback = Rc::new(RefCell::new(None as Option<Closure<dyn FnMut()>>));
//     let gl_cloned = gl.clone();
//     let sync_cloned = sync.clone();
//     let mut callback = move |resolve: Function, reject: Function| {
//         let gl = gl_cloned.clone();
//         let sync = sync_cloned.clone();
//         let timeout_callback = Rc::clone(&timeout_callback);

//         let wait = Rc::new(RefCell::new(
//             None as Option<Box<dyn Fn(Function, Function)>>,
//         ));
//         let wait_cloned = Rc::clone(&wait);
//         *wait.borrow_mut() = Some(Box::new(move |resolve: Function, reject: Function| {
//             let flags = flags
//                 .map(|flags| flags.gl_enum())
//                 .unwrap_or(WebGl2RenderingContext::NONE);
//             let timeout = (wait_timeout_nanoseconds as u32)
//                 .min(WebGl2RenderingContext::MAX_CLIENT_WAIT_TIMEOUT_WEBGL);

//             let result = match gl.client_wait_sync_with_u32(&sync, flags, timeout) {
//                 WebGl2RenderingContext::ALREADY_SIGNALED => resolve.call0(&JsValue::undefined()),
//                 WebGl2RenderingContext::TIMEOUT_EXPIRED => match retry_interval_milliseconds {
//                     Some(retry_interval_milliseconds) => {
//                         let wait_cloned = Rc::clone(&wait_cloned);
//                         let timeout_callback_cloned = Rc::clone(&timeout_callback);
//                         *timeout_callback.borrow_mut() = Some(Closure::once(move || {
//                             wait_cloned.borrow().as_ref().unwrap()(resolve.clone(), reject.clone());
//                             timeout_callback_cloned.borrow_mut().take();
//                         }));
//                         let result = window()
//                             .set_timeout_with_callback_and_timeout_and_arguments_0(
//                                 timeout_callback
//                                     .borrow()
//                                     .as_ref()
//                                     .unwrap()
//                                     .as_ref()
//                                     .unchecked_ref(),
//                                 retry_interval_milliseconds as i32,
//                             );
//                         match result {
//                             Ok(_) => Ok(JsValue::undefined()),
//                             Err(err) => Err(err),
//                         }
//                     }
//                     None => reject.call0(&JsValue::undefined()),
//                 },
//                 WebGl2RenderingContext::CONDITION_SATISFIED => resolve.call0(&JsValue::undefined()),
//                 WebGl2RenderingContext::WAIT_FAILED => reject.call0(&JsValue::undefined()),
//                 _ => unreachable!(),
//             };

//             if let Err(err) = result {
//                 let msg = err
//                     .dyn_into::<js_sys::Error>()
//                     .ok()
//                     .and_then(|err| err.message().as_string())
//                     .map(|msg| Cow::Owned(msg))
//                     .unwrap_or(Cow::Borrowed("unknown error"));
//                 error!(
//                     target: "ClientWaitAsync",
//                     "Failed to resolve promise: {}",
//                     msg
//                 )
//             }
//         }));

//         wait.borrow().as_ref().unwrap()(resolve, reject);
//     };

//     let promise = Promise::new(&mut callback);
//     let result = wasm_bindgen_futures::JsFuture::from(promise)
//         .await
//         .or_else(|err| Err(Error::ClientWaitFailure(err.as_string())));

//     gl.delete_sync(Some(&sync));

//     result?;
//     Ok(())
// }
