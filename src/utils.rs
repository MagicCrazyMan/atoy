use std::{
    borrow::{BorrowMut, Cow},
    cell::RefCell,
    io::Write,
    rc::Rc,
    sync::{Arc, Mutex, OnceLock},
    thread::{spawn, sleep}, time::Duration,
};

use gl_matrix4rust::{mat4::Mat4, vec3::Vec3};
use palette::rgb::Rgba;
use wasm_bindgen::{closure::Closure, prelude::wasm_bindgen, JsError};
use wasm_bindgen_test::console_log;

use crate::{
    entity::Entity,
    geometry::cube::Cube,
    material::solid_color::SolidColorMaterial,
    render::webgl::WebGL2Render,
    scene::{Scene, SceneOptions},
    window,
};

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

#[wasm_bindgen]
pub fn test_scene() -> Result<Scene, JsError> {
    let mut scene = Scene::with_options(SceneOptions {
        mount: Some(Cow::Borrowed("scene_container")),
    })?;
    scene
        .active_camera_mut()
        .set_position(Vec3::from_values(2.0, 2.0, 2.0));
    let mut entity = Entity::new_boxed();
    let cube = Cube::new();
    let material = SolidColorMaterial::with_color(Rgba::new(1.0, 0.0, 0.0, 1.0));
    entity.set_geometry(Some(cube));
    entity.set_material(Some(material));
    scene.root_entity_mut().children_mut().push(entity);
    let mut render = WebGL2Render::new(&scene)?;

    render.render(&scene)?;
    render.render(&scene)?;
    render.render(&scene)?;
    render.render(&scene)?;
    render.render(&scene)?;
    render.render(&scene)?;
    render.render(&scene)?;

    Ok(scene)
}
