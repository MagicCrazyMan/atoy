use std::any::Any;
use std::borrow::Cow;
use std::collections::HashMap;
use std::f64::consts::PI;
use std::ops::Mul;
use std::rc::Weak;
use std::time::Duration;
use std::{cell::RefCell, rc::Rc};

use gl_matrix4rust::mat4::Mat4;
use gl_matrix4rust::quat::Quat;
use gl_matrix4rust::vec2::Vec2;
use gl_matrix4rust::vec3::Vec3;
use gl_matrix4rust::vec4::Vec4;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;
use wasm_bindgen::{closure::Closure, JsCast};
use wasm_bindgen_test::console_log;
use web_sys::js_sys::{ArrayBuffer, Date, Function, Uint8Array};
use web_sys::{Element, HtmlImageElement, MouseEvent};

use crate::camera::orthogonal::OrthogonalCamera;
use crate::camera::perspective::PerspectiveCamera;
use crate::camera::universal::UniversalCamera;
use crate::camera::Camera;
use crate::clock::{Clock, Tick, WebClock};
use crate::entity::{Entity, Group, SimpleEntity, SimpleGroup};
use crate::error::Error;
use crate::geometry::indexed_cube::IndexedCube;
use crate::geometry::raw::RawGeometry;
use crate::geometry::rectangle::{Placement, Rectangle};
use crate::geometry::sphere::Sphere;
use crate::light::ambient_light::AmbientLight;
use crate::light::area_light::AreaLight;
use crate::light::attenuation::Attenuation;
use crate::light::directional_light::DirectionalLight;
use crate::light::point_light::PointLight;
use crate::light::spot_light::SpotLight;
use crate::loader::dds::{DirectDrawSurface, DirectDrawSurfaceLoader, DDS_DXT1, DDS_DXT3};
use crate::loader::texture::TextureLoader;
use crate::material::webgl::texture::TextureMaterial;
use crate::material::webgl::StandardMaterial;
use crate::material::{self, Transparency};
use crate::notify::Notifiee;
use crate::pipeline::webgl::{HdrToneMappingType, StandardPipelineShading};
use crate::readonly::Readonly;
use crate::renderer::webgl::attribute::AttributeValue;
use crate::renderer::webgl::buffer::{
    BufferComponentSize, BufferDataType, BufferDescriptor, BufferSource, BufferTarget, BufferUsage,
};
use crate::renderer::webgl::draw::{Draw, DrawMode};
use crate::renderer::webgl::program::{CustomBinding, Define};
use crate::renderer::webgl::state::FrameState;
use crate::renderer::webgl::texture::texture2d::Texture2D;
use crate::renderer::webgl::texture::{
    texture2d, SamplerParameter, TextureColorFormat, TextureCompressedFormat, TextureDataType,
    TextureDescriptor, TextureFormat, TextureMagnificationFilter, TextureMinificationFilter,
    TextureParameter, TexturePixelStorage, TexturePlanar, TextureSource, TextureSourceCompressed,
    TextureUnit, TextureWrapMethod,
};
use crate::renderer::webgl::uniform::{UniformBlockValue, UniformValue};
use crate::renderer::webgl::RenderEvent;
use crate::renderer::Renderer;
use crate::share::{Share, WeakShare};
use crate::utils::slice_to_float32_array;
use crate::viewer::{self, Viewer};
use crate::{document, entity};
use crate::{
    geometry::cube::Cube,
    material::webgl::solid_color::SolidColorMaterial,
    renderer::webgl::{draw::CullFace, WebGL2Renderer},
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

struct TickSolidColorMaterial(SolidColorMaterial);

impl TickSolidColorMaterial {
    /// Constructs a solid color material with specified color and transparency.
    pub fn with_color(
        color: Vec3<f32>,
        specular_shininess: f32,
        transparency: Transparency,
    ) -> Self {
        Self(SolidColorMaterial::with_color(
            color,
            specular_shininess,
            transparency,
        ))
    }
}

impl StandardMaterial for TickSolidColorMaterial {
    fn name(&self) -> Cow<'_, str> {
        self.0.name()
    }

    fn ready(&self) -> bool {
        self.0.ready()
    }

    fn prepare(&mut self, state: &mut FrameState) {
        self.0.prepare(state)
    }

    fn tick(&mut self, tick: &Tick) -> bool {
        self.0.set_color(
            Vec3::new(rand::random(), rand::random(), rand::random()),
            Transparency::Opaque,
        );
        self.0.tick(tick)
    }

    fn transparency(&self) -> Transparency {
        self.0.transparency()
    }

    fn attribute_value(&self, name: &str) -> Option<Readonly<'_, AttributeValue>> {
        self.0.attribute_value(name)
    }

    fn uniform_value(&self, name: &str) -> Option<Readonly<'_, UniformValue>> {
        self.0.uniform_value(name)
    }

    fn uniform_block_value(&self, name: &str) -> Option<Readonly<'_, UniformBlockValue>> {
        self.0.uniform_block_value(name)
    }

    fn fragment_process(&self) -> Cow<'_, str> {
        self.0.fragment_process()
    }

    fn vertex_defines(&self) -> &[Define<'_>] {
        self.0.vertex_defines()
    }

    fn snippet(&self, name: &str) -> Option<Cow<'_, str>> {
        self.0.snippet(name)
    }

    fn attribute_custom_bindings(&self) -> &[CustomBinding<'_>] {
        self.0.attribute_custom_bindings()
    }

    fn uniform_custom_bindings(&self) -> &[CustomBinding<'_>] {
        self.0.uniform_custom_bindings()
    }

    fn uniform_block_custom_bindings(&self) -> &[CustomBinding<'_>] {
        self.0.uniform_block_custom_bindings()
    }

    fn use_position_eye_space(&self) -> bool {
        self.0.use_position_eye_space()
    }

    fn use_normal(&self) -> bool {
        self.0.use_normal()
    }

    fn use_texture_coordinate(&self) -> bool {
        self.0.use_texture_coordinate()
    }

    fn use_tbn(&self) -> bool {
        self.0.use_tbn()
    }

    fn use_calculated_bitangent(&self) -> bool {
        self.0.use_calculated_bitangent()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
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

fn create_camera(camera_position: Vec3, camera_center: Vec3, camera_up: Vec3) -> UniversalCamera {
    UniversalCamera::new(
        camera_position,
        camera_center,
        camera_up,
        75.0f64.to_radians(),
        1.0,
        0.1,
        Some(1000.0),
    )
}

fn create_scene() -> Result<Scene<WebClock>, Error> {
    let mut scene = Scene::new()?;
    scene.clock_mut().start(Duration::from_secs(1));
    scene.set_light_attenuations(Attenuation::new(0.0, 1.0, 0.0));
    // scene.set_ambient_light(Some(AmbientLight::new(Vec3::new(0,0,0))));
    // scene.add_directional_light(DirectionalLight::new(
    //     Vec3::new(0.0, -1.0, -1.0),
    //     Vec3::new(0,0,0),
    //     Vec3::new(0.19, 0.19, 0.19),
    //     Vec3::new(0.8, 0.8, 0.8),
    //     128.0,
    // ));
    scene.add_spot_light(SpotLight::new(
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(1.0, -1.0, -1.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.4, 0.4, 0.4),
        Vec3::new(0.6, 0.6, 0.6),
        30f32.to_radians(),
        40f32.to_radians(),
    ));
    scene.add_spot_light(SpotLight::new(
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, -1.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.4, 0.4, 0.4),
        Vec3::new(0.6, 0.6, 0.6),
        30f32.to_radians(),
        60f32.to_radians(),
    ));
    scene.add_area_light(AreaLight::new(
        Vec3::new(-3.0, 2.0, 0.0),
        Vec3::new(-1.0, -1.0, 1.0),
        Vec3::new(1.0, 0.0, -1.0),
        0.5,
        4.0,
        1.5,
        4.5,
        2.0,
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.4, 0.4, 0.4),
        Vec3::new(0.6, 0.6, 0.6),
    ));
    let light_pos = Vec3::new(0.0, 0.5, 0.0);
    scene.add_point_light(PointLight::new(
        light_pos.clone(),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.4, 0.4, 0.4),
        Vec3::new(0.6, 0.6, 0.6),
    ));
    let mut hint = SimpleEntity::new();
    hint.set_model_matrix(Mat4::<f64>::from_translation(&light_pos));
    hint.set_geometry(Some(Sphere::with_params(0.1, 12, 24)));
    hint.set_material(Some(SolidColorMaterial::with_color(
        Vec3::new(1.0, 1.0, 1.0),
        128.0,
        Transparency::Opaque,
    )));
    scene.entity_group().borrow_mut().add_entity(hint);
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(1.0, 1.5, 0.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(1.0, 1.5, 1.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(0.0, 1.5, 1.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(-1.0, 1.5, 1.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(-1.0, 1.5, 0.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(-1.0, 1.5, -1.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(0.0, 1.5, -1.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(1.0, 1.5, -1.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(0.0, 1.5, 2.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(0.0, 1.5, 3.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(2.0, 1.5, 0.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(2.0, 1.5, 1.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(2.0, 1.5, 2.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(1.0, 1.5, 2.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(0.0, 1.5, 2.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(-1.0, 1.5, 2.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(-2.0, 1.5, 2.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(-2.0, 1.5, 1.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(-2.0, 1.5, 0.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(-2.0, 1.5, -1.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(-2.0, 1.5, -2.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(-2.0, 1.5, -1.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(-2.0, 1.5, 0.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(-2.0, 1.5, 1.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(-2.0, 1.5, 2.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(0.0, 1.5, 3.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(0.0, 1.5, 3.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(0.0, 1.5, 4.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(0.0, 1.5, 5.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(0.0, 1.5, 6.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(8.0, 0.5, 0.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.69, 0.69, 0.69),
    //     Vec3::new(0.3, 0.3, 0.3),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(2.0, 1.5, 0.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(2.0, 1.5, 1.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(2.0, 1.5, 2.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(1.0, 1.5, 2.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(0.0, 1.5, 2.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(-1.0, 1.5, 2.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    // scene.add_point_light(PointLight::new(
    //     Vec3::new(-2.0, 1.5, 2.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.4, 0.4, 0.4),
    //     Vec3::new(0.6, 0.6, 0.6),
    // ));
    Ok(scene)
}

fn create_viewer(
    scene: Scene<WebClock>,
    camera: UniversalCamera,
    render_callback: &Function,
) -> Viewer {
    let mut viewer = Viewer::new(scene, camera.clone()).unwrap();
    viewer.add_controller(camera);
    viewer
        .set_mount(document().get_element_by_id("scene"))
        .unwrap();

    struct PreRenderNotifiee(Share<f64>);
    impl Notifiee<RenderEvent> for PreRenderNotifiee {
        fn notify(&mut self, msg: &RenderEvent) {
            *self.0.borrow_mut() = crate::window().performance().unwrap().now();
        }
    }

    struct PostRenderNotifiee(Share<f64>, Function);
    impl Notifiee<RenderEvent> for PostRenderNotifiee {
        fn notify(&mut self, msg: &RenderEvent) {
            let start = *self.0.borrow();
            let end = crate::window().performance().unwrap().now();
            self.1
                .call1(&JsValue::null(), &JsValue::from_f64(end - start))
                .unwrap();
        }
    }

    let start_timestamp = Rc::new(RefCell::new(0.0));
    viewer
        .renderer()
        .borrow_mut()
        .pre_render()
        .register(PreRenderNotifiee(Rc::clone(&start_timestamp)));
    viewer
        .renderer()
        .borrow_mut()
        .post_render()
        .register(PostRenderNotifiee(
            Rc::clone(&start_timestamp),
            render_callback.clone(),
        ));

    viewer
}

#[wasm_bindgen]
pub struct ViewerWasm(Share<Viewer>);

#[wasm_bindgen]
impl ViewerWasm {
    pub fn mount_wasm(&self) -> Option<Element> {
        self.0.borrow().mount().cloned()
    }

    pub fn set_mount_wasm(&mut self, mount: Option<Element>) -> Result<(), Error> {
        self.0.borrow_mut().set_mount(mount)
    }

    pub fn render_when_needed_wasm(&self) -> bool {
        // self.render_when_needed()
        true
    }

    pub fn enable_render_when_needed_wasm(&mut self) {
        // self.enable_render_when_needed()
    }

    pub fn disable_render_when_needed_wasm(&mut self) {
        // self.disable_render_when_needed()
    }

    /// Returns `true` if entity culling enabled.
    pub fn culling_enabled_wasm(&self) -> bool {
        self.0.borrow_mut().culling_enabled()
    }

    pub fn enable_culling_wasm(&mut self) {
        self.0.borrow_mut().enable_culling()
    }

    pub fn disable_culling_wasm(&mut self) {
        self.0.borrow_mut().disable_culling()
    }

    /// Returns `true` if entity distance sorting enabled.
    pub fn distance_sorting_enabled_wasm(&self) -> bool {
        self.0.borrow().distance_sorting_enabled()
    }

    pub fn enable_distance_sorting_wasm(&mut self) {
        self.0.borrow_mut().enable_distance_sorting()
    }

    pub fn disable_distance_sorting_wasm(&mut self) {
        self.0.borrow_mut().disable_distance_sorting()
    }

    pub fn pipeline_shading_wasm(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.0.borrow().pipeline_shading()).unwrap()
    }

    pub fn set_pipeline_shading_wasm(&mut self, shading: JsValue) {
        let shading = serde_wasm_bindgen::from_value::<StandardPipelineShading>(shading).unwrap();
        self.0.borrow_mut().set_pipeline_shading(shading);
    }

    pub fn clear_color_wasm(&self) -> Box<[f32]> {
        Box::new(self.0.borrow().clear_color().raw().clone())
    }

    pub fn set_clear_color_wasm(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.0.borrow_mut().set_clear_color(Vec4::new(r, g, b, a))
    }

    pub fn gamma_correction_enabled_wasm(&self) -> bool {
        self.0.borrow().gamma_correction_enabled()
    }

    pub fn enable_gamma_correction_wasm(&mut self) {
        self.0.borrow_mut().enable_gamma_correction();
    }

    pub fn disable_gamma_correction_wasm(&mut self) {
        self.0.borrow_mut().disable_gamma_correction();
    }

    pub fn gamma_wasm(&self) -> f32 {
        self.0.borrow().gamma()
    }

    pub fn set_gamma_wasm(&mut self, gamma: f32) {
        self.0.borrow_mut().set_gamma(gamma);
    }

    pub fn lighting_enabled_wasm(&self) -> bool {
        self.0.borrow().lighting_enabled()
    }

    pub fn enable_lighting_wasm(&mut self) {
        self.0.borrow_mut().enable_lighting();
    }

    pub fn disable_lighting_wasm(&mut self) {
        self.0.borrow_mut().disable_lighting();
    }

    pub fn multisamples_wasm(&self) -> Option<i32> {
        self.0.borrow().multisamples()
    }

    pub fn set_multisamples_wasm(&mut self, samples: Option<i32>) {
        self.0.borrow_mut().set_multisamples(samples)
    }

    pub fn hdr_enabled_wasm(&self) -> bool {
        self.0.borrow().hdr_enabled()
    }

    pub fn enable_hdr_wasm(&mut self) {
        self.0.borrow_mut().enable_hdr();
    }

    pub fn disable_hdr_wasm(&mut self) {
        self.0.borrow_mut().disable_hdr();
    }

    pub fn hdr_tone_mapping_type_wasm(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.0.borrow().hdr_tone_mapping_type()).unwrap()
    }

    pub fn set_hdr_tone_mapping_type_wasm(&mut self, hdr_tone_mapping_type: JsValue) {
        let t =
            serde_wasm_bindgen::from_value::<HdrToneMappingType>(hdr_tone_mapping_type).unwrap();
        self.0.borrow_mut().set_hdr_tone_mapping_type(t);
    }

    pub fn bloom_enabled_wasm(&self) -> bool {
        self.0.borrow().bloom_enabled()
    }

    pub fn enable_bloom_wasm(&mut self) {
        self.0.borrow_mut().enable_bloom()
    }

    pub fn disable_bloom_wasm(&mut self) {
        self.0.borrow_mut().disable_bloom()
    }

    pub fn bloom_blur_epoch_wasm(&self) -> usize {
        self.0.borrow().bloom_blur_epoch()
    }

    pub fn set_bloom_blur_epoch_wasm(&mut self, epoch: usize) {
        self.0.borrow_mut().set_bloom_blur_epoch(epoch);
    }
}

struct ViewerPicker {
    viewer: WeakShare<Viewer>,
    pick_callback: Function,
}

impl Notifiee<MouseEvent> for ViewerPicker {
    fn notify(&mut self, event: &MouseEvent) {
        let Some(viewer) = self.viewer.upgrade() else {
            return;
        };
        let mut viewer = viewer.borrow_mut();

        let x = event.page_x();
        let y = event.page_y();

        let start = window().performance().unwrap().now();

        // pick entity
        if let Some(entity) = viewer.pick_entity(x, y).unwrap() {
            let mut entity = entity.borrow_mut();
            if let Some(material) = entity
                .material_mut()
                .and_then(|material| material.as_any_mut().downcast_mut::<SolidColorMaterial>())
            {
                material.set_color(
                    Vec3::new(rand::random(), rand::random(), rand::random()),
                    Transparency::Opaque,
                );
            }
            if let Some(geometry) = entity
                .geometry_mut()
                .and_then(|geometry| geometry.as_any_mut().downcast_mut::<Cube>())
            {
                geometry.set_size(rand::random::<f64>() + 0.5 * 3.0);
            }
            entity.set_resync();
            console_log!("pick entity {}", entity.id());
        };

        // pick position
        if let Some(position) = viewer.pick_position(x, y).unwrap() {
            console_log!("pick position {}", position);
        };

        let end = window().performance().unwrap().now();
        self.pick_callback
            .call1(&JsValue::null(), &JsValue::from_f64(end - start))
            .unwrap();
    }
}

#[wasm_bindgen]
pub fn test_cube(
    count: usize,
    grid: usize,
    width: f64,
    height: f64,
    render_callback: &Function,
    pick_callback: &Function,
) -> Result<ViewerWasm, Error> {
    let camera = create_camera(
        Vec3::new(0.0, 5.0, 2.0),
        // Vec3::new(0.0, 0.0, 3.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );
    let scene = create_scene()?;

    let cell_width = width / (grid as f64);
    let cell_height = height / (grid as f64);
    let start_x = width / 2.0 - cell_width / 2.0;
    let start_z = height / 2.0 - cell_height / 2.0;
    let cubes = SimpleGroup::new();
    for index in 0..count {
        let row = index / grid;
        let col = index % grid;

        let center_x = start_x - col as f64 * cell_width;
        let center_z = start_z - row as f64 * cell_height;
        let model_matrix = Mat4::<f64>::from_translation(&Vec3::new(center_x, 0.0, center_z));

        let mut cube = SimpleEntity::new();
        cube.set_geometry(Some(Cube::new()));
        cube.set_material(Some(TickSolidColorMaterial::with_color(
            Vec3::new(rand::random(), rand::random(), rand::random()),
            128.0,
            Transparency::Opaque,
        )));
        cube.set_model_matrix(model_matrix);
        cubes.borrow_mut().add_entity(cube);
    }
    scene
        .entity_group()
        .borrow_mut()
        .add_sub_group_shared(cubes);

    // let entity = Entity::new();
    // entity.borrow_mut().set_geometry(Some(Rectangle::new(
    //     Vec2::new(0.0, 0.0),
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

    let mut images = SimpleGroup::new();
    [
        (
            "/images/sky.jpg",
            false,
            vec![TexturePixelStorage::UNPACK_FLIP_Y_WEBGL(true)],
            vec![] as Vec<SamplerParameter>,
            vec![] as Vec<TextureParameter>,
            true,
            false,
            false,
            false,
        ),
        (
            "/images/sky_dxt1.dds",
            true,
            vec![],
            vec![],
            vec![],
            false,
            true,
            false,
            false,
        ),
        (
            "/images/sky_dxt1_mipmaps.dds",
            true,
            vec![],
            vec![],
            vec![],
            false,
            true,
            false,
            true,
        ),
        (
            "/images/sky_dxt1_srgb_mipmaps.dds",
            true,
            vec![],
            vec![],
            vec![],
            false,
            true,
            true,
            true,
        ),
        (
            "/images/sky_dxt3.dds",
            true,
            vec![],
            vec![],
            vec![],
            false,
            false,
            false,
            false,
        ),
        (
            "/images/sky_dxt3_mipmaps.dds",
            true,
            vec![],
            vec![],
            vec![],
            false,
            false,
            false,
            true,
        ),
        (
            "/images/sky_dxt3_srgb_mipmaps.dds",
            true,
            vec![],
            vec![],
            vec![],
            false,
            false,
            true,
            true,
        ),
        (
            "/images/sky_dxt5.dds",
            true,
            vec![],
            vec![],
            vec![],
            false,
            false,
            false,
            false,
        ),
        (
            "/images/sky_dxt5_mipmaps.dds",
            true,
            vec![],
            vec![],
            vec![],
            false,
            false,
            false,
            true,
        ),
        (
            "/images/sky_dxt5_srgb_mipmaps.dds",
            true,
            vec![],
            vec![],
            vec![],
            false,
            false,
            true,
            true,
        ),
        (
            "/images/sky_dxt5_srgb_mipmaps.dds",
            true,
            vec![],
            vec![],
            vec![],
            false,
            false,
            true,
            false,
        ),
    ]
    .iter()
    .enumerate()
    .for_each(
        |(
            index,
            (
                url,
                is_compressed,
                pixel_storages,
                sampler_parameters,
                texture_parameters,
                generate_mipmap,
                dxt1_use_alpha,
                use_srgb,
                read_mipmaps,
            ),
        )| {
            let mut image = SimpleEntity::new();
            image.set_model_matrix(Mat4::<f64>::from_rotation_translation_scale(
                &Quat::<f64>::new_identity(),
                &Vec3::<f64>::new(0.0, 0.0, -(index as f64 + 1.0)),
                &Vec3::<f64>::new(4.0, 4.0, 4.0),
            ));
            image.set_geometry(Some(Rectangle::new(
                Vec2::new(0.0, 0.0),
                Placement::Center,
                0.25,
                0.25,
                1.0,
                1.0,
            )));

            if *is_compressed {
                image.set_material(Some(
                    material::webgl::texture::Builder::new(DirectDrawSurfaceLoader::with_params(
                        *url,
                        *dxt1_use_alpha,
                        *use_srgb,
                        *read_mipmaps,
                        sampler_parameters.clone(),
                        texture_parameters.clone(),
                    ))
                    .build(),
                ));
            } else {
                image.set_material(Some(
                    material::webgl::texture::Builder::new(TextureLoader::with_params(
                        *url,
                        pixel_storages.clone(),
                        sampler_parameters.clone(),
                        texture_parameters.clone(),
                        *generate_mipmap,
                        true,
                    ))
                    .build(),
                ));
            }

            images.borrow_mut().add_entity(image);
        },
    );

    let mut brick_wall_1 = SimpleEntity::new();
    brick_wall_1.set_model_matrix(Mat4::<f64>::from_rotation_translation(
        &Quat::<f64>::from_rotation_to(
            &Vec3::<f64>::new(0.0, 0.0, 1.0),
            &Vec3::<f64>::new(-1.0, 0.0, 1.0).normalize(),
        ),
        &Vec3::<f64>::new(1.0, 0.5, 0.0),
    ));
    brick_wall_1.set_geometry(Some(Rectangle::new(
        Vec2::<f64>::new_zero(),
        Placement::Center,
        2.0,
        2.0,
        1.0,
        1.0,
    )));
    brick_wall_1.set_material(Some(
        material::webgl::texture::Builder::new(TextureLoader::with_params(
            "/images/bricks2.jpg",
            [TexturePixelStorage::UNPACK_FLIP_Y_WEBGL(true)],
            [],
            [],
            true,
            true,
        ))
        .set_normal_map(TextureLoader::with_params(
            "/images/bricks2_normal.jpg",
            [TexturePixelStorage::UNPACK_FLIP_Y_WEBGL(true)],
            [],
            [],
            true,
            false,
        ))
        .build(),
    ));
    images.borrow_mut().add_entity(brick_wall_1);

    let mut brick_wall_2 = SimpleEntity::new();
    brick_wall_2.set_model_matrix(Mat4::<f64>::from_rotation_translation(
        &Quat::<f64>::from_rotation_to(
            &Vec3::<f64>::new(0.0, 0.0, 1.0),
            &Vec3::<f64>::new(1.0, 0.0, 1.0).normalize(),
        ),
        &Vec3::<f64>::new(-1.0, 0.5, 0.0),
    ));
    brick_wall_2.set_geometry(Some(Rectangle::new(
        Vec2::<f64>::new_zero(),
        Placement::Center,
        2.0,
        2.0,
        1.0,
        1.0,
    )));
    brick_wall_2.set_material(Some(
        material::webgl::texture::Builder::new(TextureLoader::with_params(
            "/images/brickwall.jpg",
            [TexturePixelStorage::UNPACK_FLIP_Y_WEBGL(true)],
            [],
            [],
            true,
            true,
        ))
        .set_normal_map(TextureLoader::with_params(
            "/images/brickwall_normal.jpg",
            [TexturePixelStorage::UNPACK_FLIP_Y_WEBGL(true)],
            [],
            [],
            true,
            false,
        ))
        .build(),
    ));
    images.borrow_mut().add_entity(brick_wall_2);

    let mut brick_wall_parallax = SimpleEntity::new();
    brick_wall_parallax.set_model_matrix(Mat4::<f64>::from_rotation_translation(
        &Quat::<f64>::new_zero(),
        &Vec3::<f64>::new(0.0, 0.5, -1.5),
    ));
    brick_wall_parallax.set_geometry(Some(Rectangle::new(
        Vec2::<f64>::new_zero(),
        Placement::Center,
        2.0,
        2.0,
        1.0,
        1.0,
    )));
    brick_wall_parallax.set_material(Some(
        material::webgl::texture::Builder::new(TextureLoader::with_params(
            "/images/bricks2.jpg",
            [TexturePixelStorage::UNPACK_FLIP_Y_WEBGL(true)],
            [],
            [],
            true,
            true,
        ))
        .set_normal_map(TextureLoader::with_params(
            "/images/bricks2_normal.jpg",
            [TexturePixelStorage::UNPACK_FLIP_Y_WEBGL(true)],
            [],
            [],
            true,
            false,
        ))
        .set_parallax_loader(TextureLoader::with_params(
            "/images/bricks2_disp.jpg",
            vec![TexturePixelStorage::UNPACK_FLIP_Y_WEBGL(true)],
            [],
            [],
            true,
            false,
        ))
        .build(),
    ));
    images.borrow_mut().add_entity(brick_wall_parallax);

    let mut floor = SimpleEntity::new();
    floor.set_material(Some(TextureMaterial::new(
        DirectDrawSurfaceLoader::new("/images/wood_dxt3_mipmaps.dds"),
        Transparency::Opaque,
    )));
    floor.set_geometry(Some(Rectangle::new(
        Vec2::new(0.0, 0.0),
        Placement::Center,
        1000.0,
        1000.0,
        200.0,
        200.0,
    )));
    floor.set_model_matrix(Mat4::<f64>::from_rotation_translation(
        &Quat::<f64>::from_axis_angle(&Vec3::new(-1.0, 0.0, 0.0), PI / 2.0),
        &Vec3::new(0.0, -0.6, 0.0),
    ));
    scene.entity_group().borrow_mut().add_entity(floor);

    scene
        .entity_group()
        .borrow_mut()
        .add_sub_group_shared(images);

    let viewer = create_viewer(scene, camera, render_callback);
    let viewer = Rc::new(RefCell::new(viewer));
    viewer
        .borrow()
        .scene()
        .borrow_mut()
        .canvas_handler()
        .click()
        .borrow_mut()
        .register(ViewerPicker {
            viewer: Rc::downgrade(&viewer),
            pick_callback: pick_callback.clone(),
        });

    viewer.borrow_mut().start_render_loop();

    Ok(ViewerWasm(viewer))
}

// // #[wasm_bindgen]
// // pub fn test_instanced_cube(
// //     count: usize,
// //     grid: usize,
// //     width: f64,
// //     height: f64,
// // ) -> Result<(), Error> {
// //     let mut scene = create_scene((0.0, 5.0, 5.0), (0.0, 0.0, 0.0), (0.0, 1.0, 0.0));
// //     let mut render = create_render()?;
// //     let mut pipeline = create_standard_pipeline(
// //         ResourceKey::new_persist_str("position"),
// //         ResourceKey::new_persist_str("clear_color"),
// //     );

// //     // let pick_position = Rc::new(RefCell::new(None as Option<(i32, i32)>));
// //     // let pick_position_cloned = Rc::clone(&pick_position);
// //     // let click = Closure::<dyn FnMut(MouseEvent)>::new(move |event: MouseEvent| {
// //     //     let x = event.page_x();
// //     //     let y = event.page_y();
// //     //     *pick_position_cloned.borrow_mut() = Some((x, y));
// //     // });
// //     // window()
// //     //     .add_event_listener_with_callback("click", click.as_ref().unchecked_ref())
// //     //     .unwrap();
// //     // click.forget();

// //     let entity = Entity::new();
// //     entity.borrow_mut().set_geometry(Some(IndexedCube::new()));
// //     entity
// //         .borrow_mut()
// //         .set_material(Some(SolidColorInstancedMaterial::new(
// //             count, grid, width, height,
// //         )));
// //     scene.entity_collection_mut().add_entity(entity);

// //     let f = Rc::new(RefCell::new(None));
// //     let g = f.clone();
// //     *(*g).borrow_mut() = Some(Closure::new(move |frame_time: f64| {
// //         let seconds = frame_time / 1000.0;

// //         // static RADIANS_PER_SECOND: f64 = std::f64::consts::PI / 2.0;
// //         // let rotation = (seconds * RADIANS_PER_SECOND) % (2.0 * std::f64::consts::PI);

// //         // scene
// //         //     .entity_collection_mut()
// //         //     .set_local_matrix(Mat4::from_y_rotation(rotation));

// //         let start = window().performance().unwrap().now();
// //         render
// //             .render(&mut pipeline, &mut scene.stuff(), frame_time)
// //             .unwrap();
// //         let end = window().performance().unwrap().now();
// //         document()
// //             .get_element_by_id("total")
// //             .unwrap()
// //             .set_inner_html(&format!("{:.2}", end - start));

// //         request_animation_frame(f.borrow().as_ref().unwrap());
// //     }));

// //     request_animation_frame(g.borrow().as_ref().unwrap());

// //     Ok(())
// // }

// // // #[wasm_bindgen]
// // // pub fn test_texture(
// // //     url: String,
// // //     count: usize,
// // //     grid: usize,
// // //     width: f64,
// // //     height: f64,
// // // ) -> Result<(), Error> {
// // //     let mut scene = create_scene((0.0, 20.0, 0.0), (0.0, 0.0, 0.0), (0.0, 0.0, -1.0))?;
// // //     let mut render = create_render(&scene)?;

// // //     let mut entity = Entity::new();

// // //     // entity.set_geometry(Some(Cube::new()));
// // //     entity.set_geometry(Some(IndexedCube::new()));
// // //     entity.set_material(Some(TextureInstancedMaterial::new(
// // //         url, count, grid, width, height,
// // //     )));
// // //     scene.root_entity_mut().add_child(entity);

// // //     let f = Rc::new(RefCell::new(None));
// // //     let g = f.clone();
// // //     *(*g).borrow_mut() = Some(Closure::new(move |timestamp: f64| {
// // //         let seconds = timestamp / 1000.0;

// // //         static MAX_SIZE: f64 = 3.0;
// // //         static MIN_SIZE: f64 = 1.0;
// // //         static SIZE_PER_SECOND: f64 = 0.5;
// // //         let size = (seconds * SIZE_PER_SECOND % (MAX_SIZE - MIN_SIZE)) + MIN_SIZE;
// // //         scene
// // //             .root_entity_mut()
// // //             .children_mut()
// // //             .get_mut(0)
// // //             .unwrap()
// // //             .geometry_mut()
// // //             .unwrap()
// // //             .as_any_mut()
// // //             .downcast_mut::<IndexedCube>()
// // //             .unwrap()
// // //             .set_size(size as f64);

// // //         static RADIANS_PER_SECOND: f64 = std::f64::consts::PI / 2.0;
// // //         let rotation = (seconds * RADIANS_PER_SECOND) % (2.0 * std::f64::consts::PI);

// // //         scene
// // //             .root_entity_mut()
// // //             .set_local_matrix(Mat4::from_y_rotation(rotation));
// // //         render.render(&mut scene, timestamp).unwrap();

// // //         request_animation_frame(f.borrow().as_ref().unwrap());
// // //     }));

// // //     request_animation_frame(g.borrow().as_ref().unwrap());

// // //     Ok(())
// // // }

// // // #[wasm_bindgen]
// // // pub fn test_environment(
// // //     px: String,
// // //     nx: String,
// // //     py: String,
// // //     ny: String,
// // //     pz: String,
// // //     nz: String,
// // // ) -> Result<(), Error> {
// // //     let mut scene = create_scene((2.0, 2.0, 2.0), (0.0, 0.0, 0.0), (0.0, 1.0, 0.0))?;
// // //     let mut render = create_render(&scene)?;

// // //     let mut entity = Entity::new();

// // //     entity.set_geometry(Some(Sphere::with_opts(1.0, 48, 96)));
// // //     entity.set_material(Some(EnvironmentMaterial::new(px, nx, py, ny, pz, nz)));
// // //     scene.root_entity_mut().add_child(entity);

// // //     let f = Rc::new(RefCell::new(None));
// // //     let g = f.clone();
// // //     let mut scaling = Vec3::new(1.0, 1.0, 1.0);
// // //     *(*g).borrow_mut() = Some(Closure::new(move |timestamp: f64| {
// // //         let seconds = timestamp / 1000.0;

// // //         static MAX_SIZE: f64 = 1.0;
// // //         static MIN_SIZE: f64 = 0.2;
// // //         static SIZE_PER_SECOND: f64 = 0.5;
// // //         let size = (seconds * SIZE_PER_SECOND % (MAX_SIZE - MIN_SIZE)) + MIN_SIZE;
// // //         scaling.0[0] = size;
// // //         scaling.0[1] = size;
// // //         scaling.0[2] = size;
// // //         scene
// // //             .root_entity_mut()
// // //             .children_mut()
// // //             .get_mut(0)
// // //             .unwrap()
// // //             .set_local_matrix(Mat4::from_scaling(&scaling));
// // //         // bad performance below
// // //         // scene
// // //         //     .root_entity_mut()
// // //         //     .children_mut()
// // //         //     .get(0)
// // //         //     .unwrap()
// // //         //     .geometry()
// // //         //     .unwrap()
// // //         //     .borrow_mut()
// // //         //     .as_any_mut()
// // //         //     .downcast_mut::<Sphere>()
// // //         //     .unwrap()
// // //         //     .set_radius(size as f32);

// // //         render.render(&mut scene, timestamp).unwrap();

// // //         request_animation_frame(f.borrow().as_ref().unwrap());
// // //     }));

// // //     request_animation_frame(g.borrow().as_ref().unwrap());

// // //     Ok(())
// // // }

// // // #[wasm_bindgen]
// // // pub fn test_drop_buffer_descriptor() -> Result<(), Error> {
// // //     let mut scene = create_scene((2.0, 2.0, 2.0), (0.0, 0.0, 0.0), (0.0, 1.0, 0.0))?;
// // //     let mut render = create_render(&scene)?;

// // //     let mut entity = Entity::new();
// // //     entity.set_geometry(Some(Sphere::with_opts(1.0, 48, 96)));
// // //     entity.set_material(Some(SolidColorMaterial::with_color(rand::random::<Rgb>())));
// // //     scene.root_entity_mut().add_child(entity);

// // //     let f = Rc::new(RefCell::new(None));
// // //     let g = f.clone();
// // //     *(*g).borrow_mut() = Some(Closure::new(move |timestamp: f64| {
// // //         if timestamp > 5.0 * 1000.0 {
// // //             let _ = scene.root_entity_mut().remove_child_by_index(0);
// // //             render.render(&mut scene, timestamp).unwrap();
// // //         } else {
// // //             render.render(&mut scene, timestamp).unwrap();
// // //             request_animation_frame(f.borrow().as_ref().unwrap());
// // //         }
// // //     }));

// // //     request_animation_frame(g.borrow().as_ref().unwrap());

// // //     Ok(())
// // // }

// // // #[wasm_bindgen]
// // // pub fn test_drop_buffer_descriptor2() -> Result<(), Error> {
// // //     let mut scene = create_scene((2.0, 2.0, 2.0), (0.0, 0.0, 0.0), (0.0, 1.0, 0.0))?;
// // //     let mut render = create_render(&scene)?;

// // //     let buffer = Uint8Array::new_with_length(1 * 1024 * 1024 * 1024);
// // //     buffer.fill(1, 0, buffer.byte_length());
// // //     let large_buffer = BufferDescriptor::new(
// // //         BufferSource::from_uint8_array(buffer, 0, 0),
// // //         BufferUsage::StaticDraw,
// // //     );
// // //     let buffer = Uint8Array::new_with_length(1 * 1024 * 1024 * 1024);
// // //     buffer.fill(1, 0, buffer.byte_length());
// // //     let large_buffer_1 = BufferDescriptor::new(
// // //         BufferSource::from_uint8_array(buffer, 0, 0),
// // //         BufferUsage::StaticDraw,
// // //     );
// // //     render
// // //         .buffer_store_mut()
// // //         .use_buffer(large_buffer.clone(), BufferTarget::ArrayBuffer)?;
// // //     render
// // //         .buffer_store_mut()
// // //         .use_buffer(large_buffer_1.clone(), BufferTarget::ArrayBuffer)?;

// // //     let mut entity = Entity::new();
// // //     entity.set_geometry(Some(Sphere::with_opts(1.0, 48, 96)));
// // //     entity.set_material(Some(SolidColorMaterial::with_color(rand::random::<Rgb>())));
// // //     scene.root_entity_mut().add_child(entity);

// // //     let f = Rc::new(RefCell::new(None));
// // //     let g = f.clone();
// // //     *(*g).borrow_mut() = Some(Closure::new(move |timestamp: f64| {
// // //         if timestamp <= 30.0 * 1000.0 {
// // //             render.render(&mut scene, timestamp).unwrap();
// // //             request_animation_frame(f.borrow().as_ref().unwrap());
// // //         } else {
// // //             scene.set_mount(None).unwrap();
// // //             console_log!("stop rendering");
// // //         }
// // //     }));

// // //     request_animation_frame(g.borrow().as_ref().unwrap());

// // //     let callback = Closure::once(move || {
// // //         drop(large_buffer);
// // //         drop(large_buffer_1);
// // //     });

// // //     window()
// // //         .set_timeout_with_callback_and_timeout_and_arguments_0(
// // //             callback.into_js_value().unchecked_ref(),
// // //             10 * 1000,
// // //         )
// // //         .unwrap();

// // //     Ok(())
// // // }

// // // #[wasm_bindgen]
// // // pub fn test_binary() {
// // //     let b0 = Uint8Array::new_with_length(1 * 1024 * 1024 * 1024);
// // //     b0.fill(1, 0, 1 * 1024 * 1024 * 1024);
// // //     let b1 = Uint8Array::new_with_length(1 * 1024 * 1024 * 1024);
// // //     b1.fill(1, 0, 1 * 1024 * 1024 * 1024);
// // //     let b2 = Uint8Array::new_with_length(1 * 1024 * 1024 * 1024);
// // //     b2.fill(1, 0, 1 * 1024 * 1024 * 1024);
// // //     let b3 = b0.clone();
// // //     let b4 = b0.clone();
// // //     let b5 = b0.clone();
// // //     let b6 = b0.clone();
// // //     let b7 = b0.clone();
// // //     let b8 = b0.clone();
// // //     let b9 = b0.clone();

// // //     let callback = Closure::once(|| {
// // //         drop(b0);
// // //         drop(b1);
// // //         drop(b2);
// // //         drop(b3);
// // //         drop(b4);
// // //         drop(b5);
// // //         drop(b6);
// // //         drop(b7);
// // //         drop(b8);
// // //         drop(b9);
// // //         console_log!("dropped")
// // //     });

// // //     window()
// // //         .set_timeout_with_callback_and_timeout_and_arguments_0(
// // //             callback.into_js_value().unchecked_ref(),
// // //             10 * 1000,
// // //         )
// // //         .unwrap();
// // // }

#[wasm_bindgen]
pub fn test_pick(
    count: usize,
    grid: usize,
    width: f64,
    height: f64,
    render_callback: &Function,
    pick_callback: &Function,
) -> Result<(), Error> {
    let camera = create_camera(
        Vec3::new(0.0, 3.0, 8.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );
    let mut scene = create_scene()?;

    let cell_width = width / (grid as f64);
    let cell_height = height / (grid as f64);
    let start_x = width / 2.0 - cell_width / 2.0;
    let start_z = height / 2.0 - cell_height / 2.0;
    let cubes = SimpleGroup::new();
    for index in 0..count {
        let row = index / grid;
        let col = index % grid;

        let center_x = start_x - col as f64 * cell_width;
        let center_z = start_z - row as f64 * cell_height;
        let model_matrix = Mat4::<f64>::from_translation(&Vec3::new(center_x, 0.0, center_z));

        let mut cube = SimpleEntity::new();
        cube.set_geometry(Some(Cube::new()));
        cube.set_material(Some(SolidColorMaterial::with_color(
            Vec3::new(rand::random(), rand::random(), rand::random()),
            128.0,
            rand::random(),
        )));
        cube.set_model_matrix(model_matrix);
        cubes.borrow_mut().add_entity(cube);
    }
    scene
        .entity_group()
        .borrow_mut()
        .add_sub_group_shared(cubes);

    let viewer = create_viewer(scene, camera, render_callback);
    let viewer = Rc::new(RefCell::new(viewer));
    viewer
        .borrow()
        .scene()
        .borrow_mut()
        .canvas_handler()
        .click()
        .borrow_mut()
        .register(ViewerPicker {
            viewer: Rc::downgrade(&viewer),
            pick_callback: pick_callback.clone(),
        });

    viewer.borrow_mut().start_render_loop();

    Ok(())
}

// // #[wasm_bindgen]
// // pub fn test_camera() {
// //     let camera = PerspectiveCamera::new(
// //         (0.0, 0.0, 1.0),
// //         (0.0, 0.0, 0.0),
// //         (0.0, 1.0, 0.0),
// //         60.0f64.to_radians(),
// //         1080.0 / 1920.0,
// //         1.0,
// //         Some(2.0),
// //     );

// //     // let camera = PerspectiveCamera::new(
// //     //     (0.0, 1.0, 0.0),
// //     //     (0.0, 0.0, 0.0),
// //     //     (0.0, 0.0, -1.0),
// //     //     60.0f64.to_radians(),
// //     //     1080.0 / 1920.0,
// //     //     0.1,
// //     //     Some(2.0),
// //     // );
// //     let frustum = camera.view_frustum();
// //     console_log!(
// //         "near ({}), ({})",
// //         frustum.near().normal(),
// //         frustum.near().point_on_plane()
// //     );
// //     console_log!(
// //         "far ({:?}), ({:?})",
// //         frustum.far().map(|p| p.normal()),
// //         frustum.far().map(|p| p.point_on_plane())
// //     );
// //     console_log!(
// //         "top ({}), ({})",
// //         frustum.top().normal(),
// //         frustum.top().point_on_plane()
// //     );
// //     console_log!(
// //         "bottom ({}), ({})",
// //         frustum.bottom().normal(),
// //         frustum.bottom().point_on_plane()
// //     );
// //     console_log!(
// //         "left ({}), ({})",
// //         frustum.left().normal(),
// //         frustum.left().point_on_plane()
// //     );
// //     console_log!(
// //         "right ({}), ({})",
// //         frustum.right().normal(),
// //         frustum.right().point_on_plane()
// //     );

// //     let position = Vec4::new(0.0, 0.0, -1.0, 1.0);

// //     let view_matrix = camera.view_matrix();
// //     let view_translated_matrix = view_matrix.translate(&(0.0, 0.0, 2.0));
// //     let view_inv_matrix = view_matrix.invert().unwrap();
// //     let proj_matrix = camera.proj_matrix();
// //     let view_proj_matrix = camera.view_proj_matrix();

// //     console_log!("{}", position.transform_mat4(&view_matrix));
// //     console_log!("{}", position.transform_mat4(&view_translated_matrix));
// //     console_log!(
// //         "{}",
// //         Vec3::new(0.0, 0.0, 1.0).transform_mat4(&view_translated_matrix)
// //     );
// //     console_log!(
// //         "{}",
// //         Vec3::new(0.0, 0.0, 3.0).transform_mat4(&view_translated_matrix)
// //     );
// //     console_log!(
// //         "{}",
// //         Vec3::new(0.0, 0.0, 0.0).transform_mat4(&view_matrix)
// //     );
// //     console_log!(
// //         "{}",
// //         Vec3::new(0.0, 0.0, -1.0).transform_mat4(&view_inv_matrix)
// //     );
// //     console_log!(
// //         "{}",
// //         Vec3::new(0.0, 0.0, 1.0).transform_mat4(&view_inv_matrix)
// //     );
// //     console_log!("{}", Vec3::new().transform_mat4(&view_matrix));
// //     console_log!("{}", Vec3::new().transform_mat4(&view_inv_matrix));
// //     console_log!("{}", position.transform_mat4(&view_proj_matrix));
// //     console_log!(
// //         "{}",
// //         position.transform_mat4(&view_proj_matrix) / position.transform_mat4(&view_proj_matrix).w()
// //     );
// // }

// // // #[wasm_bindgen]
// // // pub fn test_simd() {
// // //         let vec1 = gl_matrix4rust::wasm32::simd128::f32::vec4::Vec4::new(1.0, 1.0, 1.0, 1.0);

// // //     let count = 1500000000usize;
// // //     let start = window().performance().unwrap().now();
// // //     let mut result = gl_matrix4rust::wasm32::simd128::f32::vec4::Vec4::new(1.0, 1.0, 1.0, 1.0);
// // //     for _ in 0..count {
// // //         result = result * vec1;
// // //     }
// // //     let end = window().performance().unwrap().now();

// // //     console_log!("simd costs {}ms", end - start);
// // // }

// // // #[wasm_bindgen]
// // // pub fn test_non_simd() {
// // //         let vec1 = Vec4::<f32>::new(1.0, 1.0, 1.0, 1.0);

// // //     let count = 1500000000usize;
// // //     let start = window().performance().unwrap().now();
// // //     let mut result = Vec4::<f32>::new(1.0, 1.0, 1.0, 1.0);
// // //     for _ in 0..count {
// // //         result = result * vec1;
// // //     }
// // //     let end = window().performance().unwrap().now();

// // //     console_log!("non simd costs {}ms", end - start);
// // // }
