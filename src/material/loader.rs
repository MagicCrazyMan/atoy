use std::{cell::RefCell, rc::Rc};

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
    status: Rc<RefCell<LoadingStatus>>,
}

impl TextureLoader {
    pub fn from_image<F: FnOnce(HtmlImageElement) -> UniformValue + 'static>(
        image: HtmlImageElement,
        onfinish: F,
    ) -> Self {
        if image.complete() {
            let texture = onfinish(image.clone());

            Self {
                url: None,
                status: Rc::new(RefCell::new(LoadingStatus::Loaded(image.clone(), texture))),
            }
        } else {
            let onfinish = Box::leak(Box::new(onfinish));
            Self {
                url: None,
                status: Rc::new(RefCell::new(LoadingStatus::Unload(onfinish))),
            }
        }
    }

    pub fn from_url<S: Into<String>, F: FnOnce(HtmlImageElement) -> UniformValue + 'static>(
        url: S,
        onfinish: F,
    ) -> Self {
        let onfinish = Box::leak(Box::new(onfinish));
        Self {
            url: Some(url.into()),
            status: Rc::new(RefCell::new(LoadingStatus::Unload(onfinish))),
        }
    }

    pub fn load(&mut self) {
        let status_mut = &mut *self.status.borrow_mut();
        if let LoadingStatus::Unload(onfinish) = status_mut {
            if let Some(url) = &self.url {
                let onfinish = *onfinish;
                let image = HtmlImageElement::new().unwrap();

                let status_cloned = self.status.clone();
                let image_cloned = image.clone();
                let onload = Closure::once(move || unsafe {
                    let onfinish = Box::from_raw(onfinish);
                    let texture = onfinish(image_cloned.clone());

                    // removes callback
                    if let LoadingStatus::Loading(_, callback, _) = &*status_cloned.borrow() {
                        image_cloned
                            .remove_event_listener_with_callback(
                                "load",
                                callback.as_ref().unchecked_ref(),
                            )
                            .unwrap();
                    }

                    *status_cloned.borrow_mut() = LoadingStatus::Loaded(image_cloned, texture);
                });
                image.set_src(url);
                image
                    .add_event_listener_with_callback("load", onload.as_ref().unchecked_ref())
                    .unwrap();
                *status_mut = LoadingStatus::Loading(image.clone(), onload, onfinish);
            }
        }
    }

    pub fn loaded(&self) -> bool {
        if let LoadingStatus::Loaded(_, _) = &*self.status.borrow() {
            true
        } else {
            false
        }
    }

    pub fn texture(&self) -> Option<UniformValue> {
        if let LoadingStatus::Loaded(_, texture) = &*self.status.borrow() {
            Some(texture.clone())
        } else {
            None
        }
    }

    pub fn image(&self) -> Option<HtmlImageElement> {
        match &*self.status.borrow() {
            LoadingStatus::Unload(_) => None,
            LoadingStatus::Loading(img, _, _) | LoadingStatus::Loaded(img, _) => Some(img.clone()),
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
        // remove listener from image
        if let LoadingStatus::Loading(image, callback, _) = &*self.status.borrow() {
            image
                .remove_event_listener_with_callback("load", callback.as_ref().unchecked_ref())
                .unwrap();
        }

        // drops onfinish function
        if let LoadingStatus::Loading(_, _, onfinish) | LoadingStatus::Unload(onfinish) =
            &*self.status.borrow()
        {
            unsafe {
                drop(Box::from_raw(*onfinish));
            }
        }
    }
}
