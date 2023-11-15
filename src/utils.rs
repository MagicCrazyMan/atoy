use std::{borrow::Cow, cell::RefCell, io::Write, rc::Rc, sync::OnceLock};

use gl_matrix4rust::{mat4::Mat4, vec3::Vec3};
use palette::rgb::Rgb;
use wasm_bindgen::{closure::Closure, prelude::wasm_bindgen, JsCast, JsError};
use wasm_bindgen_test::console_log;

use crate::{
    entity::Entity,
    geometry::cube::Cube,
    material::{
        solid_color::SolidColorMaterial, solid_color_instanced::SolidColorInstancedMaterial,
    },
    render::webgl::{CullFace, WebGL2Render},
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

fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

#[wasm_bindgen]
pub fn test_cube(count: i32, grid: i32, width: f32, height: f32) -> Result<(), JsError> {
    let mut scene = Scene::with_options(SceneOptions {
        mount: Some(Cow::Borrowed("scene_container")),
    })?;
    scene
        .active_camera_mut()
        .set_position(Vec3::from_values(0.0, 500.0, 0.0));
    scene
        .active_camera_mut()
        .set_up(Vec3::from_values(0.0, 0.0, -1.0));

    let cell_width = width / (grid as f32);
    let cell_height = height / (grid as f32);
    let start_x = width / 2.0 - cell_width / 2.0;
    let start_z = height / 2.0 - cell_height / 2.0;
    for index in 0..count {
        let row = index / grid;
        let col = index % grid;

        let center_x = start_x - col as f32 * cell_width;
        let center_z = start_z - row as f32 * cell_height;
        let model_matrix = Mat4::from_translation(Vec3::from_values(center_x, 0.0, center_z));

        let mut entity = Entity::new_boxed();

        entity.set_geometry(Some(Cube::new()));
        // entity.set_geometry(Some(IndexedCube::new()));
        entity.set_material(Some(SolidColorMaterial::with_color(rand::random::<Rgb>())));
        entity.set_model_matrix(model_matrix);
        scene.root_entity_mut().add_child_boxed(entity);
    }
    let mut render = WebGL2Render::new(&scene)?;
    render.set_cull_face(Some(CullFace::Back));

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    *(*g).borrow_mut() = Some(Closure::new(move |timestamp: f64| {
        let seconds = timestamp / 1000.0;

        static RADIANS_PER_SECOND: f64 = std::f64::consts::PI / 2.0;
        let rotation = (seconds * RADIANS_PER_SECOND) % (2.0 * std::f64::consts::PI);

        scene
            .root_entity_mut()
            .set_model_matrix(Mat4::from_y_rotation(rotation as f32));
        render.render(&scene);

        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

#[wasm_bindgen]
pub fn test_instanced_cube(count: i32, grid: i32, width: f32, height: f32) -> Result<(), JsError> {
    let mut scene = Scene::with_options(SceneOptions {
        mount: Some(Cow::Borrowed("scene_container")),
    })?;
    scene
        .active_camera_mut()
        .set_position(Vec3::from_values(0.0, 500.0, 0.0));
    scene
        .active_camera_mut()
        .set_up(Vec3::from_values(0.0, 0.0, -1.0));

    let mut entity = Entity::new_boxed();

    entity.set_geometry(Some(Cube::new()));
    // entity.set_geometry(Some(IndexedCube::new()));
    entity.set_material(Some(SolidColorInstancedMaterial::new(
        rand::random::<Rgb>(),
        count,
        grid,
        width,
        height,
    )));
    scene.root_entity_mut().add_child_boxed(entity);
    let mut render = WebGL2Render::new(&scene)?;
    render.set_cull_face(Some(CullFace::Back));

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    *(*g).borrow_mut() = Some(Closure::new(move |timestamp: f64| {
        let seconds = timestamp / 1000.0;

        // static MAX_SIZE: f64 = 2.0;
        // static MIN_SIZE: f64 = 1.0;
        // static SIZE_PER_SECOND: f64 = 0.5;
        // let size = (seconds * SIZE_PER_SECOND % (MAX_SIZE - MIN_SIZE)) + MIN_SIZE;
        // scene
        //     .root_entity_mut()
        //     .children_mut()
        //     .get(0)
        //     .unwrap()
        //     .geometry()
        //     .unwrap()
        //     .borrow_mut()
        //     .as_any_mut()
        //     .downcast_mut::<Cube>()
        //     .unwrap()
        //     .set_size(size as f32);

        static RADIANS_PER_SECOND: f64 = std::f64::consts::PI / 2.0;
        let rotation = (seconds * RADIANS_PER_SECOND) % (2.0 * std::f64::consts::PI);

        scene
            .root_entity_mut()
            .set_model_matrix(Mat4::from_y_rotation(rotation as f32));
        render.render(&scene);

        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}
