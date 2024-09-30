pub mod clock;
#[cfg(feature = "webgl")]
pub mod webgl;

use std::{cell::RefCell, rc::Rc};

use js_sys::Promise;
use log::info;
use tokio::sync::broadcast::{self, Receiver};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

use crate::{anewthing::channel::Event, performance, window};

use super::channel::{self, Handler};

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn async_test() {
    let channel = channel::Channel::new();
    struct A;

    impl Handler<M> for A {
        fn handle<'a>(&'a mut self, msg: &mut Event<'_, M>) {
            log::info!(
                "broadcast message received: {}, {}",
                msg.0,
                performance().now()
            );
        }
    }

    struct M(usize);

    {
        let channel = channel.clone();
        let callback: Closure<dyn FnMut()> = Closure::new(move || {
            let num = rand::random::<usize>();
            channel.send(M(num));
            log::info!("broadcast message sent: {num}, {}", performance().now());
        });
        window()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                callback.as_ref().unchecked_ref(),
                5000,
            )
            .unwrap();
        callback.forget();
    }
    for _ in 0..100 {
        channel.on(A);
    }

    // let (tx, _) = broadcast::channel::<usize>(10000);
    // let (abort_tx, mut abort_rx) = broadcast::channel::<()>(10000);
    // let count = Rc::new(RefCell::new(0));

    // {
    //     let tx = tx.clone();
    //     let abort_tx = abort_tx.clone();
    //     let handle = Rc::new(RefCell::new(-1));
    //     let handle_cloned = Rc::clone(&handle);
    //     let callback: Closure<dyn FnMut()> = Closure::new(move || {
    //         *count.borrow_mut() += 1;
    //         if *count.borrow() >= 3 {
    //             abort_tx.send(()).unwrap();
    //             log::info!("broadcast send abort");
    //             window().clear_interval_with_handle(*handle_cloned.borrow());
    //             return;
    //         }

    //         let num = rand::random::<usize>();
    //         tx.send(num).unwrap();
    //         log::info!("broadcast message sent: {num}, {}", performance().now());
    //     });
    //     *handle.borrow_mut() = window()
    //         .set_interval_with_callback_and_timeout_and_arguments_0(
    //             callback.as_ref().unchecked_ref(),
    //             5000,
    //         )
    //         .unwrap();
    //     callback.forget();
    // }

    // for _ in 0..100 {
    //     let mut rx = tx.subscribe();
    //     let mut abort_rx = abort_tx.subscribe();
    //     wasm_bindgen_futures::spawn_local(async move {
    //         loop {
    //             let num = tokio::select! {
    //                 _ = abort_rx.recv() => break,
    //                 err = rx.recv() => {
    //                     match err {
    //                         Ok(num) => num,
    //                         Err(err) => match err {
    //                             broadcast::error::RecvError::Closed => break,
    //                             broadcast::error::RecvError::Lagged(_) => continue,
    //                         }
    //                     }
    //                 }
    //             };
    //             log::info!("broadcast message received: {num}, {}", performance().now());
    //         }
    //         log::info!("broadcast aborted");
    //     });
    // }

    // let rxs: Rc<RefCell<Vec<broadcast::Receiver<usize>>>> = Rc::new(RefCell::new(Vec::new()));
    // let rxs_cloned = Rc::clone(&rxs);
    // async fn func(rxs: Rc<RefCell<Vec<broadcast::Receiver<usize>>>>, mut abort_rx: Receiver<()>) {
    //     log::info!("1");
    //     let abort = match abort_rx.try_recv() {
    //         Ok(_) => true,
    //         Err(err) => match err {
    //             broadcast::error::TryRecvError::Closed => true,
    //             _ => false,
    //         },
    //     };
    //     if abort {
    //         log::info!("broadcast aborted");
    //         return;
    //     }

    //     for rx in rxs.borrow_mut().iter_mut() {
    //         match rx.try_recv() {
    //             Ok(num) => {
    //                 log::info!("broadcast message received: {num}, {}", performance().now());
    //             }
    //             Err(err) => match err {
    //                 broadcast::error::TryRecvError::Closed => continue,
    //                 _ => {}
    //             },
    //         }
    //     }

    //     wasm_bindgen_futures::spawn_local(async move {
    //         log::info!("3");
    //         func(rxs, abort_rx).await;
    //     });
    // }
    // wasm_bindgen_futures::spawn_local(async move {
    //     log::info!("2");
    //     func(rxs_cloned, abort_rx).await;
    // });

    // for _ in 0..100 {
    //     let rx = tx.subscribe();
    //     rxs.borrow_mut().push(rx);
    // }

    // let rxs: Rc<RefCell<Vec<broadcast::Receiver<usize>>>> = Rc::new(RefCell::new(Vec::new()));
    // let rxs_cloned = Rc::clone(&rxs);
    // wasm_bindgen_futures::spawn_local(async move {
    //     loop {
    //         log::info!("1");
    //         let abort = match abort_rx.try_recv() {
    //             Ok(_) => true,
    //             Err(err) => match err {
    //                 broadcast::error::TryRecvError::Closed => true,
    //                 _ => false,
    //             },
    //         };
    //         if abort {
    //             log::info!("broadcast aborted");
    //             return;
    //         }

    //         for rx in rxs_cloned.borrow_mut().iter_mut() {
    //             match rx.try_recv() {
    //                 Ok(num) => {
    //                     log::info!("broadcast message received: {num}, {}", performance().now());
    //                 }
    //                 Err(err) => match err {
    //                     broadcast::error::TryRecvError::Closed => continue,
    //                     _ => {}
    //                 },
    //             }
    //         }

    //         window().set_interval_with_callback(handler)
    //         JsFuture::from(Promise::resolve(&JsValue::undefined())).await.unwrap();
    //     }
    // });

    // for _ in 0..100 {
    //     let rx = tx.subscribe();
    //     rxs.borrow_mut().push(rx);
    // }

    let callback: Rc<RefCell<Option<Closure<dyn Fn()>>>> = Rc::new(RefCell::new(None));
    let callback_cloned = Rc::clone(&callback);
    *callback.borrow_mut() = Some(Closure::new(move || {
        info!("raf, {}", window().performance().unwrap().now());
        window()
            .request_animation_frame(
                (*callback_cloned.borrow())
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unchecked_ref(),
            )
            .unwrap();
    }));
    window()
        .request_animation_frame(
            (*callback.borrow())
                .as_ref()
                .unwrap()
                .as_ref()
                .unchecked_ref(),
        )
        .unwrap();
}
