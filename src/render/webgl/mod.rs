use std::collections::{HashMap, VecDeque};

use gl_matrix4rust::{
    mat4::{AsMat4, Mat4},
    vec3::AsVec3,
    vec4::Vec4,
};
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};
use wasm_bindgen_test::console_log;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation};

use crate::{
    entity::Entity, event::EventTarget, geometry::Geometry, material::Material, scene::Scene,
};

use self::{
    attribute::{AttributeBinding, AttributeValue},
    buffer::{BufferStore, BufferTarget},
    conversion::{GLfloat, GLint, GLuint, ToGlEnum},
    draw::{CullFace, Draw},
    error::Error,
    program::ProgramStore,
    texture::TextureStore,
    uniform::{UniformBinding, UniformValue},
};

pub mod attribute;
pub mod buffer;
pub mod conversion;
pub mod draw;
pub mod error;
pub mod program;
pub mod texture;
pub mod uniform;

#[wasm_bindgen(typescript_custom_section)]
const WEBGL2_RENDER_OPTIONS_TYPE: &'static str = r#"
export type WebGL2RenderOptions = WebGLContextAttributes;
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "WebGL2RenderOptions")]
    pub type WebGL2RenderOptionsObject;
}

struct EventTargets<'a> {
    before_render: EventTarget<()>,
    before_prepare: EventTarget<()>,
    after_prepare: EventTarget<()>,
    before_entity_bind_attributes: EventTarget<(
        &'a Entity,
        &'a dyn Geometry,
        &'a dyn Material,
        &'a HashMap<AttributeBinding, GLuint>,
    )>,
    after_entity_bind_attributes: EventTarget<(
        &'a Entity,
        &'a dyn Geometry,
        &'a dyn Material,
        &'a HashMap<AttributeBinding, GLuint>,
    )>,
    before_entity_bind_uniforms: EventTarget<(
        &'a Entity,
        &'a dyn Geometry,
        &'a dyn Material,
        &'a HashMap<UniformBinding, WebGlUniformLocation>,
    )>,
    after_entity_bind_uniforms: EventTarget<(
        &'a Entity,
        &'a dyn Geometry,
        &'a dyn Material,
        &'a HashMap<UniformBinding, WebGlUniformLocation>,
    )>,
    before_entity_draw: EventTarget<(&'a Entity, &'a dyn Geometry, &'a dyn Material)>,
    after_entity_draw: EventTarget<(&'a Entity, &'a dyn Geometry, &'a dyn Material)>,
    after_render: EventTarget<()>,
}

impl<'a> EventTargets<'a> {
    fn new() -> Self {
        Self {
            before_render: EventTarget::new(),
            before_prepare: EventTarget::new(),
            after_prepare: EventTarget::new(),
            before_entity_bind_attributes: EventTarget::new(),
            after_entity_bind_attributes: EventTarget::new(),
            before_entity_bind_uniforms: EventTarget::new(),
            after_entity_bind_uniforms: EventTarget::new(),
            before_entity_draw: EventTarget::new(),
            after_entity_draw: EventTarget::new(),
            after_render: EventTarget::new(),
        }
    }
}

pub struct WebGL2Render<'a> {
    gl: WebGl2RenderingContext,
    depth_test: bool,
    cull_face: Option<CullFace>,
    clear_color: Vec4,
    program_store: ProgramStore,
    buffer_store: BufferStore,
    texture_store: TextureStore,
    event_targets: EventTargets<'a>,
}

impl<'a> WebGL2Render<'a> {
    /// Constructs a new WebGL2 render.
    pub fn new(scene: &Scene) -> Result<WebGL2Render<'a>, Error> {
        Self::new_inner(scene, None)
    }

    /// Constructs a new WebGL2 render.
    pub fn with_options(
        scene: &Scene,
        options: WebGL2RenderOptionsObject,
    ) -> Result<WebGL2Render<'a>, Error> {
        Self::new_inner(scene, Some(options))
    }

    fn new_inner(
        scene: &Scene,
        options: Option<WebGL2RenderOptionsObject>,
    ) -> Result<WebGL2Render<'a>, Error> {
        let gl = Self::gl_context(scene.canvas(), options)?;
        let mut render = Self {
            program_store: ProgramStore::new(gl.clone()),
            buffer_store: BufferStore::new(gl.clone()),
            texture_store: TextureStore::new(gl.clone()),
            gl,
            depth_test: true,
            cull_face: None,
            clear_color: Vec4::new(),
            event_targets: EventTargets::new(),
        };

        render.set_clear_color(Vec4::new());
        render.set_cull_face(None);
        render.set_depth_test(true);

        Ok(render)
    }

    /// Gets WebGl2RenderingContext.
    fn gl_context(
        canvas: &HtmlCanvasElement,
        options: Option<WebGL2RenderOptionsObject>,
    ) -> Result<WebGl2RenderingContext, Error> {
        let options = match options {
            Some(options) => options.obj,
            None => JsValue::UNDEFINED,
        };

        let gl = canvas
            .get_context_with_context_options("webgl2", &options)
            .ok()
            .and_then(|context| context)
            .and_then(|context| context.dyn_into::<WebGl2RenderingContext>().ok())
            .ok_or(Error::WebGl2RenderingContextNotFound)?;

        gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);

        Ok(gl)
    }
}

