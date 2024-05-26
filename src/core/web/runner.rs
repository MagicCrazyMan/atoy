use proc::AsAny;
use wasm_bindgen::closure::Closure;

use crate::core::{
    app::AppConfig,
    runner::{Job, Runner},
};

use super::{cancel_animation_frame, request_animation_frame};

#[derive(AsAny)]
pub struct WebRunner {
    raf_id: *mut i32,
    job: Option<*mut dyn Job>,
    callback: *mut Option<Closure<dyn FnMut(f64)>>,
}

impl Drop for WebRunner {
    fn drop(&mut self) {
        self.stop();

        unsafe {
            drop(Box::from_raw(self.raf_id));
            drop(Box::from_raw(self.callback));
        }
    }
}

impl WebRunner {
    pub fn new() -> Self {
        Self {
            raf_id: Box::into_raw(Box::new(-1)),
            job: None,
            callback: Box::into_raw(Box::new(None)),
        }
    }
}

impl Runner for WebRunner {
    fn new(_: &AppConfig) -> Self
    where
        Self: Sized,
    {
        Self::new()
    }

    fn start(&mut self, job: Box<dyn Job>) {
        self.stop();

        unsafe {
            let raf_id: *mut i32 = self.raf_id;
            let job: *mut dyn Job = Box::into_raw(job);
            let callback: *mut Option<Closure<dyn FnMut(f64)>> = Box::into_raw(Box::new(None));
            *self.callback = Some(Closure::new(move |_| {
                (*job).execute();
                *raf_id = request_animation_frame((*callback).as_ref().unwrap());
            }));
            *self.raf_id = request_animation_frame((*self.callback).as_ref().unwrap());

            self.job = Some(job);
        }
    }

    fn stop(&mut self) {
        unsafe {
            if (*self.raf_id) != -1 {
                cancel_animation_frame(*self.raf_id);
                *self.raf_id = -1;
            }

            if let Some(job) = self.job.take() {
                drop(Box::from_raw(job));
            }

            (*self.callback).take();
        }
    }

    fn running(&self) -> bool {
        unsafe { *self.raf_id != -1 }
    }
}
