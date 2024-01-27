use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::HtmlImageElement;

use crate::render::webgl::uniform::UniformValue;

enum LoadingStatus {
    Unload(*mut dyn FnOnce(HtmlImageElement) -> UniformValue),
    Loading(
        HtmlImageElement,
        Closure<dyn FnMut()>,
        *mut dyn FnOnce(HtmlImageElement) -> UniformValue,
    ),
    Loaded(HtmlImageElement, UniformValue),
}

pub struct TextureLoader {
    url: Option<String>,
    status: *mut LoadingStatus,
}

impl TextureLoader {
    pub fn from_image<F>(image: HtmlImageElement, on_finish: F) -> Self
    where
        F: FnOnce(HtmlImageElement) -> UniformValue + 'static,
    {
        if image.complete() {
            let texture = on_finish(image.clone());

            Self {
                url: None,
                status: Box::leak(Box::new(LoadingStatus::Loaded(image.clone(), texture))),
            }
        } else {
            let on_finish = Box::leak(Box::new(on_finish));
            Self {
                url: None,
                status: Box::leak(Box::new(LoadingStatus::Unload(on_finish))),
            }
        }
    }

    pub fn from_url<S, F>(url: S, on_finish: F) -> Self
    where
        S: Into<String>,
        F: FnOnce(HtmlImageElement) -> UniformValue + 'static,
    {
        let on_finish = Box::leak(Box::new(on_finish));
        Self {
            url: Some(url.into()),
            status: Box::leak(Box::new(LoadingStatus::Unload(on_finish))),
        }
    }

    pub fn load(&mut self) {
        unsafe {
            if let LoadingStatus::Unload(on_finish) = &*self.status {
                if let Some(url) = &self.url {
                    let on_finish = *on_finish;
                    let image = HtmlImageElement::new().unwrap();

                    let status_cloned = self.status;
                    let image_cloned = image.clone();
                    let onload = Closure::once(move || {
                        let on_finish = Box::from_raw(on_finish);
                        let texture = on_finish(image_cloned.clone());

                        // removes callback
                        if let LoadingStatus::Loading(_, callback, _) = &*status_cloned {
                            image_cloned
                                .remove_event_listener_with_callback(
                                    "load",
                                    callback.as_ref().unchecked_ref(),
                                )
                                .unwrap();
                        }

                        *status_cloned = LoadingStatus::Loaded(image_cloned, texture);
                    });
                    image.set_src(url);
                    image
                        .add_event_listener_with_callback("load", onload.as_ref().unchecked_ref())
                        .unwrap();
                    *self.status = LoadingStatus::Loading(image.clone(), onload, on_finish);
                }
            }
        }
    }

    pub fn loaded(&self) -> bool {
        unsafe {
            if let LoadingStatus::Loaded(_, _) = &*self.status {
                true
            } else {
                false
            }
        }
    }

    pub fn texture(&self) -> Option<&UniformValue> {
        unsafe {
            if let LoadingStatus::Loaded(_, texture) = &*self.status {
                Some(texture)
            } else {
                None
            }
        }
    }

    pub fn image(&self) -> Option<&HtmlImageElement> {
        unsafe {
            match &*self.status {
                LoadingStatus::Unload(_) => None,
                LoadingStatus::Loading(img, _, _) | LoadingStatus::Loaded(img, _) => Some(img),
            }
        }
    }

    pub fn url(&self) -> Option<&str> {
        match &self.url {
            Some(url) => Some(url.as_ref()),
            None => None,
        }
    }
}

impl Drop for TextureLoader {
    fn drop(&mut self) {
        unsafe {
            // remove listener from image
            if let LoadingStatus::Loading(image, callback, _) = &*self.status {
                let _ = image
                    .remove_event_listener_with_callback("load", callback.as_ref().unchecked_ref());
            }

            // drops onfinish function
            if let LoadingStatus::Loading(_, _, on_finish) | LoadingStatus::Unload(on_finish) =
                &*self.status
            {
                drop(Box::from_raw(*on_finish));
            }

            drop(Box::from_raw(self.status));
        }
    }
}