impl<'a> WebGL2Render<'a> {
    pub fn depth_test(&self) -> bool {
        self.depth_test
    }

    pub fn set_depth_test(&mut self, depth_test: bool) {
        self.depth_test = depth_test;
        if self.depth_test {
            self.gl.enable(WebGl2RenderingContext::DEPTH_TEST);
        } else {
            self.gl.disable(WebGl2RenderingContext::DEPTH_TEST);
        }
    }

    pub fn cull_face(&self) -> Option<CullFace> {
        self.cull_face
    }

    pub fn set_cull_face(&mut self, cull_face_mode: Option<CullFace>) {
        self.cull_face = cull_face_mode;
        match self.cull_face {
            Some(cull_face_mode) => {
                self.gl.enable(WebGl2RenderingContext::CULL_FACE);
                self.gl.cull_face(cull_face_mode.gl_enum())
            }
            None => self.gl.disable(WebGl2RenderingContext::CULL_FACE),
        }
    }

    pub fn clear_color(&self) -> Vec4 {
        self.clear_color
    }

    pub fn set_clear_color(&mut self, clear_color: Vec4) {
        self.clear_color = clear_color;
        self.gl.clear_color(
            self.clear_color.0[0] as GLfloat,
            self.clear_color.0[1] as GLfloat,
            self.clear_color.0[2] as GLfloat,
            self.clear_color.0[3] as GLfloat,
        );
    }

    pub fn gl(&self) -> &WebGl2RenderingContext {
        &self.gl
    }

    pub fn buffer_store(&self) -> &BufferStore {
        &self.buffer_store
    }

    pub fn buffer_store_mut(&mut self) -> &mut BufferStore {
        &mut self.buffer_store
    }
}

impl<'a> WebGL2Render<'a> {
    pub fn before_render(&mut self) -> &mut EventTarget<()> {
        &mut self.event_targets.before_render
    }

    pub fn before_prepare(&mut self) -> &mut EventTarget<()> {
        &mut self.event_targets.before_prepare
    }

    pub fn after_prepare(&mut self) -> &mut EventTarget<()> {
        &mut self.event_targets.after_prepare
    }

    pub fn before_entity_bind_attributes(
        &mut self,
    ) -> &mut EventTarget<(
        &'a Entity,
        &'a dyn Geometry,
        &'a dyn Material,
        &'a HashMap<AttributeBinding, GLuint>,
    )> {
        &mut self.event_targets.before_entity_bind_attributes
    }

    pub fn after_entity_bind_attributes(
        &mut self,
    ) -> &mut EventTarget<(
        &'a Entity,
        &'a dyn Geometry,
        &'a dyn Material,
        &'a HashMap<AttributeBinding, GLuint>,
    )> {
        &mut self.event_targets.after_entity_bind_attributes
    }

    pub fn before_entity_bind_uniforms(
        &mut self,
    ) -> &mut EventTarget<(
        &'a Entity,
        &'a dyn Geometry,
        &'a dyn Material,
        &'a HashMap<UniformBinding, WebGlUniformLocation>,
    )> {
        &mut self.event_targets.before_entity_bind_uniforms
    }

    pub fn after_entity_bind_uniforms(
        &mut self,
    ) -> &mut EventTarget<(
        &'a Entity,
        &'a dyn Geometry,
        &'a dyn Material,
        &'a HashMap<UniformBinding, WebGlUniformLocation>,
    )> {
        &mut self.event_targets.after_entity_bind_uniforms
    }

    pub fn before_entity_draw(
        &mut self,
    ) -> &mut EventTarget<(&'a Entity, &'a dyn Geometry, &'a dyn Material)> {
        &mut self.event_targets.before_entity_draw
    }

