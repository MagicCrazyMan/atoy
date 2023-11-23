use std::{cell::RefCell, rc::Rc};

use gl_matrix4rust::mat4::Mat4;
use gl_matrix4rust::vec3::{AsVec3, Vec3};
use palette::rgb::Rgb;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{closure::Closure, JsCast};
use wasm_bindgen_test::console_log;

use crate::camera::perspective::PerspectiveCamera;
use crate::entity::Entity;
use crate::error::Error;
use crate::geometry::indexed_cube::IndexedCube;
use crate::geometry::sphere::Sphere;
use crate::material::environment_mapping::EnvironmentMaterial;
use crate::material::solid_color_instanced::SolidColorInstancedMaterial;
use crate::material::texture_mapping_instanced::TextureInstancedMaterial;
use crate::render::webgl::texture::TextureUnit;
use crate::{
    geometry::cube::Cube,
    material::solid_color::SolidColorMaterial,
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

    let mat_a = Mat4::from_slice(values_a);
    let mat_b = Mat4::from_slice(values_b);
    for _ in 0..iteration {
        let _ = mat_a * mat_b;
    }

    let end = performance.now();
    console_log!(
        "gl-matrix4rust iterate {} times cost {}ms",
        iteration,
        end - start
    );
}

// static PREALLOCATED: OnceLock<Vec<u8>> = OnceLock::new();

// #[wasm_bindgen]
// pub fn test_memory_prepare(length: usize) {
//     PREALLOCATED.set(vec![1; length]).unwrap();
// }

// #[wasm_bindgen]
// pub fn test_memory_copy(mut buffer: Box<[u8]>) {
//     buffer
//         .as_mut()
//         .write_all(PREALLOCATED.get().unwrap())
//         .unwrap();
// }

// #[wasm_bindgen]
// pub fn test_send_buffer() -> Box<[u8]> {
//     PREALLOCATED.get().unwrap().clone().into_boxed_slice()
// }

fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn create_scene(
    camera_position: impl AsVec3<f64>,
    camera_center: impl AsVec3<f64>,
    camera_up: impl AsVec3<f64>,
) -> Result<Scene, Error> {
    let scene_options = SceneOptions::new()
        .with_mount("scene_container")
        .with_default_camera(PerspectiveCamera::new(
            camera_position,
            camera_center,
            camera_up,
            60.0f64.to_radians(),
            1.0,
            0.5,
            None,
        ));

    Scene::with_options(scene_options)
}

#[wasm_bindgen]
pub fn test_max_combined_texture_image_units() -> Result<(), Error> {
    let scene = create_scene((0.0, 500.0, 0.0), (0.0, 0.0, 0.0), (0.0, 0.0, -1.0))?;
    let render = WebGL2Render::new(&scene)?;
    let count = TextureUnit::max_combined_texture_image_units(render.gl());
    console_log!("max combined texture image units: {}", count);

    Ok(())
}

