use wasm_bindgen::closure::Closure;

use crate::core::looper::JobLooper;

use super::{cancel_animation_frame, request_animation_frame};

pub struct WebJobLooper {
    raf_id: *mut i32,
    job: Option<*mut dyn FnMut()>,
    callback: *mut Option<Closure<dyn FnMut(f64)>>,
}

impl Drop for WebJobLooper {
    fn drop(&mut self) {
        self.stop();

        unsafe {
            drop(Box::from_raw(self.raf_id));
            drop(Box::from_raw(self.callback));
        }
    }
}

impl WebJobLooper {
    pub fn new() -> Self {
        Self {
            raf_id: Box::into_raw(Box::new(-1)),
            job: None,
            callback: Box::into_raw(Box::new(None)),
        }
    }
}

impl JobLooper for WebJobLooper {
    fn start<J>(&mut self, job: J)
    where
        J: FnMut() + 'static,
    {
        self.stop();

        unsafe {
            let raf_id: *mut i32 = self.raf_id;
            let job: *mut dyn FnMut() = Box::into_raw(Box::new(job));
            let callback: *mut Option<Closure<dyn FnMut(f64)>> = Box::into_raw(Box::new(None));
            *self.callback = Some(Closure::new(move |_| {
                (*job)();
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

    fn is_running(&self) -> bool {
        unsafe { *self.raf_id != -1 }
    }
}