    pub fn after_entity_draw(
        &mut self,
    ) -> &mut EventTarget<(&'a Entity, &'a dyn Geometry, &'a dyn Material)> {
        &mut self.event_targets.after_entity_draw
    }

    pub fn after_render(&mut self) -> &mut EventTarget<()> {
        &mut self.event_targets.after_render
    }
}

struct RenderGroup {
    program: *const WebGlProgram,
    attribute_locations: *const HashMap<AttributeBinding, GLuint>,
    uniform_locations: *const HashMap<UniformBinding, WebGlUniformLocation>,
    entities: Vec<RenderItem>,
}

struct RenderItem {
    entity_ptr: *mut Entity,
    geometry_ptr: *mut dyn Geometry,
    material_ptr: *mut dyn Material,
}

impl<'a> WebGL2Render<'a> {
    /// Render frame.
    pub fn render(&mut self, scene: &mut Scene) -> Result<(), Error> {
        self.event_targets.before_render.raise(());

        // update WebGL viewport
        self.gl.viewport(
            0,
            0,
            scene.canvas().width() as i32,
            scene.canvas().height() as i32,
        );

        // collects entities and render console_error_panic_hook
        let entities_group = self.prepare(scene)?;

        // clear scene
        self.gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        self.gl.clear(WebGl2RenderingContext::DEPTH_BUFFER_BIT);

        // render each entities group
        for (
            _,
            RenderGroup {
                program,
                attribute_locations,
                uniform_locations,
                entities,
            },
        ) in entities_group
        {
            let (program, attribute_locations, uniform_locations) =
                unsafe { (&*program, &*attribute_locations, &*uniform_locations) };

            // binds program
            self.gl.use_program(Some(program));

            // render each entity
            for entity_item in entities {
                let RenderItem {
                    entity_ptr,
                    geometry_ptr,
                    material_ptr,
                } = entity_item;
                let (entity, geometry, material) =
                    unsafe { (&mut *entity_ptr, &mut *geometry_ptr, &mut *material_ptr) };

                // pre-render
                self.pre_render(scene, entity, geometry, material);
                // binds attributes
                self.bind_attributes(attribute_locations, entity_ptr, geometry_ptr, material_ptr);
                // binds uniforms
                self.bind_uniforms(
                    scene,
                    uniform_locations,
                    entity_ptr,
                    geometry_ptr,
                    material_ptr,
                );
                // draws
                self.draw(entity_ptr, geometry_ptr, material_ptr);
                // post-render
                self.post_render(scene, entity, geometry, material);
            }
        }

        self.event_targets.after_render.raise(());

        Ok(())
    }

    /// Prepares graphic scene.
    /// Updates entities matrices using current frame status, collects and groups all entities.
    fn prepare(&mut self, scene: &mut Scene) -> Result<HashMap<String, RenderGroup>, Error> {
        self.event_targets.before_prepare.raise(());

        let view_matrix = scene.active_camera().view_matrix();
        let proj_matrix = scene.active_camera().proj_matrix();

        let mut group: HashMap<String, RenderGroup> = HashMap::new();

        let mut rollings: VecDeque<*mut Entity> =
            VecDeque::from([scene.root_entity_mut() as *mut Entity]);
        while let Some(entity) = rollings.pop_front() {
            let entity = unsafe { &mut *entity };

            // update entity matrices in current frame
            let parent_model_matrix = entity
                .parent()
                .map(|parent| parent.model_matrix() as *const Mat4);

            if let Err(err) =
                entity.update_frame_matrices(parent_model_matrix, &view_matrix, &proj_matrix)
            {
                // should log warning
                console_log!("{}", err);
                continue;
            }

            // filters any entity that has no geometry or material
            // groups entities by material to prevent unnecessary program switching
            if let (Some(geometry), Some(material)) = (entity.geometry_raw(), entity.material_raw())
            {
                let (geometry, material) = unsafe { (&mut *geometry, &mut *material) };

                // calls prepare callback
                material.prepare(scene, entity, geometry);

                // check whether material is ready or not
                if material.ready() {
                    let render_item = RenderItem {
                        entity_ptr: entity,
                        geometry_ptr: geometry,
                        material_ptr: material,
                    };
                    match group.get_mut(material.name()) {
                        Some(group) => group.entities.push(render_item),
                        None => {
                            // precompile material to program
                            let item = self.program_store.program_or_compile(material)?;

                            group.insert(
                                material.name().to_string(),
                                RenderGroup {
                                    program: item.program(),
                                    attribute_locations: item.attribute_locations(),
                                    uniform_locations: item.uniform_locations(),
                                    entities: vec![render_item],
                                },
                            );
                        }
                    };
                }
            }

            // add children to rollings list
            rollings.extend(
                entity
                    .children_mut()
                    .iter_mut()
                    .map(|child| child as *mut Entity),
            );
        }

        self.event_targets.after_prepare.raise(());

        Ok(group)
    }