#[wasm_bindgen]
pub fn test_cube(count: usize, grid: usize, width: f64, height: f64) -> Result<(), Error> {
    let mut scene = create_scene((0.0, 500.0, 0.0), (0.0, 0.0, 0.0), (0.0, 0.0, -1.0))?;

    let cell_width = width / (grid as f64);
    let cell_height = height / (grid as f64);
    let start_x = width / 2.0 - cell_width / 2.0;
    let start_z = height / 2.0 - cell_height / 2.0;
    for index in 0..count {
        let row = index / grid;
        let col = index % grid;

        let center_x = start_x - col as f64 * cell_width;
        let center_z = start_z - row as f64 * cell_height;
        let model_matrix = Mat4::from_translation(&[center_x, 0.0, center_z]);

        let mut entity = Entity::new();

        entity.set_geometry(Some(Cube::new()));
        // entity.set_geometry(Some(IndexedCube::new()));
        entity.set_material(Some(SolidColorMaterial::with_color(rand::random::<Rgb>())));
        entity.set_local_matrix(model_matrix);
        scene.root_entity_mut().add_child(entity);
    }

    let mut collection = Vec::with_capacity(count);
    for _ in 0..count {
        let node = scene.root_entity_mut().remove_child_by_index(0).unwrap();
        collection.push(node);
    }
    scene.root_entity_mut().add_children(collection);

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
            .set_local_matrix(Mat4::from_y_rotation(rotation));

        render.render(&mut scene).unwrap();

        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

#[wasm_bindgen]
pub fn test_instanced_cube(count: i32, grid: i32, width: f64, height: f64) -> Result<(), Error> {
    let mut scene = create_scene((0.0, 500.0, 0.0), (0.0, 0.0, 0.0), (0.0, 0.0, -1.0))?;

    let mut entity = Entity::new();

    // entity.set_geometry(Some(Cube::new()));
    entity.set_geometry(Some(IndexedCube::new()));
    entity.set_material(Some(SolidColorInstancedMaterial::new(
        count, grid, width, height,
    )));
    scene.root_entity_mut().add_child(entity);

    let mut render = WebGL2Render::new(&scene)?;
    render.set_cull_face(Some(CullFace::Back));

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    *(*g).borrow_mut() = Some(Closure::new(move |timestamp: f64| {
        let seconds = timestamp / 1000.0;

        static MAX_SIZE: f64 = 3.0;
        static MIN_SIZE: f64 = 1.0;
        static SIZE_PER_SECOND: f64 = 0.5;
        let size = (seconds * SIZE_PER_SECOND % (MAX_SIZE - MIN_SIZE)) + MIN_SIZE;
        scene
            .root_entity_mut()
            .children_mut()
            .get_mut(0)
            .unwrap()
            .geometry_mut()
            .unwrap()
            .as_any_mut()
            .downcast_mut::<IndexedCube>()
            .unwrap()
            .set_size(size);

        static RADIANS_PER_SECOND: f64 = std::f64::consts::PI / 2.0;
        let rotation = (seconds * RADIANS_PER_SECOND) % (2.0 * std::f64::consts::PI);

        scene
            .root_entity_mut()
            .set_local_matrix(Mat4::from_y_rotation(rotation));

        render.render(&mut scene).unwrap();

        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

#[wasm_bindgen]
pub fn test_texture(
    url: String,
    count: i32,
    grid: i32,
    width: f32,
    height: f32,
) -> Result<(), Error> {
    let mut scene = create_scene((0.0, 20.0, 0.0), (0.0, 0.0, 0.0), (0.0, 0.0, -1.0))?;

    let mut entity = Entity::new();

    // entity.set_geometry(Some(Cube::new()));
    entity.set_geometry(Some(IndexedCube::new()));
    entity.set_material(Some(TextureInstancedMaterial::new(
        url, count, grid, width, height,
    )));
    scene.root_entity_mut().add_child(entity);

    let mut render = WebGL2Render::new(&scene)?;
    render.set_cull_face(Some(CullFace::Back));

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    *(*g).borrow_mut() = Some(Closure::new(move |timestamp: f64| {
        let seconds = timestamp / 1000.0;

        // static MAX_SIZE: f64 = 3.0;
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
        //     .downcast_mut::<IndexedCube>()
        //     .unwrap()
        //     .set_size(size as f32);

        static RADIANS_PER_SECOND: f64 = std::f64::consts::PI / 2.0;
        let rotation = (seconds * RADIANS_PER_SECOND) % (2.0 * std::f64::consts::PI);

        scene
            .root_entity_mut()
            .set_local_matrix(Mat4::from_y_rotation(rotation));
        render.render(&mut scene).unwrap();

        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

#[wasm_bindgen]
pub fn test_environment(
    px: String,
    nx: String,
    py: String,
    ny: String,
    pz: String,
    nz: String,
) -> Result<(), Error> {
    let mut scene = create_scene((2.0, 2.0, 2.0), (0.0, 0.0, 0.0), (0.0, 1.0, 0.0))?;

    let mut entity = Entity::new();

    entity.set_geometry(Some(Sphere::with_opts(1.0, 48, 96)));
    entity.set_material(Some(EnvironmentMaterial::new(px, nx, py, ny, pz, nz)));
    scene.root_entity_mut().add_child(entity);

    let mut render = WebGL2Render::new(&scene)?;
    render.set_cull_face(Some(CullFace::Back));

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    let mut scaling = Vec3::from_values(1.0, 1.0, 1.0);
    *(*g).borrow_mut() = Some(Closure::new(move |timestamp: f64| {
        let seconds = timestamp / 1000.0;

        static MAX_SIZE: f64 = 1.0;
        static MIN_SIZE: f64 = 0.2;
        static SIZE_PER_SECOND: f64 = 0.5;
        let size = (seconds * SIZE_PER_SECOND % (MAX_SIZE - MIN_SIZE)) + MIN_SIZE;
        scaling.0[0] = size;
        scaling.0[1] = size;
        scaling.0[2] = size;
        scene
            .root_entity_mut()
            .children_mut()
            .get_mut(0)
            .unwrap()
            .set_local_matrix(Mat4::from_scaling(&scaling));
        // bad performance below
        // scene
        //     .root_entity_mut()
        //     .children_mut()
        //     .get(0)
        //     .unwrap()
        //     .geometry()
        //     .unwrap()
        //     .borrow_mut()
        //     .as_any_mut()
        //     .downcast_mut::<Sphere>()
        //     .unwrap()
        //     .set_radius(size as f32);

        render.render(&mut scene).unwrap();

        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}
