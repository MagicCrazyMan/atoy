use std::collections::HashMap;
use std::f64::consts::PI;
use std::ops::Mul;
use std::{cell::RefCell, rc::Rc};

use gl_matrix4rust::mat4::{AsMat4, Mat4};
use gl_matrix4rust::quat::Quat;
use gl_matrix4rust::vec2::{AsVec2, Vec2};
use gl_matrix4rust::vec3::{AsVec3, Vec3};
use gl_matrix4rust::vec4::{AsVec4, Vec4};
use palette::rgb::{Rgb, Rgba};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{closure::Closure, JsCast};
use wasm_bindgen_test::console_log;
use web_sys::js_sys::{Date, Uint8Array};
use web_sys::MouseEvent;

use crate::camera::perspective::PerspectiveCamera;
use crate::camera::universal::UniversalCamera;
use crate::camera::Camera;
use crate::entity::{Entity, Weak};
use crate::error::Error;
use crate::geometry::indexed_cube::IndexedCube;
use crate::geometry::multicube::MultiCube;
use crate::geometry::raw::RawGeometry;
use crate::geometry::rectangle::{Placement, Rectangle};
use crate::geometry::sphere::Sphere;
use crate::light::ambient::SimpleAmbientLight;
use crate::light::diffuse::SimpleDiffuseLight;
use crate::light::specular::SimpleSpecularLight;
use crate::material::environment_mapping::EnvironmentMaterial;
use crate::material::icon::IconMaterial;
use crate::material::loader::TextureLoader;
use crate::material::multiple_textures_instanced::MultipleTexturesInstanced;
use crate::material::solid_color_instanced::SolidColorInstancedMaterial;
use crate::material::texture_mapping::TextureMaterial;
use crate::material::texture_mapping_instanced::TextureInstancedMaterial;
use crate::material::{self, Transparency};
use crate::render::pp::ResourceKey;
use crate::render::webgl::attribute::AttributeValue;
use crate::render::webgl::buffer::{
    BufferComponentSize, BufferDataType, BufferDescriptor, BufferSource, BufferTarget, BufferUsage,
};
use crate::render::webgl::draw::{Draw, DrawMode};
use crate::render::webgl::pipeline::create_standard_pipeline;
use crate::render::webgl::pipeline::picking::create_picking_pipeline;
use crate::render::webgl::texture::{
    TextureDataType, TextureDescriptor, TextureFormat, TextureMagnificationFilter,
    TextureMinificationFilter, TextureParameter, TexturePixelStorage, TextureUnit,
    TextureWrapMethod,
};
use crate::render::webgl::uniform::UniformValue;
use crate::scene::SceneStuff;
use crate::utils::slice_to_float32_array;
use crate::{document, entity};
use crate::{
    geometry::cube::Cube,
    material::solid_color::SolidColorMaterial,
    render::webgl::{draw::CullFace, WebGL2Render},
    scene::Scene,
    window,
};

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

// // static PREALLOCATED: OnceLock<Vec<u8>> = OnceLock::new();

// // #[wasm_bindgen]
// // pub fn test_memory_prepare(length: usize) {
// //     PREALLOCATED.set(vec![1; length]).unwrap();
// // }

// // #[wasm_bindgen]
// // pub fn test_memory_copy(mut buffer: Box<[u8]>) {
// //     buffer
// //         .as_mut()
// //         .write_all(PREALLOCATED.get().unwrap())
// //         .unwrap();
// // }

// // #[wasm_bindgen]
// // pub fn test_send_buffer() -> Box<[u8]> {
// //     PREALLOCATED.get().unwrap().clone().into_boxed_slice()
// // }

fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn create_scene(
    camera_position: impl AsVec3<f64>,
    camera_center: impl AsVec3<f64>,
    camera_up: impl AsVec3<f64>,
) -> Scene {
    let mut scene = Scene::new();
    scene.set_active_camera(UniversalCamera::new(
        camera_position,
        camera_center,
        camera_up,
        60.0f64.to_radians(),
        1.0,
        0.5,
        Some(200.0),
    ));
    scene.set_ambient_light(Some(SimpleAmbientLight::new(0.2, 0.2, 0.2)));
    scene.add_diffuse_light(SimpleDiffuseLight::new(
        Vec3::from_values(0.0, 6.0, 0.0),
        Vec3::from_values(1.0, 1.0, 1.0),
        Vec3::from_values(1.0, 0.05, 0.0),
    ));
    scene.add_specular_light(SimpleSpecularLight::new(
        Vec3::from_values(0.0, 12.0, 0.0),
        Vec3::from_values(1.0, 1.0, 1.0),
        128.0,
        Vec3::from_values(1.0, 0.05, 0.0),
    ));
    scene.add_specular_light(SimpleSpecularLight::new(
        Vec3::from_values(6.0, 12.0, 0.0),
        Vec3::from_values(1.0, 1.0, 1.0),
        128.0,
        Vec3::from_values(1.0, 0.05, 0.0),
    ));
    scene
}

fn create_render() -> Result<WebGL2Render, Error> {
    let render = WebGL2Render::with_mount("scene_container")?;

    Ok(render)
}

// #[wasm_bindgen]
// pub fn test_max_combined_texture_image_units() -> Result<(), Error> {
//     let scene = create_scene((0.0, 500.0, 0.0), (0.0, 0.0, 0.0), (0.0, 0.0, -1.0))?;
//     let render = create_render(&scene)?;
//     let count = TextureUnit::max_combined_texture_image_units(render.gl());
//     console_log!("max combined texture image units: {}", count);

//     Ok(())
// }

