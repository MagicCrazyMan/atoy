use std::{borrow::Cow, fmt::Display};

use js_sys::{Function, Promise};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{AddEventListenerOptions, HtmlImageElement};

use crate::{
    channel::{channel, Receiver, Sender},
    error::{AsJsError, Error},
    renderer::webgl::texture::{
        Builder, SamplerParameter, Texture, Texture2D, TextureData, TextureInternalFormat,
        TextureParameter, TexturePixelStorage, TextureSource, TextureUncompressedData,
        TextureUncompressedInternalFormat, TextureUncompressedPixelDataType,
        TextureUncompressedPixelFormat,
    },
};

use super::{Loader, LoaderStatus};

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

/// An texture loader loads texture using [`HtmlImageElement`] from a given url.
pub struct TextureLoader {
    url: String,
    status: *mut LoaderStatus,
    channel: (Sender<LoaderStatus>, Receiver<LoaderStatus>),
    cross_origin: Option<ImageCrossOrigin>,
    image: *mut Option<HtmlImageElement>,
    error: *mut Option<Error>,

    is_srgb: bool,
    generate_mipmaps: bool,
    pixel_storages: Vec<TexturePixelStorage>,
    sampler_params: Vec<SamplerParameter>,
    texture_params: Vec<TextureParameter>,

    load_callback: *mut Option<Closure<dyn FnMut()>>,
    error_callback: *mut Option<Closure<dyn FnMut(js_sys::Error)>>,

    promise_callback: *mut Option<Box<dyn FnMut(Function, Function)>>,
    promise_resolve: *mut Option<Function>,
    promise_reject: *mut Option<Function>,
}

impl Drop for TextureLoader {
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
            drop(Box::from_raw(self.error));
            drop(Box::from_raw(self.promise_callback));
            drop(Box::from_raw(self.promise_resolve));
            drop(Box::from_raw(self.promise_reject));
        }
    }
}