    /// Calls pre-render callback of the entity.
    fn pre_render(
        &self,
        scene: &mut Scene,
        entity: &mut Entity,
        geometry: &mut dyn Geometry,
        material: &mut dyn Material,
    ) {
        material.pre_render(scene, entity, geometry);
    }

    /// Calls post-render callback of the entity.
    fn post_render(
        &self,
        scene: &mut Scene,
        entity: &mut Entity,
        geometry: &mut dyn Geometry,
        material: &mut dyn Material,
    ) {
        material.post_render(scene, entity, geometry);
    }

    /// Binds attributes of the entity.
    fn bind_attributes(
        &mut self,
        attribute_locations: &'a HashMap<AttributeBinding, GLuint>,
        entity_ptr: *const Entity,
        geometry_ptr: *const dyn Geometry,
        material_ptr: *const dyn Material,
    ) {
        let (entity, geometry, material) =
            unsafe { (&*entity_ptr, &*geometry_ptr, &*material_ptr) };

        self.event_targets.before_entity_bind_attributes.raise((
            entity,
            geometry,
            material,
            attribute_locations,
        ));

        let gl = &self.gl;

        for (binding, location) in attribute_locations {
            let value = match binding {
                AttributeBinding::GeometryPosition => geometry.vertices(),
                AttributeBinding::GeometryTextureCoordinate => geometry.texture_coordinates(),
                AttributeBinding::GeometryNormal => geometry.normals(),
                AttributeBinding::FromGeometry(name) => geometry.attribute_value(name),
                AttributeBinding::FromMaterial(name) => material.attribute_value(name),
                AttributeBinding::FromEntity(name) => entity.attribute_value(name),
            };
            let Some(value) = value else {
                // should log warning
                console_log!("3");
                continue;
            };

            match value {
                AttributeValue::Buffer {
                    descriptor,
                    target,
                    component_size,
                    data_type,
                    normalized,
                    bytes_stride,
                    bytes_offset,
                } => {
                    match self.buffer_store.use_buffer(&descriptor, target) {
                        Ok(buffer) => buffer,
                        Err(err) => {
                            // should log error
                            console_log!("{}", err);
                            continue;
                        }
                    };

                    gl.vertex_attrib_pointer_with_i32(
                        *location,
                        component_size as GLint,
                        data_type.gl_enum(),
                        normalized,
                        bytes_stride,
                        bytes_offset,
                    );
                    gl.enable_vertex_attrib_array(*location);
                }
                AttributeValue::InstancedBuffer {
                    descriptor,
                    target,
                    component_size,
                    data_type,
                    normalized,
                    component_count_per_instance: components_length_per_instance,
                    divisor,
                } => {
                    match self.buffer_store.use_buffer(&descriptor, target) {
                        Ok(buffer) => buffer,
                        Err(err) => {
                            // should log error
                            console_log!("{}", err);
                            continue;
                        }
                    };

                    let component_size = component_size as GLint;
                    // binds each instance
                    for i in 0..components_length_per_instance {
                        let offset_location = *location + (i as GLuint);
                        gl.vertex_attrib_pointer_with_i32(
                            offset_location,
                            component_size,
                            data_type.gl_enum(),
                            normalized,
                            data_type.bytes_length()
                                * component_size
                                * components_length_per_instance,
                            i * data_type.bytes_length() * component_size,
                        );
                        gl.enable_vertex_attrib_array(offset_location);
                        gl.vertex_attrib_divisor(offset_location, divisor);
                    }
                    gl.bind_buffer(target.gl_enum(), None);
                }
                AttributeValue::Vertex1f(x) => gl.vertex_attrib1f(*location, x),
                AttributeValue::Vertex2f(x, y) => gl.vertex_attrib2f(*location, x, y),
                AttributeValue::Vertex3f(x, y, z) => gl.vertex_attrib3f(*location, x, y, z),
                AttributeValue::Vertex4f(x, y, z, w) => gl.vertex_attrib4f(*location, x, y, z, w),
                AttributeValue::Vertex1fv(v) => gl.vertex_attrib1fv_with_f32_array(*location, &v),
                AttributeValue::Vertex2fv(v) => gl.vertex_attrib2fv_with_f32_array(*location, &v),
                AttributeValue::Vertex3fv(v) => gl.vertex_attrib3fv_with_f32_array(*location, &v),
                AttributeValue::Vertex4fv(v) => gl.vertex_attrib4fv_with_f32_array(*location, &v),
            };

            self.event_targets.after_entity_bind_attributes.raise((
                entity,
                geometry,
                material,
                attribute_locations,
            ));
        }
    }