#[wasm_bindgen]
pub fn test_cube(count: usize, grid: usize, width: f64, height: f64) -> Result<(), Error> {
    let mut scene = create_scene((0.0, 5.0, 5.0), (0.0, 0.0, 0.0), (0.0, 1.0, 0.0));
    // let mut scene = create_scene((0.0, 500.0, 0.0), (0.0, 0.0, 0.0), (0.0, 0.0, -1.0))?;
    let render = create_render()?;
    let render = Rc::new(RefCell::new(render));
    let last_frame_time = Rc::new(RefCell::new(0.0));
    let mut picking_pipeline = create_picking_pipeline(
        ResourceKey::new_persist_str("position"),
        ResourceKey::new_persist_str("picked_entity"),
        ResourceKey::new_persist_str("picked_position"),
    );
    let clear_color_key = ResourceKey::new_persist_str("clear_color");
    let mut standard_pipeline = create_standard_pipeline(
        ResourceKey::new_persist_str("position"),
        clear_color_key.clone(),
    );
    standard_pipeline
        .resources_mut()
        .insert(clear_color_key, (0.0, 0.0, 0.0, 1.0));
    let standard_pipeline = Rc::new(RefCell::new(standard_pipeline));

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

        let entity = Entity::new();

        entity.borrow_mut().set_geometry(Some(Cube::new()));
        // entity.set_geometry(Some(IndexedCube::new()));
        entity
            .borrow_mut()
            .set_material(Some(SolidColorMaterial::with_color(rand::random())));
        entity.borrow_mut().set_local_matrix(model_matrix);
        scene.entity_collection_mut().add_entity(entity);
    }

    // let entity = Entity::new();
    // entity.borrow_mut().set_geometry(Some(Rectangle::new(
    //     Vec2::from_values(0.0, 0.0),
    //     Placement::Center,
    //     4.0,
    //     4.0,
    // )));
    // entity.borrow_mut().set_material(Some(IconMaterial::new(
    //     TextureLoader::from_url("./skybox/skybox_py.jpg", |image| UniformValue::Texture {
    //         descriptor: TextureDescriptor::texture_2d_with_html_image_element(
    //             image,
    //             TextureDataType::UnsignedByte,
    //             TextureFormat::RGB,
    //             TextureFormat::RGB,
    //             0,
    //             vec![TexturePixelStorage::UnpackFlipYWebGL(true)],
    //             true,
    //         ),
    //         params: vec![
    //             TextureParameter::MinFilter(TextureMinificationFilter::LinearMipmapLinear),
    //             TextureParameter::MagFilter(TextureMagnificationFilter::Linear),
    //             TextureParameter::WrapS(TextureWrapMethod::ClampToEdge),
    //             TextureParameter::WrapT(TextureWrapMethod::ClampToEdge),
    //         ],
    //         texture_unit: TextureUnit::TEXTURE0,
    //     }),
    //     Transparency::Opaque,
    // )));
    // scene.entity_collection_mut().add_entity(entity);

    let entity = Entity::new();
    entity.borrow_mut().set_geometry(Some(Rectangle::new(
        Vec2::from_values(0.0, 0.0),
        Placement::TopLeft,
        0.25,
        0.25,
        1.0,
        1.0,
    )));
    entity
        .borrow_mut()
        .set_material(Some(MultipleTexturesInstanced::new()));
    scene.entity_collection_mut().add_entity(entity);

    let floor_entity = Entity::new();
    floor_entity.borrow_mut().set_geometry(Some(Rectangle::new(
        Vec2::from_values(0.0, 0.0),
        Placement::Center,
        10000.0,
        10000.0,
        2000.0,
        2000.0,
    )));
    floor_entity
        .borrow_mut()
        .set_material(Some(TextureMaterial::new("./wood.png")));
    floor_entity
        .borrow_mut()
        .set_local_matrix(Mat4::from_rotation_translation(
            &Quat::from_axis_angle(&Vec3::from_values(1.0, 0.0, 0.0), PI / 2.0),
            &Vec3::from_values(0.0, -0.5, 0.0),
        ));
    scene.entity_collection_mut().add_entity(floor_entity);

    let scene = Rc::new(RefCell::new(scene));

    let render_cloned = Rc::clone(&render);
    let scene_cloned = Rc::clone(&scene);
    let last_frame_time_cloned = Rc::clone(&last_frame_time);
    let click = Closure::<dyn FnMut(MouseEvent)>::new(move |event: MouseEvent| {
        let x = event.page_x();
        let y = event.page_y();

        let start = window().performance().unwrap().now();
        // sets pick position
        picking_pipeline
            .resources_mut()
            .insert(ResourceKey::new_persist_str("position"), (x, y));

        // pick
        render_cloned
            .borrow_mut()
            .render(
                &mut picking_pipeline,
                &mut SceneStuff::new(&mut scene_cloned.borrow_mut()),
                *last_frame_time_cloned.borrow(),
            )
            .unwrap();
        let end = window().performance().unwrap().now();
        document()
            .get_element_by_id("pick")
            .unwrap()
            .set_inner_html(&format!("{:.2}", end - start));

        // get entity
        if let Some(entity) = picking_pipeline
            .resources()
            .get(&ResourceKey::<Weak>::new_persist_str("picked_entity"))
            .and_then(|e| e.upgrade())
        {
            console_log!("pick entity {}", entity.borrow().id());

            entity
                .borrow_mut()
                .material_mut()
                .and_then(|material| material.as_any_mut().downcast_mut::<SolidColorMaterial>())
                .map(|material| material.set_color(rand::random()));
        }
        // get position
        if let Some(position) = picking_pipeline
            .resources()
            .get(&ResourceKey::<Vec3>::new_persist_str("picked_position"))
        {
            console_log!("pick position {}", position);
        }
    });
    window()
        .add_event_listener_with_callback("click", click.as_ref().unchecked_ref())
        .unwrap();
    click.forget();

    let standard_pipeline_cloned = Rc::clone(&standard_pipeline);
    let click = Closure::<dyn FnMut(MouseEvent)>::new(move |event: MouseEvent| {
        standard_pipeline_cloned
            .borrow_mut()
            .resources_mut()
            .insert(
                ResourceKey::new_persist_str("position"),
                (event.page_x(), event.page_y()),
            );
    });
    window()
        .add_event_listener_with_callback("mousemove", click.as_ref().unchecked_ref())
        .unwrap();
    click.forget();

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    *(*g).borrow_mut() = Some(Closure::new(move |frame_time: f64| {
        let seconds = frame_time / 1000.0;

        static RADIANS_PER_SECOND: f64 = std::f64::consts::PI / 4.0;
        let rotation = (seconds * RADIANS_PER_SECOND) % (2.0 * std::f64::consts::PI);

        // scene
        //     .borrow_mut()
        //     .active_camera_mut()
        //     .as_any_mut()
        //     .downcast_mut::<PerspectiveCamera>()
        //     .unwrap()
        //     .set_center(&(rotation.cos() * 6.0, 0.0, rotation.sin() * 6.0));
        // .set_up(&(rotation.cos(), 0.0, rotation.sin()));

        let start = window().performance().unwrap().now();
        render
            .borrow_mut()
            .render(
                &mut standard_pipeline.borrow_mut(),
                &mut SceneStuff::new(&mut scene.borrow_mut()),
                frame_time,
            )
            .unwrap();
        let end = window().performance().unwrap().now();
        document()
            .get_element_by_id("total")
            .unwrap()
            .set_inner_html(&format!("{:.2}", end - start));

        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

#[wasm_bindgen]
pub fn test_instanced_cube(
    count: usize,
    grid: usize,
    width: f64,
    height: f64,
) -> Result<(), Error> {
    let mut scene = create_scene((0.0, 5.0, 5.0), (0.0, 0.0, 0.0), (0.0, 1.0, 0.0));
    let mut render = create_render()?;
    let mut pipeline = create_standard_pipeline(
        ResourceKey::new_persist_str("position"),
        ResourceKey::new_persist_str("clear_color"),
    );

    // let pick_position = Rc::new(RefCell::new(None as Option<(i32, i32)>));
    // let pick_position_cloned = Rc::clone(&pick_position);
    // let click = Closure::<dyn FnMut(MouseEvent)>::new(move |event: MouseEvent| {
    //     let x = event.page_x();
    //     let y = event.page_y();
    //     *pick_position_cloned.borrow_mut() = Some((x, y));
    // });
    // window()
    //     .add_event_listener_with_callback("click", click.as_ref().unchecked_ref())
    //     .unwrap();
    // click.forget();

    let entity = Entity::new();
    entity.borrow_mut().set_geometry(Some(IndexedCube::new()));
    entity
        .borrow_mut()
        .set_material(Some(SolidColorInstancedMaterial::new(
            count, grid, width, height,
        )));
    // entity
    //     .borrow_mut()
    //     .set_material(Some(SolidColorMaterial::with_color(rand::random())));
    scene.entity_collection_mut().add_entity(entity);

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    *(*g).borrow_mut() = Some(Closure::new(move |frame_time: f64| {
        let seconds = frame_time / 1000.0;

        // static RADIANS_PER_SECOND: f64 = std::f64::consts::PI / 2.0;
        // let rotation = (seconds * RADIANS_PER_SECOND) % (2.0 * std::f64::consts::PI);

        // scene
        //     .entity_collection_mut()
        //     .set_local_matrix(Mat4::from_y_rotation(rotation));

        let start = window().performance().unwrap().now();
        render
            .render(&mut pipeline, &mut SceneStuff::new(&mut scene), frame_time)
            .unwrap();
        let end = window().performance().unwrap().now();
        document()
            .get_element_by_id("total")
            .unwrap()
            .set_inner_html(&format!("{:.2}", end - start));

        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

// #[wasm_bindgen]
// pub fn test_texture(
//     url: String,
//     count: usize,
//     grid: usize,
//     width: f64,
//     height: f64,
// ) -> Result<(), Error> {
//     let mut scene = create_scene((0.0, 20.0, 0.0), (0.0, 0.0, 0.0), (0.0, 0.0, -1.0))?;
//     let mut render = create_render(&scene)?;

//     let mut entity = Entity::new();

//     // entity.set_geometry(Some(Cube::new()));
//     entity.set_geometry(Some(IndexedCube::new()));
//     entity.set_material(Some(TextureInstancedMaterial::new(
//         url, count, grid, width, height,
//     )));
//     scene.root_entity_mut().add_child(entity);

//     let f = Rc::new(RefCell::new(None));
//     let g = f.clone();
//     *(*g).borrow_mut() = Some(Closure::new(move |timestamp: f64| {
//         let seconds = timestamp / 1000.0;

//         static MAX_SIZE: f64 = 3.0;
//         static MIN_SIZE: f64 = 1.0;
//         static SIZE_PER_SECOND: f64 = 0.5;
//         let size = (seconds * SIZE_PER_SECOND % (MAX_SIZE - MIN_SIZE)) + MIN_SIZE;
//         scene
//             .root_entity_mut()
//             .children_mut()
//             .get_mut(0)
//             .unwrap()
//             .geometry_mut()
//             .unwrap()
//             .as_any_mut()
//             .downcast_mut::<IndexedCube>()
//             .unwrap()
//             .set_size(size as f64);

//         static RADIANS_PER_SECOND: f64 = std::f64::consts::PI / 2.0;
//         let rotation = (seconds * RADIANS_PER_SECOND) % (2.0 * std::f64::consts::PI);

//         scene
//             .root_entity_mut()
//             .set_local_matrix(Mat4::from_y_rotation(rotation));
//         render.render(&mut scene, timestamp).unwrap();

//         request_animation_frame(f.borrow().as_ref().unwrap());
//     }));

//     request_animation_frame(g.borrow().as_ref().unwrap());

//     Ok(())
// }

// #[wasm_bindgen]
// pub fn test_environment(
//     px: String,
//     nx: String,
//     py: String,
//     ny: String,
//     pz: String,
//     nz: String,
// ) -> Result<(), Error> {
//     let mut scene = create_scene((2.0, 2.0, 2.0), (0.0, 0.0, 0.0), (0.0, 1.0, 0.0))?;
//     let mut render = create_render(&scene)?;

//     let mut entity = Entity::new();

//     entity.set_geometry(Some(Sphere::with_opts(1.0, 48, 96)));
//     entity.set_material(Some(EnvironmentMaterial::new(px, nx, py, ny, pz, nz)));
//     scene.root_entity_mut().add_child(entity);

//     let f = Rc::new(RefCell::new(None));
//     let g = f.clone();
//     let mut scaling = Vec3::from_values(1.0, 1.0, 1.0);
//     *(*g).borrow_mut() = Some(Closure::new(move |timestamp: f64| {
//         let seconds = timestamp / 1000.0;

//         static MAX_SIZE: f64 = 1.0;
//         static MIN_SIZE: f64 = 0.2;
//         static SIZE_PER_SECOND: f64 = 0.5;
//         let size = (seconds * SIZE_PER_SECOND % (MAX_SIZE - MIN_SIZE)) + MIN_SIZE;
//         scaling.0[0] = size;
//         scaling.0[1] = size;
//         scaling.0[2] = size;
//         scene
//             .root_entity_mut()
//             .children_mut()
//             .get_mut(0)
//             .unwrap()
//             .set_local_matrix(Mat4::from_scaling(&scaling));
//         // bad performance below
//         // scene
//         //     .root_entity_mut()
//         //     .children_mut()
//         //     .get(0)
//         //     .unwrap()
//         //     .geometry()
//         //     .unwrap()
//         //     .borrow_mut()
//         //     .as_any_mut()
//         //     .downcast_mut::<Sphere>()
//         //     .unwrap()
//         //     .set_radius(size as f32);

//         render.render(&mut scene, timestamp).unwrap();

//         request_animation_frame(f.borrow().as_ref().unwrap());
//     }));

//     request_animation_frame(g.borrow().as_ref().unwrap());

//     Ok(())
// }

// #[wasm_bindgen]
// pub fn test_drop_buffer_descriptor() -> Result<(), Error> {
//     let mut scene = create_scene((2.0, 2.0, 2.0), (0.0, 0.0, 0.0), (0.0, 1.0, 0.0))?;
//     let mut render = create_render(&scene)?;

//     let mut entity = Entity::new();
//     entity.set_geometry(Some(Sphere::with_opts(1.0, 48, 96)));
//     entity.set_material(Some(SolidColorMaterial::with_color(rand::random::<Rgb>())));
//     scene.root_entity_mut().add_child(entity);

//     let f = Rc::new(RefCell::new(None));
//     let g = f.clone();
//     *(*g).borrow_mut() = Some(Closure::new(move |timestamp: f64| {
//         if timestamp > 5.0 * 1000.0 {
//             let _ = scene.root_entity_mut().remove_child_by_index(0);
//             render.render(&mut scene, timestamp).unwrap();
//         } else {
//             render.render(&mut scene, timestamp).unwrap();
//             request_animation_frame(f.borrow().as_ref().unwrap());
//         }
//     }));

//     request_animation_frame(g.borrow().as_ref().unwrap());

//     Ok(())
// }

// #[wasm_bindgen]
// pub fn test_drop_buffer_descriptor2() -> Result<(), Error> {
//     let mut scene = create_scene((2.0, 2.0, 2.0), (0.0, 0.0, 0.0), (0.0, 1.0, 0.0))?;
//     let mut render = create_render(&scene)?;

//     let buffer = Uint8Array::new_with_length(1 * 1024 * 1024 * 1024);
//     buffer.fill(1, 0, buffer.byte_length());
//     let large_buffer = BufferDescriptor::new(
//         BufferSource::from_uint8_array(buffer, 0, 0),
//         BufferUsage::StaticDraw,
//     );
//     let buffer = Uint8Array::new_with_length(1 * 1024 * 1024 * 1024);
//     buffer.fill(1, 0, buffer.byte_length());
//     let large_buffer_1 = BufferDescriptor::new(
//         BufferSource::from_uint8_array(buffer, 0, 0),
//         BufferUsage::StaticDraw,
//     );
//     render
//         .buffer_store_mut()
//         .use_buffer(large_buffer.clone(), BufferTarget::ArrayBuffer)?;
//     render
//         .buffer_store_mut()
//         .use_buffer(large_buffer_1.clone(), BufferTarget::ArrayBuffer)?;

//     let mut entity = Entity::new();
//     entity.set_geometry(Some(Sphere::with_opts(1.0, 48, 96)));
//     entity.set_material(Some(SolidColorMaterial::with_color(rand::random::<Rgb>())));
//     scene.root_entity_mut().add_child(entity);

//     let f = Rc::new(RefCell::new(None));
//     let g = f.clone();
//     *(*g).borrow_mut() = Some(Closure::new(move |timestamp: f64| {
//         if timestamp <= 30.0 * 1000.0 {
//             render.render(&mut scene, timestamp).unwrap();
//             request_animation_frame(f.borrow().as_ref().unwrap());
//         } else {
//             scene.set_mount(None).unwrap();
//             console_log!("stop rendering");
//         }
//     }));

//     request_animation_frame(g.borrow().as_ref().unwrap());

//     let callback = Closure::once(move || {
//         drop(large_buffer);
//         drop(large_buffer_1);
//     });

//     window()
//         .set_timeout_with_callback_and_timeout_and_arguments_0(
//             callback.into_js_value().unchecked_ref(),
//             10 * 1000,
//         )
//         .unwrap();

//     Ok(())
// }

// #[wasm_bindgen]
// pub fn test_binary() {
//     let b0 = Uint8Array::new_with_length(1 * 1024 * 1024 * 1024);
//     b0.fill(1, 0, 1 * 1024 * 1024 * 1024);
//     let b1 = Uint8Array::new_with_length(1 * 1024 * 1024 * 1024);
//     b1.fill(1, 0, 1 * 1024 * 1024 * 1024);
//     let b2 = Uint8Array::new_with_length(1 * 1024 * 1024 * 1024);
//     b2.fill(1, 0, 1 * 1024 * 1024 * 1024);
//     let b3 = b0.clone();
//     let b4 = b0.clone();
//     let b5 = b0.clone();
//     let b6 = b0.clone();
//     let b7 = b0.clone();
//     let b8 = b0.clone();
//     let b9 = b0.clone();

//     let callback = Closure::once(|| {
//         drop(b0);
//         drop(b1);
//         drop(b2);
//         drop(b3);
//         drop(b4);
//         drop(b5);
//         drop(b6);
//         drop(b7);
//         drop(b8);
//         drop(b9);
//         console_log!("dropped")
//     });

//     window()
//         .set_timeout_with_callback_and_timeout_and_arguments_0(
//             callback.into_js_value().unchecked_ref(),
//             10 * 1000,
//         )
//         .unwrap();
// }

#[wasm_bindgen]
pub fn test_pick(count: usize, grid: usize, width: f64, height: f64) -> Result<(), Error> {
    let mut scene = create_scene((0.0, 3.0, 8.0), (0.0, 0.0, 0.0), (0.0, 1.0, 0.0));
    let render = create_render()?;
    let render = Rc::new(RefCell::new(render));
    let last_frame_time = Rc::new(RefCell::new(0.0));
    let mut picking_pipeline = create_picking_pipeline(
        ResourceKey::new_persist_str("position"),
        ResourceKey::new_persist_str("picked_entity"),
        ResourceKey::new_persist_str("picked_position"),
    );
    let standard_pipeline = create_standard_pipeline(
        ResourceKey::new_persist_str("position"),
        ResourceKey::new_persist_str("clear_color"),
    );
    let standard_pipeline = Rc::new(RefCell::new(standard_pipeline));

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

        let entity = Entity::new();

        entity.borrow_mut().set_geometry(Some(Cube::new()));
        // entity.set_geometry(Some(IndexedCube::new()));
        entity
            .borrow_mut()
            .set_material(Some(SolidColorMaterial::with_color(rand::random())));
        entity.borrow_mut().set_local_matrix(model_matrix);
        scene.entity_collection_mut().add_entity(entity);
    }
    let scene = Rc::new(RefCell::new(scene));

    let render_cloned = Rc::clone(&render);
    let scene_cloned = Rc::clone(&scene);
    let last_frame_time_cloned = Rc::clone(&last_frame_time);
    let click = Closure::<dyn FnMut(MouseEvent)>::new(move |event: MouseEvent| {
        let x = event.page_x();
        let y = event.page_y();

        let start = window().performance().unwrap().now();
        // sets pick position
        picking_pipeline
            .resources_mut()
            .insert(ResourceKey::new_persist_str("position"), (x, y));

        // pick
        render_cloned
            .borrow_mut()
            .render(
                &mut picking_pipeline,
                &mut SceneStuff::new(&mut scene_cloned.borrow_mut()),
                *last_frame_time_cloned.borrow(),
            )
            .unwrap();
        let end = window().performance().unwrap().now();
        document()
            .get_element_by_id("pick")
            .unwrap()
            .set_inner_html(&format!("{:.2}", end - start));

        // get entity
        if let Some(entity) = picking_pipeline
            .resources_mut()
            .get(&ResourceKey::<Weak>::new_persist_str("picked_entity"))
            .and_then(|e| e.upgrade())
        {
            console_log!("pick entity {}", entity.borrow().id());

            let mut material = entity.borrow_mut();
            let material = material.material_mut().unwrap();
            material
                .as_any_mut()
                .downcast_mut::<SolidColorMaterial>()
                .unwrap()
                .set_color(rand::random());
        }
        // get position
        if let Some(position) = picking_pipeline
            .resources_mut()
            .get(&ResourceKey::<Vec3>::new_persist_str("picked_position"))
        {
            console_log!("pick position {}", position);
        }
    });
    window()
        .add_event_listener_with_callback("click", click.as_ref().unchecked_ref())
        .unwrap();
    click.forget();

    let standard_pipeline_cloned = Rc::clone(&standard_pipeline);
    let click = Closure::<dyn FnMut(MouseEvent)>::new(move |event: MouseEvent| {
        standard_pipeline_cloned
            .borrow_mut()
            .resources_mut()
            .insert(
                ResourceKey::new_persist_str("position"),
                (event.page_x(), event.page_y()),
            );
    });
    window()
        .add_event_listener_with_callback("mousemove", click.as_ref().unchecked_ref())
        .unwrap();
    click.forget();

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    *(*g).borrow_mut() = Some(Closure::new(move |frame_time: f64| {
        let seconds = frame_time / 1000.0;

        static RADIANS_PER_SECOND: f64 = std::f64::consts::PI / 4.0;
        let rotation = (seconds * RADIANS_PER_SECOND) % (2.0 * std::f64::consts::PI);

        scene
            .borrow_mut()
            .entity_collection_mut()
            .set_local_matrix(Mat4::from_y_rotation(rotation));

        let start = window().performance().unwrap().now();
        let mut standard_pipeline = standard_pipeline.borrow_mut();
        let standard_pipeline = &mut *standard_pipeline;
        render
            .borrow_mut()
            .render(
                standard_pipeline,
                &mut SceneStuff::new(&mut scene.borrow_mut()),
                frame_time,
            )
            .unwrap();
        let end = window().performance().unwrap().now();
        document()
            .get_element_by_id("total")
            .unwrap()
            .set_inner_html(&format!("{:.2}", end - start));

        *last_frame_time.borrow_mut() = frame_time;

        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

#[wasm_bindgen]
pub fn test_camera() {
    let camera = PerspectiveCamera::new(
        (0.0, 0.0, 1.0),
        (0.0, 0.0, 0.0),
        (0.0, 1.0, 0.0),
        60.0f64.to_radians(),
        1080.0 / 1920.0,
        1.0,
        Some(2.0),
    );

    // let camera = PerspectiveCamera::new(
    //     (0.0, 1.0, 0.0),
    //     (0.0, 0.0, 0.0),
    //     (0.0, 0.0, -1.0),
    //     60.0f64.to_radians(),
    //     1080.0 / 1920.0,
    //     0.1,
    //     Some(2.0),
    // );
    let frustum = camera.view_frustum();
    console_log!(
        "near ({}), ({})",
        frustum.near().normal(),
        frustum.near().point_on_plane()
    );
    console_log!(
        "far ({:?}), ({:?})",
        frustum.far().map(|p| p.normal()),
        frustum.far().map(|p| p.point_on_plane())
    );
    console_log!(
        "top ({}), ({})",
        frustum.top().normal(),
        frustum.top().point_on_plane()
    );
    console_log!(
        "bottom ({}), ({})",
        frustum.bottom().normal(),
        frustum.bottom().point_on_plane()
    );
    console_log!(
        "left ({}), ({})",
        frustum.left().normal(),
        frustum.left().point_on_plane()
    );
    console_log!(
        "right ({}), ({})",
        frustum.right().normal(),
        frustum.right().point_on_plane()
    );

    let position = Vec4::from_values(0.0, 0.0, -1.0, 1.0);

    let view_matrix = camera.view_matrix();
    let view_translated_matrix = view_matrix.translate(&(0.0, 0.0, 2.0));
    let view_inv_matrix = view_matrix.invert().unwrap();
    let proj_matrix = camera.proj_matrix();
    let view_proj_matrix = camera.view_proj_matrix();

    console_log!("{}", position.transform_mat4(&view_matrix));
    console_log!("{}", position.transform_mat4(&view_translated_matrix));
    console_log!(
        "{}",
        Vec3::from_values(0.0, 0.0, 1.0).transform_mat4(&view_translated_matrix)
    );
    console_log!(
        "{}",
        Vec3::from_values(0.0, 0.0, 3.0).transform_mat4(&view_translated_matrix)
    );
    console_log!(
        "{}",
        Vec3::from_values(0.0, 0.0, 0.0).transform_mat4(&view_matrix)
    );
    console_log!(
        "{}",
        Vec3::from_values(0.0, 0.0, -1.0).transform_mat4(&view_inv_matrix)
    );
    console_log!(
        "{}",
        Vec3::from_values(0.0, 0.0, 1.0).transform_mat4(&view_inv_matrix)
    );
    console_log!("{}", Vec3::new().transform_mat4(&view_matrix));
    console_log!("{}", Vec3::new().transform_mat4(&view_inv_matrix));
    console_log!("{}", position.transform_mat4(&view_proj_matrix));
    console_log!(
        "{}",
        position.transform_mat4(&view_proj_matrix) / position.transform_mat4(&view_proj_matrix).w()
    );
}

// #[wasm_bindgen]
// pub fn test_simd() {
//         let vec1 = gl_matrix4rust::wasm32::simd128::f32::vec4::Vec4::new(1.0, 1.0, 1.0, 1.0);

//     let count = 1500000000usize;
//     let start = window().performance().unwrap().now();
//     let mut result = gl_matrix4rust::wasm32::simd128::f32::vec4::Vec4::new(1.0, 1.0, 1.0, 1.0);
//     for _ in 0..count {
//         result = result * vec1;
//     }
//     let end = window().performance().unwrap().now();

//     console_log!("simd costs {}ms", end - start);
// }

// #[wasm_bindgen]
// pub fn test_non_simd() {
//         let vec1 = Vec4::<f32>::from_values(1.0, 1.0, 1.0, 1.0);

//     let count = 1500000000usize;
//     let start = window().performance().unwrap().now();
//     let mut result = Vec4::<f32>::from_values(1.0, 1.0, 1.0, 1.0);
//     for _ in 0..count {
//         result = result * vec1;
//     }
//     let end = window().performance().unwrap().now();

//     console_log!("non simd costs {}ms", end - start);
// }
