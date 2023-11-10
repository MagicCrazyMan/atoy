use std::{fmt::Display, io::Write, sync::OnceLock};

use gl_matrix4rust::mat4::Mat4;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_test::console_log;

use crate::window;

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub struct A {
    data: String,
    parent: Option<*mut dyn B>,
    children: Vec<Box<dyn B>>,
}

impl B for A {
    fn o_0(&self) {
        console_log!("o_0");
    }

    fn data(&self) -> &str {
        &self.data
    }

    fn set_data(&mut self, data: String) {
        self.data = data
    }

    fn parent(&self) -> Option<&dyn B> {
        match &self.parent {
            Some(p) => unsafe {
                let p = &*p.cast_const();
                Some(p)
            },
            None => None,
        }
    }

    fn parent_mut(&mut self) -> Option<&mut dyn B> {
        match &self.parent {
            Some(p) => unsafe {
                let p = &mut **p;
                Some(p)
            },
            None => None,
        }
    }

    fn children(&self) -> &Vec<Box<dyn B>> {
        &self.children
    }

    fn children_mut(&mut self) -> &mut Vec<Box<dyn B>> {
        &mut self.children
    }
}

impl Display for A {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.data())?;
        f.write_str(" ")?;
        match self.parent() {
            Some(parent) => f.write_fmt(format_args!("Parent Data: {}", parent.data()))?,
            None => f.write_str("No Parent")?,
        }
        f.write_str("\n")?;
        self.children()
            .into_iter()
            .try_for_each(|child| child.fmt(f))?;

        Ok(())
    }
}

trait B: Display {
    fn o_0(&self);

    fn data(&self) -> &str;

    fn set_data(&mut self, data: String);

    fn parent(&self) -> Option<&dyn B>;

    fn parent_mut(&mut self) -> Option<&mut dyn B>;

    fn children(&self) -> &Vec<Box<dyn B>>;

    fn children_mut(&mut self) -> &mut Vec<Box<dyn B>>;
}

#[wasm_bindgen]
pub fn test() {
    set_panic_hook();

    let mut parent = A {
        data: String::from("Root"),
        parent: None,
        children: Vec::new(),
    };
    let parent_ptr: *mut dyn B = &mut parent;

    {
        parent.children = (0..100)
            .into_iter()
            .map(|i| {
                let child = A {
                    data: format!("Child_{i}"),
                    parent: Some(parent_ptr),
                    children: Vec::new(),
                };
                // let child_ptr: *mut dyn B = &mut child;
                // console_log!("{:?}", child_ptr);

                let mut child_boxed = Box::new(child) as Box<dyn B>;

                // if i == 50 {
                let child_ptr: *mut dyn B = &mut *child_boxed;
                console_log!("{:?}", child_ptr);
                (0..200).into_iter().for_each(|g| {
                    let grandchild: A = A {
                        data: format!("Grandchild_{g}"),
                        parent: Some(child_ptr),
                        children: Vec::new(),
                    };

                    child_boxed
                        .as_mut()
                        .children_mut()
                        .push(Box::new(grandchild) as Box<dyn B>)
                });
                // }

                child_boxed
            })
            .collect::<Vec<_>>();
    }

    // parent.children.get(10).unwrap().o_0();

    // parent
    //     .children
    //     .get_mut(11)
    //     .unwrap()
    //     .parent_mut()
    //     .unwrap()
    //     .set_data(String::from("AA"));

    // console_log!(
    //     "{}",
    //     parent.children.get(1).unwrap().parent().unwrap().data()
    // );

    console_log!("{}", parent.to_string())
}

#[wasm_bindgen]
pub fn test_gl_matrix_4_rust() {
    struct Random {
        seed: f64,
    }

    impl Random {
        fn new(seed: f64) -> Self {
            Self { seed }
        }

        fn get(&mut self) -> f64 {
            let x = self.seed.sin() * 10000.0;
            self.seed += 1.0;
            return x - x.floor();
        }
    }

    let performance = window()
        .performance()
        .expect("performance should be available");

    console_log!("start benchmark");

    let start = performance.now();

    let iteration = 10000000u32;
    let mut random_a = Random::new(1928473.0);
    let mut random_b = Random::new(1928473.0);

    let mut values_a = [0.0; 4 * 4];
    let mut values_b = [0.0; 4 * 4];
    for i in 0..(4 * 4) {
        values_a[i] = random_a.get();
        values_b[i] = random_b.get();
    }

    let mat_a = Mat4::<f64>::from_slice(&values_a);
    let mat_b = Mat4::<f64>::from_slice(&values_b);
    // let mut out = Mat4::<f64>::new();
    for _ in 0..iteration {
        // mat_a.mul_to(&mat_b, &mut out);
        let _ = mat_a * mat_b;
    }

    let end = performance.now();
    console_log!("gl-matrix4rust duration: {}ms", end - start);
}

static PREALLOCATED: OnceLock<Vec<u8>> = OnceLock::new();

#[wasm_bindgen]
pub fn test_memory_prepare(length: usize) {
    PREALLOCATED.set(vec![1; length]).unwrap();
}

#[wasm_bindgen]
pub fn test_memory_copy(mut buffer: Box<[u8]>) {
    buffer
        .as_mut()
        .write_all(PREALLOCATED.get().unwrap())
        .unwrap();
}

#[wasm_bindgen]
pub fn test_send_buffer() -> Box<[u8]> {
    PREALLOCATED.get().unwrap().clone().into_boxed_slice()
}
