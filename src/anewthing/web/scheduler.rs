use std::{cell::RefCell, rc::Rc};

use async_trait::async_trait;
use js_sys::{Function, Promise};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};

use crate::{
    anewthing::scheduler::{Scheduler, SchedulingTasks},
    core::web::window,
};

pub struct IntervalScheduler {
    tasks: SchedulingTasks,

    timeout: i32,
    resolve: Rc<RefCell<Option<Function>>>,
    handler: Rc<Closure<dyn FnMut()>>,
    handle: Rc<RefCell<i32>>,
}

impl IntervalScheduler {
    pub fn new(timeout: i32) -> Self {
        let tasks = SchedulingTasks::new();
        let handle = Rc::new(RefCell::new(-1));

        let handler = {
            let tasks = tasks.clone();
            Closure::new(move || {
                // Collects all tasks in current queue and executes.
                // Any other new added tasks are scheduled to next event loop.
                let queue = tasks.raw_mut().drain(..).rev().collect::<Vec<_>>();
                queue.into_iter().for_each(|mut task| task.exec(&tasks));
            })
        };

        Self {
            tasks,

            resolve: Rc::new(RefCell::new(None)),
            handler: Rc::new(handler),
            handle,
            timeout,
        }
    }
}

#[async_trait(?Send)]
impl Scheduler for IntervalScheduler {
    async fn start(&mut self) {
        let resolve_cloned = self.resolve.clone();
        let handler_cloned = self.handler.clone();
        let handle_cloned = self.handle.clone();
        let mut callback = |resolve, _| {
            let handler = (*handler_cloned).as_ref().unchecked_ref();
            let timeout = self.timeout;
            let handle = window()
                .set_interval_with_callback_and_timeout_and_arguments_0(handler, timeout)
                .unwrap();
            *handle_cloned.borrow_mut() = handle;
            *resolve_cloned.borrow_mut() = Some(resolve);
        };
        let promise = Promise::new(&mut callback);
        wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
    }

    async fn stop(&mut self) {
        let handle = self.handle.replace(-1);
        if handle != -1 {
            window().clear_interval_with_handle(handle);
        }
        if let Some(resolve) = self.resolve.borrow_mut().take() {
            resolve.call0(&JsValue::UNDEFINED).unwrap();
        }
    }

    fn running(&self) -> bool {
        *self.handle.borrow() != -1
    }

    fn tasks(&self) -> &SchedulingTasks {
        &self.tasks
    }
}