    /// Binds uniform data of the entity.
    fn bind_uniforms(
        &mut self,
        scene: &Scene,
        uniform_locations: &'a HashMap<UniformBinding, WebGlUniformLocation>,
        entity_ptr: *const Entity,
        geometry_ptr: *const dyn Geometry,
        material_ptr: *const dyn Material,
    ) {
        let (entity, geometry, material) =
            unsafe { (&*entity_ptr, &*geometry_ptr, &*material_ptr) };

        self.event_targets.before_entity_bind_uniforms.raise((
            entity,
            geometry,
            material,
            uniform_locations,
        ));

        let gl = &self.gl;

        for (binding, location) in uniform_locations {
            let value = match binding {
                UniformBinding::FromGeometry(name) => geometry.uniform_value(name),
                UniformBinding::FromMaterial(name) => material.uniform_value(name),
                UniformBinding::FromEntity(name) => entity.uniform_value(name),
                UniformBinding::ParentModelMatrix
                | UniformBinding::ModelMatrix
                | UniformBinding::NormalMatrix
                | UniformBinding::ModelViewMatrix
                | UniformBinding::ModelViewProjMatrix
                | UniformBinding::ViewProjMatrix => {
                    let mat = match binding {
                        UniformBinding::ParentModelMatrix => match entity.parent() {
                            Some(parent) => parent.model_matrix().to_gl(),
                            None => Mat4::<f32>::new_identity().to_gl(), // use identity if not exists
                        },
                        UniformBinding::ModelMatrix => entity.model_matrix().to_gl(),
                        UniformBinding::NormalMatrix => entity.normal_matrix().to_gl(),
                        UniformBinding::ModelViewMatrix => entity.model_view_matrix().to_gl(),
                        UniformBinding::ModelViewProjMatrix => {
                            entity.model_view_proj_matrix().to_gl()
                        }
                        UniformBinding::ViewProjMatrix => {
                            scene.active_camera().view_proj_matrix().to_gl()
                        }
                        _ => unreachable!(),
                    };

                    Some(UniformValue::Matrix4 {
                        data: mat,
                        transpose: false,
                    })
                }
                UniformBinding::ActiveCameraPosition | UniformBinding::ActiveCameraDirection => {
                    let vec = match binding {
                        UniformBinding::ActiveCameraPosition => {
                            scene.active_camera().position().to_gl()
                        }
                        UniformBinding::ActiveCameraDirection => {
                            scene.active_camera().direction().to_gl()
                        }
                        _ => unreachable!(),
                    };

                    Some(UniformValue::FloatVector3(vec))
                }
            };
            let Some(value) = value else {
                // should log warning
                continue;
            };

            match value {
                UniformValue::UnsignedInteger1(x) => gl.uniform1ui(Some(location), x),
                UniformValue::UnsignedInteger2(x, y) => gl.uniform2ui(Some(location), x, y),
                UniformValue::UnsignedInteger3(x, y, z) => gl.uniform3ui(Some(location), x, y, z),
                UniformValue::UnsignedInteger4(x, y, z, w) => {
                    gl.uniform4ui(Some(location), x, y, z, w)
                }
                UniformValue::FloatVector1(data) => {
                    gl.uniform1fv_with_f32_array(Some(location), &data)
                }
                UniformValue::FloatVector2(data) => {
                    gl.uniform2fv_with_f32_array(Some(location), &data)
                }
                UniformValue::FloatVector3(data) => {
                    gl.uniform3fv_with_f32_array(Some(location), &data)
                }
                UniformValue::FloatVector4(data) => {
                    gl.uniform4fv_with_f32_array(Some(location), &data)
                }
                UniformValue::IntegerVector1(data) => {
                    gl.uniform1iv_with_i32_array(Some(location), &data)
                }
                UniformValue::IntegerVector2(data) => {
                    gl.uniform2iv_with_i32_array(Some(location), &data)
                }
                UniformValue::IntegerVector3(data) => {
                    gl.uniform3iv_with_i32_array(Some(location), &data)
                }
                UniformValue::IntegerVector4(data) => {
                    gl.uniform4iv_with_i32_array(Some(location), &data)
                }
                UniformValue::UnsignedIntegerVector1(data) => {
                    gl.uniform1uiv_with_u32_array(Some(location), &data)
                }
                UniformValue::UnsignedIntegerVector2(data) => {
                    gl.uniform2uiv_with_u32_array(Some(location), &data)
                }
                UniformValue::UnsignedIntegerVector3(data) => {
                    gl.uniform3uiv_with_u32_array(Some(location), &data)
                }
                UniformValue::UnsignedIntegerVector4(data) => {
                    gl.uniform4uiv_with_u32_array(Some(location), &data)
                }
                UniformValue::Matrix2 { data, transpose } => {
                    gl.uniform_matrix2fv_with_f32_array(Some(location), transpose, &data)
                }
                UniformValue::Matrix3 { data, transpose } => {
                    gl.uniform_matrix3fv_with_f32_array(Some(location), transpose, &data)
                }
                UniformValue::Matrix4 { data, transpose } => {
                    gl.uniform_matrix4fv_with_f32_array(Some(location), transpose, &data)
                }
                UniformValue::Texture {
                    descriptor,
                    params,
                    texture_unit,
                } => {
                    // active texture
                    gl.active_texture(texture_unit.gl_enum());

                    let (target, texture) = match self.texture_store.use_texture(&descriptor)
                    {
                        Ok(texture) => texture,
                        Err(err) => {
                            // should log warning
                            console_log!("{}", err);
                            continue;
                        }
                    };

                    // binds texture
                    gl.bind_texture(target, Some(texture));
                    // setups sampler parameters
                    params
                        .iter()
                        .for_each(|param| param.tex_parameteri(gl, target));
                    // binds to shader
                    gl.uniform1i(Some(location), texture_unit.unit_index());
                }
            };

            self.event_targets.after_entity_bind_uniforms.raise((
                entity,
                geometry,
                material,
                uniform_locations,
            ));
        }
    }

