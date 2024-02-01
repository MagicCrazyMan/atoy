use std::{borrow::Cow, fmt::Display};

use js_sys::{Function, Promise};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{AddEventListenerOptions, HtmlImageElement};

use crate::{
    error::{AsJsError, Error},
    notify::Notifier,
};

/// Loading status of [`ImageLoader`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadingStatus {
    Unload,
    Loading(HtmlImageElement),
    Loaded(HtmlImageElement),
    Errored(HtmlImageElement),
}

/// Cross origin mode of a [`HtmlImageElement`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageCrossOrigin {
    Anonymous,
    UseCredentials,
}

impl AsRef<str> for ImageCrossOrigin {
    fn as_ref(&self) -> &str {
        match self {
            ImageCrossOrigin::Anonymous => "anonymous",
            ImageCrossOrigin::UseCredentials => "use-credentials",
        }
    }
}

impl Display for ImageCrossOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageCrossOrigin::Anonymous => write!(f, "anonymous"),
            ImageCrossOrigin::UseCredentials => write!(f, "use-credentials"),
        }
    }
}

/// An image loader loads image using [`HtmlImageElement`] from a given url.
pub struct ImageLoader {
    url: String,
    status: *mut LoadingStatus,
    notifier: Notifier<LoadingStatus>,
    cross_origin: Option<ImageCrossOrigin>,
    image: *mut Option<HtmlImageElement>,
    load_callback: *mut Option<Closure<dyn FnMut()>>,
    error_callback: *mut Option<Closure<dyn FnMut()>>,
    promise_callback: *mut Option<Box<dyn FnMut(Function, Function)>>,
    promise_resolve: *mut Option<Function>,
    promise_reject: *mut Option<Function>,
}

impl Drop for ImageLoader {
    fn drop(&mut self) {
        unsafe {
            if let Some(image) = Box::from_raw(self.image).take() {
                // remove listener from image
                if let Some(callback) = Box::from_raw(self.load_callback).take() {
                    let _ = image.remove_event_listener_with_callback(
                        "load",
                        callback.as_ref().unchecked_ref(),
                    );
                }
                if let Some(callback) = Box::from_raw(self.error_callback).take() {
                    let _ = image.remove_event_listener_with_callback(
                        "error",
                        callback.as_ref().unchecked_ref(),
                    );
                }
            }

            drop(Box::from_raw(self.status));
            drop(Box::from_raw(self.promise_callback));
            drop(Box::from_raw(self.promise_resolve));
            drop(Box::from_raw(self.promise_reject));
        }
    }
}

