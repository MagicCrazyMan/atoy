use std::{borrow::Cow, cell::RefCell, rc::Rc};

use log::error;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{
    js_sys::{Function, Promise},
    WebGl2RenderingContext,
};

use crate::window;

use super::{ conversion::ToGlEnum, error::Error};

/// Available client wait flags mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClientWaitFlags {
    SYNC_FLUSH_COMMANDS_BIT,
}

pub async fn client_wait_async(
    gl: &WebGl2RenderingContext,
    flags: Option<ClientWaitFlags>,
    wait_timeout_nanoseconds: usize,
    retry_interval_milliseconds: Option<usize>,
) -> Result<(), Error> {
    let sync = gl
        .fence_sync(WebGl2RenderingContext::SYNC_GPU_COMMANDS_COMPLETE, 0)
        .ok_or(Error::CreateFenceSyncFailure)?;
    gl.flush();

    let timeout_callback = Rc::new(RefCell::new(None as Option<Closure<dyn FnMut()>>));
    let gl_cloned = gl.clone();
    let sync_cloned = sync.clone();
    let mut callback = move |resolve: Function, reject: Function| {
        let gl = gl_cloned.clone();
        let sync = sync_cloned.clone();
        let timeout_callback = Rc::clone(&timeout_callback);

        let wait = Rc::new(RefCell::new(
            None as Option<Box<dyn Fn(Function, Function)>>,
        ));
        let wait_cloned = Rc::clone(&wait);
        *wait.borrow_mut() = Some(Box::new(move |resolve: Function, reject: Function| {
            let flags = flags
                .map(|flags| flags.gl_enum())
                .unwrap_or(WebGl2RenderingContext::NONE);
            let timeout = (wait_timeout_nanoseconds as u32)
                .min(WebGl2RenderingContext::MAX_CLIENT_WAIT_TIMEOUT_WEBGL);

            let result = match gl.client_wait_sync_with_u32(&sync, flags, timeout) {
                WebGl2RenderingContext::ALREADY_SIGNALED => resolve.call0(&JsValue::undefined()),
                WebGl2RenderingContext::TIMEOUT_EXPIRED => match retry_interval_milliseconds {
                    Some(retry_interval_milliseconds) => {
                        let wait_cloned = Rc::clone(&wait_cloned);
                        let timeout_callback_cloned = Rc::clone(&timeout_callback);
                        *timeout_callback.borrow_mut() = Some(Closure::once(move || {
                            wait_cloned.borrow().as_ref().unwrap()(resolve.clone(), reject.clone());
                            timeout_callback_cloned.borrow_mut().take();
                        }));
                        let result = window()
                            .set_timeout_with_callback_and_timeout_and_arguments_0(
                                timeout_callback
                                    .borrow()
                                    .as_ref()
                                    .unwrap()
                                    .as_ref()
                                    .unchecked_ref(),
                                retry_interval_milliseconds as i32,
                            );
                        match result {
                            Ok(_) => Ok(JsValue::undefined()),
                            Err(err) => Err(err),
                        }
                    }
                    None => reject.call0(&JsValue::undefined()),
                },
                WebGl2RenderingContext::CONDITION_SATISFIED => resolve.call0(&JsValue::undefined()),
                WebGl2RenderingContext::WAIT_FAILED => reject.call0(&JsValue::undefined()),
                _ => unreachable!(),
            };

            if let Err(err) = result {
                let msg = err
                    .dyn_into::<js_sys::Error>()
                    .ok()
                    .and_then(|err| err.message().as_string())
                    .map(|msg| Cow::Owned(msg))
                    .unwrap_or(Cow::Borrowed("unknown error"));
                error!(
                    target: "ClientWaitAsync",
                    "Failed to resolve promise: {}",
                    msg
                )
            }
        }));

        wait.borrow().as_ref().unwrap()(resolve, reject);
    };

    let promise = Promise::new(&mut callback);
    let result = wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .or_else(|err| Err(Error::ClientWaitFailure(err.as_string())));

    gl.delete_sync(Some(&sync));

    result?;
    Ok(())
}