impl TextureLoader {
    /// Constructs a new texture loader.
    pub fn new<S>(url: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            url: url.into(),
            status: Box::leak(Box::new(LoaderStatus::Unload)),
            channel: channel(),
            cross_origin: None,
            image: Box::leak(Box::new(None)),
            error: Box::leak(Box::new(None)),

            is_srgb: false,
            generate_mipmaps: true,
            pixel_storages: Vec::new(),
            sampler_params: Vec::new(),
            texture_params: Vec::new(),

            load_callback: Box::leak(Box::new(None)),
            error_callback: Box::leak(Box::new(None)),

            promise_callback: Box::leak(Box::new(None)),
            promise_resolve: Box::leak(Box::new(None)),
            promise_reject: Box::leak(Box::new(None)),
        }
    }

    /// Constructs a new texture loader with parameters.
    pub fn with_params<S, PI, SI, TI>(
        url: S,
        pixel_storages: PI,
        sampler_params: SI,
        texture_params: TI,
        generate_mipmaps: bool,
        is_srgb: bool,
    ) -> Self
    where
        S: Into<String>,
        PI: IntoIterator<Item = TexturePixelStorage>,
        SI: IntoIterator<Item = SamplerParameter>,
        TI: IntoIterator<Item = TextureParameter>,
    {
        Self {
            url: url.into(),
            status: Box::leak(Box::new(LoaderStatus::Unload)),
            channel: channel(),
            cross_origin: None,
            image: Box::leak(Box::new(None)),
            error: Box::leak(Box::new(None)),

            is_srgb,
            generate_mipmaps,
            pixel_storages: pixel_storages.into_iter().collect(),
            sampler_params: sampler_params.into_iter().collect(),
            texture_params: texture_params.into_iter().collect(),

            load_callback: Box::leak(Box::new(None)),
            error_callback: Box::leak(Box::new(None)),

            promise_callback: Box::leak(Box::new(None)),
            promise_resolve: Box::leak(Box::new(None)),
            promise_reject: Box::leak(Box::new(None)),
        }
    }

    fn load_inner(
        &self,
        success: Option<Function>,
        failure: Option<Function>,
    ) -> Result<(), Error> {
        unsafe {
            let image = HtmlImageElement::new().unwrap();
            image.set_src(&self.url);
            image.set_cross_origin(self.cross_origin.as_ref().map(|v| v.as_ref()));

            let status = self.status;
            let error = self.error;
            let load_callback = self.load_callback;
            let error_callback = self.error_callback;
            let promise_resolve = self.promise_resolve;
            let promise_reject = self.promise_reject;

            let img = image.clone();
            let sender = self.channel.0.clone();
            *self.load_callback = Some(Closure::new(move || {
                *status = LoaderStatus::Loaded;
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

                sender.send(&mut *status);

                if let Some(resolve) = &*promise_resolve {
                    resolve.call0(&JsValue::undefined()).unwrap();
                }
                if let Some(success) = &success {
                    success.call0(&JsValue::undefined()).unwrap();
                }

                *load_callback = None;
                *error_callback = None;
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
            let sender = self.channel.0.clone();
            *self.error_callback = Some(Closure::new(move |err: js_sys::Error| {
                let msg = err.as_error().and_then(|e| e.message().as_string());
                log::error!(
                    "remove load callback failure: {}",
                    msg.as_ref().map(|m| m.as_str()).unwrap_or("unknown")
                );

                *status = LoaderStatus::Errored;
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

                sender.send(&mut *status);

                if let Some(reject) = &*promise_reject {
                    reject.call0(&JsValue::undefined()).unwrap();
                }
                if let Some(failure) = &failure {
                    failure.call0(&JsValue::undefined()).unwrap();
                }

                *load_callback = None;
                *error_callback = None;
                *error = Some(Error::CommonError(msg));
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

            *self.status = LoaderStatus::Loading;
            *self.image = Some(image);
            self.channel.0.send(&mut *self.status);
        }

        Ok(())
    }

    /// Starts loading image.
    /// This method does nothing if image is not in [`LoaderStatus::Unload`] status.
    pub fn load(&self) {
        unsafe {
            if LoaderStatus::Unload == *self.status {
                if let Err(err) = self.load_inner(None, None) {
                    *self.status = LoaderStatus::Errored;
                    *self.error = Some(err);
                    self.channel.0.send(&*self.status);
                }
            }
        }
    }

    /// Starts loading image and puts it into a [`Promise`].
    /// This method does nothing if image is not in [`LoaderStatus::Unload`] status.
    pub fn load_promise(&self) -> Promise {
        unsafe {
            if LoaderStatus::Unload == *self.status {
                let promise_resolve = self.promise_resolve;
                let promise_reject = self.promise_reject;
                *self.promise_callback = Some(Box::new(move |resolve, reject| {
                    *promise_resolve = Some(resolve);
                    *promise_reject = Some(reject);
                }));
                let promise = Promise::new((*self.promise_callback).as_deref_mut().unwrap());
                if let Err(err) = self.load_inner(None, None) {
                    *self.status = LoaderStatus::Errored;
                    *self.error = Some(err);
                    self.channel.0.send(&*self.status);
                    (*promise_reject)
                        .as_ref()
                        .unwrap()
                        .call0(&JsValue::undefined())
                        .unwrap();
                }

                promise
            } else {
                Promise::resolve(&JsValue::undefined())
            }
        }
    }

    /// Starts loading image and asynchronous awaiting.
    /// This method does nothing if image is not in [`LoaderStatus::Unload`] status.
    pub async fn load_async(&self) -> Result<&HtmlImageElement, Error> {
        unsafe {
            let promise = self.load_promise();
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(_) => Ok((*self.image).as_ref().unwrap()),
                Err(_) => Err((*self.error).clone().unwrap()),
            }
        }
    }

    /// Returns image regardless whether successfully loaded or not.
    pub fn image(&self) -> Option<&HtmlImageElement> {
        unsafe { (*self.image).as_ref() }
    }

    /// Returns loaded image if successfully loaded.
    pub fn loaded_image(&self) -> Option<&HtmlImageElement> {
        unsafe {
            match &*self.status {
                LoaderStatus::Unload | LoaderStatus::Loading | LoaderStatus::Errored => None,
                LoaderStatus::Loaded => (*self.image).as_ref(),
            }
        }
    }

    /// Returns image source url.
    pub fn url(&self) -> &str {
        &self.url
    }
}

impl Loader<Texture<Texture2D>> for TextureLoader {
    type Failure = Error;

    fn status(&self) -> LoaderStatus {
        unsafe { *self.status }
    }

    fn load(&mut self) {
        Self::load(&self);
    }

    fn loaded(&self) -> Result<Texture<Texture2D>, Error> {
        unsafe {
            if let Some(err) = &*self.error {
                return Err(err.clone());
            }

            let image = (*self.image).as_ref().unwrap();
            let mut builder = Builder::<Texture2D>::with_auto_levels(
                TextureInternalFormat::Uncompressed(if self.is_srgb {
                    TextureUncompressedInternalFormat::SRGB8_ALPHA8
                } else {
                    TextureUncompressedInternalFormat::RGBA8
                }),
                image.natural_width() as usize,
                image.natural_height() as usize,
            );
            builder.set_texture_parameters(self.texture_params.iter().map(|p| *p));
            builder.set_sampler_parameters(self.sampler_params.iter().map(|p| *p));
            builder.tex_image(
                HtmlImageTextureSource::new(image.clone(), self.pixel_storages.clone()),
                0,
                self.generate_mipmaps,
            );

            Ok(builder.build())
        }
    }

    fn success(&self) -> Receiver<LoaderStatus> {
        self.channel.1.clone()
    }
}

struct HtmlImageTextureSource {
    image: HtmlImageElement,
    pixel_storages: Vec<TexturePixelStorage>,
}

impl HtmlImageTextureSource {
    fn new(image: HtmlImageElement, pixel_storages: Vec<TexturePixelStorage>) -> Self {
        Self {
            image,
            pixel_storages,
        }
    }
}

impl TextureSource for HtmlImageTextureSource {
    fn data(&self) -> TextureData {
        TextureData::Uncompressed {
            pixel_format: TextureUncompressedPixelFormat::RGBA,
            pixel_data_type: TextureUncompressedPixelDataType::UNSIGNED_BYTE,
            pixel_storages: self.pixel_storages.clone(),
            data: TextureUncompressedData::HtmlImageElement {
                data: self.image.clone(),
            },
        }
    }
}