impl ImageLoader {
    /// Constructs a new image loader with given url using [`HtmlImageElement`].
    pub fn new<S>(url: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            url: url.into(),
            status: Box::leak(Box::new(LoadingStatus::Unload)),
            notifier: Notifier::new(),
            cross_origin: None,
            image: Box::leak(Box::new(None)),
            load_callback: Box::leak(Box::new(None)),
            error_callback: Box::leak(Box::new(None)),
            promise_callback: Box::leak(Box::new(None)),
            promise_resolve: Box::leak(Box::new(None)),
            promise_reject: Box::leak(Box::new(None)),
        }
    }

    fn load_inner(
        &mut self,
        success: Option<Function>,
        failed: Option<Function>,
    ) -> Result<(), Error> {
        unsafe {
            if LoadingStatus::Unload != *self.status {
                return Ok(());
            }

            let image = HtmlImageElement::new().unwrap();
            image.set_src(&self.url);
            image.set_cross_origin(self.cross_origin.as_ref().map(|v| v.as_ref()));

            let status = self.status;
            let load_callback = self.load_callback;
            let error_callback = self.error_callback;
            let promise_resolve = self.promise_resolve;
            let promise_reject = self.promise_reject;

            let img = image.clone();
            let mut notifier = self.notifier.clone();
            *self.load_callback = Some(Closure::new(move || {
                *status = LoadingStatus::Loaded(img.clone());
                *load_callback = None;
                *error_callback = None;
                if let Err(err) = img.remove_event_listener_with_callback(
                    "error",
                    (*error_callback).as_ref().unwrap().as_ref().unchecked_ref(),
                ) {
                    log::error!(
                        "remove load callback failure: {}",
                        err.as_error()
                            .and_then(|e| e.message().as_string())
                            .map(|m| Cow::Owned(m))
                            .unwrap_or(Cow::Borrowed("unknown"))
                    );
                }

                notifier.notify(&mut *status);

                if let Some(resolve) = &*promise_resolve {
                    resolve.call0(&JsValue::undefined()).unwrap();
                }
                if let Some(success) = &success {
                    success.call0(&JsValue::undefined()).unwrap();
                }
            }));
            image
                .add_event_listener_with_callback_and_add_event_listener_options(
                    "load",
                    (*self.load_callback)
                        .as_ref()
                        .unwrap()
                        .as_ref()
                        .unchecked_ref(),
                    &AddEventListenerOptions::new().once(true),
                )
                .or_else(|err| {
                    Err(Error::AddEventCallbackFailure(
                        "load",
                        err.as_error().and_then(|err| err.message().as_string()),
                    ))
                })?;

            let img = image.clone();
            let mut notifier = self.notifier.clone();
            *self.error_callback = Some(Closure::new(move || {
                *status = LoadingStatus::Errored(img.clone());
                *load_callback = None;
                *error_callback = None;
                if let Err(err) = img.remove_event_listener_with_callback(
                    "load",
                    (*load_callback).as_ref().unwrap().as_ref().unchecked_ref(),
                ) {
                    log::error!(
                        "remove load callback failure: {}",
                        err.as_error()
                            .and_then(|e| e.message().as_string())
                            .map(|m| Cow::Owned(m))
                            .unwrap_or(Cow::Borrowed("unknown"))
                    );
                }

                notifier.notify(&mut *status);

                if let Some(reject) = &*promise_reject {
                    reject.call0(&JsValue::undefined()).unwrap();
                }
                if let Some(failed) = &failed {
                    failed.call0(&JsValue::undefined()).unwrap();
                }
            }));
            image
                .add_event_listener_with_callback_and_add_event_listener_options(
                    "error",
                    (*self.error_callback)
                        .as_ref()
                        .unwrap()
                        .as_ref()
                        .unchecked_ref(),
                    &AddEventListenerOptions::new().once(true),
                )
                .or_else(|err| {
                    Err(Error::AddEventCallbackFailure(
                        "error",
                        err.as_error().and_then(|err| err.message().as_string()),
                    ))
                })?;

            *self.status = LoadingStatus::Loading(image.clone());
            *self.image = Some(image);
            self.notifier.notify(&mut *self.status);
        }

        Ok(())
    }

    /// Starts loading image.
    /// This method does nothing if image is not in [`LoadingStatus::Unload`] status.
    pub fn load(&mut self) -> Result<(), Error> {
        self.load_inner(None, None)
    }

    /// Starts loading image and asynchronous awaiting.
    /// This method does nothing if image is not in [`LoadingStatus::Unload`] status.
    pub async fn load_async(&mut self) -> Result<(), Error> {
        unsafe {
            if LoadingStatus::Unload != *self.status {
                return Ok(());
            }

            let promise_resolve = self.promise_resolve;
            let promise_reject = self.promise_reject;
            *self.promise_callback = Some(Box::new(move |resolve, reject| {
                *promise_resolve = Some(resolve);
                *promise_reject = Some(reject);
            }));
            let promise = Promise::new((*self.promise_callback).as_deref_mut().unwrap());
            self.load_inner(None, None)?;

            wasm_bindgen_futures::JsFuture::from(promise)
                .await
                .or_else(|err| {
                    Err(Error::PromiseAwaitFailure(
                        err.as_error().and_then(|err| err.message().as_string()),
                    ))
                })?;

            Ok(())
        }
    }

    /// Starts loading image with callback.
    /// This method does nothing if image is not in [`LoadingStatus::Unload`] status.
    pub async fn load_callback(
        &mut self,
        success: Function,
        failed: Function,
    ) -> Result<(), Error> {
        self.load_inner(Some(success), Some(failed))
    }

    /// Returns current loading status.
    pub fn status(&self) -> &LoadingStatus {
        unsafe { &*self.status }
    }

    /// Returns loaded image if successfully loaded.
    pub fn image(&self) -> Option<&HtmlImageElement> {
        unsafe { (*self.image).as_ref() }
    }

    /// Returns image source url.
    pub fn url(&self) -> &str {
        &self.url
    }
}
