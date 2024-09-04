use std::{borrow::Cow, cell::RefCell, rc::Rc, time::Duration};

use log::error;
use proc::GlEnum;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    js_sys::{Function, Promise},
    WebGl2RenderingContext,
};

use crate::window;

use super::error::Error;

/// Available client wait flags mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlClientWaitFlag {
    SyncFlushCommandsBit,
}

/// Available client wait condition mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlClientWaitCondition {
    SyncGpuCommandsComplete,
}

pub struct WebGlClientWait {
    gl: WebGl2RenderingContext,
    flag_bits: u32,
    wait_timeout: Duration,
    retry_interval: Duration,
    max_retries: usize,
}

impl WebGlClientWait {
    /// Constructs a new client wait.
    pub fn new(gl: WebGl2RenderingContext, wait_timeout: Duration) -> Self {
        Self::with_retries(gl, wait_timeout, Duration::from_millis(0), 0)
    }

    /// Constructs a new client wait with retries.
    pub fn with_retries(
        gl: WebGl2RenderingContext,
        wait_timeout: Duration,
        retry_interval: Duration,
        max_retries: usize,
    ) -> Self {
        Self {
            gl,
            flag_bits: WebGl2RenderingContext::NONE,
            wait_timeout,
            retry_interval,
            max_retries,
        }
    }

    /// Constructs a new client wait with flags.
    pub fn with_flags<I>(gl: WebGl2RenderingContext, wait_timeout: Duration, flags: I) -> Self
    where
        I: IntoIterator<Item = WebGlClientWaitFlag>,
    {
        Self::with_flags_and_retries(gl, wait_timeout, Duration::from_millis(0), 0, flags)
    }

    /// Constructs a new client wait with flags.
    pub fn with_flags_and_retries<I>(
        gl: WebGl2RenderingContext,
        wait_timeout: Duration,
        retry_interval: Duration,
        max_retries: usize,
        flags: I,
    ) -> Self
    where
        I: IntoIterator<Item = WebGlClientWaitFlag>,
    {
        let mut flag_bits = WebGl2RenderingContext::NONE;
        flags.into_iter().for_each(|flag| {
            flag_bits |= flag.to_gl_enum();
        });

        Self {
            gl,
            flag_bits,
            wait_timeout,
            retry_interval,
            max_retries,
        }
    }

    /// Executes client wait.
    pub async fn client_wait(self) -> Result<(), Error> {
        let gl = self.gl.clone();
        let flag_bits = self.flag_bits;
        let wait_timeout = self.wait_timeout;
        let retry_interval = self.retry_interval;
        let max_retries = self.max_retries;

        let sync = gl
            .fence_sync(
                WebGlClientWaitCondition::SyncGpuCommandsComplete.to_gl_enum(),
                flag_bits,
            )
            .ok_or(Error::CreateFenceSyncFailure)?;
        self.gl.flush();

        let gl_cloned = self.gl.clone();
        let sync_cloned = sync.clone();
        let mut promise_callback = move |resolve: Function, reject: Function| {
            let gl = gl_cloned.clone();
            let sync = sync_cloned.clone();

            let retry_callback = Rc::new(RefCell::new(None as Option<Closure<dyn FnMut()>>));
            let retries = Rc::new(RefCell::new(0));
            let wait = Rc::new(RefCell::new(
                None as Option<Box<dyn Fn(Function, Function)>>,
            ));

            let wait_cloned = Rc::clone(&wait);
            *wait.borrow_mut() = Some(Box::new(move |resolve: Function, reject: Function| {
                let result = match gl.client_wait_sync_with_u32(
                    &sync,
                    flag_bits,
                    wait_timeout.as_nanos() as u32,
                ) {
                    WebGl2RenderingContext::ALREADY_SIGNALED => {
                        resolve.call0(&JsValue::undefined())
                    }
                    WebGl2RenderingContext::TIMEOUT_EXPIRED => {
                        if *retries.borrow() >= max_retries {
                            reject.call0(&JsValue::undefined())
                        } else {
                            let wait_cloned = Rc::clone(&wait_cloned);
                            let retry_callback_cloned = Rc::clone(&retry_callback);
                            *retry_callback.borrow_mut() = Some(Closure::once(move || {
                                wait_cloned.borrow().as_ref().unwrap()(
                                    resolve.clone(),
                                    reject.clone(),
                                );
                                retry_callback_cloned.borrow_mut().take();
                            }));
                            window()
                                .set_timeout_with_callback_and_timeout_and_arguments_0(
                                    retry_callback
                                        .borrow()
                                        .as_ref()
                                        .unwrap()
                                        .as_ref()
                                        .unchecked_ref(),
                                    retry_interval.as_millis() as i32,
                                )
                                .map(|_| JsValue::undefined())
                        }
                    }
                    WebGl2RenderingContext::CONDITION_SATISFIED => {
                        resolve.call0(&JsValue::undefined())
                    }
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

        let result = JsFuture::from(Promise::new(&mut promise_callback))
            .await
            .map(|_| ())
            .or_else(|err| Err(Error::ClientWaitFailure(err.as_string())));

        self.gl.delete_sync(Some(&sync));

        result
    }
}