    fn draw(
        &mut self,
        entity_ptr: *const Entity,
        geometry_ptr: *const dyn Geometry,
        material_ptr: *const dyn Material,
    ) {
        let (entity, geometry, material) =
            unsafe { (&*entity_ptr, &*geometry_ptr, &*material_ptr) };

        self.event_targets
            .before_entity_draw
            .raise((entity, geometry, material));

        let gl = &self.gl;

        // draws entity
        if let Some(num_instances) = material.instanced() {
            // dr: f64aw instanced
            match geometry.draw() {
                Draw::Arrays {
                    mode,
                    first,
                    count: num_vertices,
                } => gl.draw_arrays_instanced(mode.gl_enum(), first, num_vertices, num_instances),
                Draw::Elements {
                    mode,
                    count: num_vertices,
                    element_type,
                    offset,
                    indices,
                } => {
                    match self
                        .buffer_store
                        .use_buffer(&indices, BufferTarget::ElementArrayBuffer)
                    {
                        Ok(buffer) => buffer,
                        Err(err) => {
                            // should log warning
                            console_log!("{}", err);
                            return;
                        }
                    };

                    gl.draw_elements_instanced_with_i32(
                        mode.gl_enum(),
                        num_vertices,
                        element_type.gl_enum(),
                        offset,
                        num_instances,
                    );
                }
            }
        } else {
            // draw normally!
            match geometry.draw() {
                Draw::Arrays {
                    mode,
                    first,
                    count: num_vertices,
                } => gl.draw_arrays(mode.gl_enum(), first, num_vertices),
                Draw::Elements {
                    mode,
                    count: num_vertices,
                    element_type,
                    offset,
                    indices,
                } => {
                    match self
                        .buffer_store
                        .use_buffer(&indices, BufferTarget::ElementArrayBuffer)
                    {
                        Ok(buffer) => buffer,
                        Err(err) => {
                            // should log warning
                            console_log!("{}", err);
                            return;
                        }
                    };

                    gl.draw_elements_with_i32(
                        mode.gl_enum(),
                        num_vertices,
                        element_type.gl_enum(),
                        offset,
                    );
                }
            }
        }

        self.event_targets
            .after_entity_draw
            .raise((entity, geometry, material));
    }
}
