use std::{rc::Rc, cell::RefCell};

use wasm_bindgen::JsValue;
use web_sys::{WebGl2RenderingContext, js_sys::{Function, Promise}};

use super::error::Error;

pub async fn client_wait_async(gl: WebGl2RenderingContext, timeout: u32) -> Result<(), Error> {
    let sync = gl
        .fence_sync(WebGl2RenderingContext::SYNC_GPU_COMMANDS_COMPLETE, 0)
        .ok_or(Error::CreateFenceSyncFailure)?;
    gl.flush();

    let gl_cloned = gl.clone();
    let sync_cloned = sync.clone();
    let mut callback = move |resolve: Function, reject: Function| {
        let gl = gl_cloned.clone();
        let sync = sync_cloned.clone();

        let wait = Rc::new(RefCell::new(
            None as Option<Box<dyn FnMut(Function, Function)>>,
        ));
        let wait_cloned = Rc::clone(&wait);
        *wait.borrow_mut() = Some(Box::new(move |resolve: Function, reject: Function| {
            match gl.client_wait_sync_with_u32(
                &sync,
                WebGl2RenderingContext::SYNC_FLUSH_COMMANDS_BIT,
                timeout,
            ) {
                WebGl2RenderingContext::ALREADY_SIGNALED => {
                    resolve.call0(&JsValue::undefined()).unwrap();
                }
                WebGl2RenderingContext::TIMEOUT_EXPIRED => {
                    wait_cloned.borrow_mut().as_mut().unwrap()(resolve, reject);
                }
                WebGl2RenderingContext::CONDITION_SATISFIED => {
                    resolve.call0(&JsValue::undefined()).unwrap();
                }
                WebGl2RenderingContext::WAIT_FAILED => {
                    reject.call0(&JsValue::undefined()).unwrap();
                }
                _ => unreachable!(),
            };
        }));

        wait.borrow_mut().as_mut().unwrap()(resolve, reject);
    };

    let promise = Promise::new(&mut callback);
    let _ = wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .or_else(|err| Err(Error::ClientWaitFailure(err.as_string())))?;

    gl.delete_sync(Some(&sync));

    Ok(())
}
