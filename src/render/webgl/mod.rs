use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap, VecDeque},
    rc::Rc,
};

use gl_matrix4rust::{
    mat4::{AsMat4, Mat4},
    vec3::AsVec3,
};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_test::console_log;
use web_sys::{
    HtmlCanvasElement, HtmlElement, ResizeObserver, ResizeObserverEntry, WebGl2RenderingContext,
    WebGlProgram, WebGlUniformLocation,
};

use crate::{document, entity::Entity, geometry::Geometry, material::Material};

use self::{
    attribute::{AttributeBinding, AttributeValue},
    buffer::{BufferStore, BufferTarget},
    conversion::{GLint, GLuint, ToGlEnum},
    draw::Draw,
    error::Error,
    pipeline::{
        policy::{CollectPolicy, GeometryPolicy, MaterialPolicy},
        preprocess::PreProcessor,
        RenderPipeline, RenderState, RenderStuff,
    },
    program::ProgramStore,
    texture::TextureStore,
    uniform::{UniformBinding, UniformValue},
};

pub mod attribute;
pub mod buffer;
pub mod conversion;
pub mod draw;
pub mod error;
pub mod pick;
pub mod pipeline;
pub mod program;
pub mod texture;
pub mod uniform;

// #[wasm_bindgen(typescript_custom_section)]
// const WEBGL2_RENDER_OPTIONS_TYPE: &'static str = r#"
// export type WebGL2RenderOptions = WebGLContextAttributes;
// "#;

// #[wasm_bindgen]
// extern "C" {
//     #[wasm_bindgen(typescript_type = "WebGL2RenderOptions")]
//     pub type WebGL2RenderOptionsObject;
// }

pub struct WebGL2Render {
    mount: Option<HtmlElement>,
    canvas: HtmlCanvasElement,
    // require for storing callback closure function
    resize_observer: (ResizeObserver, Closure<dyn FnMut(Vec<ResizeObserverEntry>)>),
    gl: WebGl2RenderingContext,
    program_store: ProgramStore,
    buffer_store: BufferStore,
    texture_store: TextureStore,
}

impl WebGL2Render {
    pub fn new() -> Result<Self, Error> {
        Self::new_inner(None)
    }

    pub fn with_mount(mount: &str) -> Result<Self, Error> {
        Self::new_inner(Some(mount))
    }

    /// Constructs a new WebGL2 render.
    fn new_inner(mount: Option<&str>) -> Result<Self, Error> {
        let canvas = document()
            .create_element("canvas")
            .ok()
            .and_then(|ele| ele.dyn_into::<HtmlCanvasElement>().ok())
            .ok_or(Error::CreateCanvasFailure)?;
        canvas.style().set_css_text("width: 100%; height: 100%;");

        let resize_observer = Self::observer_canvas_size(&canvas);

        let gl = canvas
            .get_context_with_context_options("webgl2", &JsValue::undefined())
            .ok()
            .and_then(|context| context)
            .and_then(|context| context.dyn_into::<WebGl2RenderingContext>().ok())
            .ok_or(Error::WenGL2Unsupported)?;

        let mut render = Self {
            mount: None,
            program_store: ProgramStore::new(gl.clone()),
            buffer_store: BufferStore::with_max_memory(gl.clone(), 2 * 1024 * 1024 * 1024),
            // buffer_store: BufferStore::with_max_memory(gl.clone(), 2000),
            texture_store: TextureStore::new(gl.clone()),
            canvas,
            gl,
            resize_observer,
        };

        render.set_mount(mount)?;

        Ok(render)
    }

    fn observer_canvas_size(
        canvas: &HtmlCanvasElement,
    ) -> (ResizeObserver, Closure<dyn FnMut(Vec<ResizeObserverEntry>)>) {
        // create observer observing size change event of canvas
        let resize_observer_callback = Closure::new(move |entries: Vec<ResizeObserverEntry>| {
            // should have only one entry
            let Some(target) = entries.get(0).map(|entry| entry.target()) else {
                return;
            };
            let Some(canvas) = target.dyn_ref::<HtmlCanvasElement>() else {
                return;
            };

            canvas.set_width(canvas.client_width() as u32);
            canvas.set_height(canvas.client_height() as u32);
        });

        let resize_observer =
            ResizeObserver::new(resize_observer_callback.as_ref().unchecked_ref()).unwrap();
        resize_observer.observe(canvas);

        (resize_observer, resize_observer_callback)
    }
}

impl Drop for WebGL2Render {
    fn drop(&mut self) {
        // cleanups observers
        self.resize_observer.0.disconnect();
    }
}

impl WebGL2Render {
    /// Gets mount target.
    pub fn mount(&self) -> Option<&HtmlElement> {
        match &self.mount {
            Some(mount) => Some(mount),
            None => None,
        }
    }

    /// Sets the mount target.
    pub fn set_mount(&mut self, mount: Option<&str>) -> Result<(), Error> {
        if let Some(mount) = mount {
            if !mount.is_empty() {
                // gets and sets mount target using `document.getElementById`
                let mount = document()
                    .get_element_by_id(&mount)
                    .and_then(|ele| ele.dyn_into::<HtmlElement>().ok())
                    .ok_or(Error::MountElementNotFound)?;

                // mounts canvas to target (creates new if not exists)
                mount.append_child(&self.canvas).unwrap();
                let width = mount.client_width() as u32;
                let height = mount.client_height() as u32;
                self.canvas.set_width(width);
                self.canvas.set_height(height);

                self.mount = Some(mount);

                return Ok(());
            }
        }

        // for all other situations, removes canvas from mount target
        self.canvas.remove();
        self.mount = None;
        Ok(())
    }

    pub fn buffer_store(&self) -> &BufferStore {
        &self.buffer_store
    }

    pub fn buffer_store_mut(&mut self) -> &mut BufferStore {
        &mut self.buffer_store
    }
}

pub struct RenderGroup {
    program: *const WebGlProgram,
    attribute_locations: *const HashMap<AttributeBinding, GLuint>,
    uniform_locations: *const HashMap<UniformBinding, WebGlUniformLocation>,
    entities: Vec<RenderEntity>,
}

impl RenderGroup {
    pub fn entities(&self) -> &[RenderEntity] {
        &self.entities
    }
}

/// [`Entity`] and associated [`Material`] and [`Geometry`] for rendering.
/// Be aware, geometry and material may not extract from entity,
/// which depending on [`MaterialPolicy`] and [`GeometryPolicy`].
pub struct RenderEntity {
    entity: Rc<RefCell<Entity>>,
    geometry: Rc<RefCell<dyn Geometry>>,
    material: Rc<RefCell<dyn Material>>,
}

impl RenderEntity {
    pub fn entity(&self) -> &Rc<RefCell<Entity>> {
        &self.entity
    }

    pub fn geometry(&self) -> &Rc<RefCell<dyn Geometry>> {
        &self.geometry
    }

    pub fn material(&self) -> &Rc<RefCell<dyn Material>> {
        &self.material
    }
}

impl WebGL2Render {
    pub fn render<Stuff, Pipeline>(
        &mut self,
        pipeline: &mut Pipeline,
        stuff: &mut Stuff,
        frame_time: f64,
    ) -> Result<(), Error>
    where
        Stuff: RenderStuff,
        Pipeline: RenderPipeline,
    {
        // constructs render state
        let state = RenderState {
            canvas: self.canvas.clone(),
            gl: self.gl.clone(),
            frame_time,
        };

        // prepares stage, obtains a render stuff
        pipeline.prepare(&state, stuff)?;

        // pre-process stages
        for processor in pipeline.pre_process(&state, stuff)? {
            processor.pre_process(&state, stuff)?;
        }

        // collects render groups
        let groups = self.prepare_entities(pipeline, stuff, &state)?;

        // render stage
        for (
            _,
            RenderGroup {
                program,
                attribute_locations,
                uniform_locations,
                entities,
            },
        ) in groups
        {
            // console_log!("{}", entities.len());
            let (program, attribute_locations, uniform_locations) =
                unsafe { (&*program, &*attribute_locations, &*uniform_locations) };

            // binds program
            state.gl.use_program(Some(program));

            // render each entity
            for entity in entities {
                // pre-render
                // self.pre_render(&state);
                // binds attributes
                self.bind_attributes(&state, &entity, attribute_locations);
                // binds uniforms
                self.bind_uniforms(&state, stuff, &entity, uniform_locations);
                // draws
                self.draw(&state, &entity);
                // post-render
                // self.post_render(&state);
            }
        }

        // post-process stages
        pipeline.post_precess(&state, stuff)?;

        Ok(())
    }

    /// Prepares graphic scene.
    /// Updates entities matrices using current frame status, collects and groups all entities.
    fn prepare_entities<Stuff, Pipeline>(
        &mut self,
        pipeline: &mut Pipeline,
        stuff: &mut Stuff,
        state: &RenderState,
    ) -> Result<HashMap<String, RenderGroup>, Error>
    where
        Stuff: RenderStuff,
        Pipeline: RenderPipeline,
    {
        let view_matrix = stuff.camera().view_matrix();
        let proj_matrix = stuff.camera().proj_matrix();

        let material_policy = pipeline.material_policy(state, stuff)?;
        let geometry_policy = pipeline.geometry_policy(state, stuff)?;
        let collect_policy = pipeline.collect_policy(state, stuff)?;

        let mut groups: HashMap<String, RenderGroup> = HashMap::new();

        let mut collections =
            VecDeque::from([(Mat4::new_identity(), stuff.entity_collection_mut())]);
        while let Some((parent_model_matrix, collection)) = collections.pop_front() {
            // update collection matrices
            collection.update_frame_matrices(&parent_model_matrix);
            let collection_model_matrix = *collection.model_matrix();

            for entity in collection.entities() {
                // update matrices
                if let Err(err) = entity.borrow_mut().update_frame_matrices(
                    &collection_model_matrix,
                    &view_matrix,
                    &proj_matrix,
                ) {
                    // should log warning
                    console_log!("{}", err);
                    continue;
                }

                // selects material
                let material = match &material_policy {
                    MaterialPolicy::FollowEntity => entity.borrow_mut().material().cloned(),
                    MaterialPolicy::Overwrite(material) => material.as_ref().cloned(),
                    MaterialPolicy::Custom(func) => func(&entity.borrow()),
                };
                let Some(material) = material else {
                    continue;
                };
                // trigger material preparation
                material.borrow_mut().prepare(state, &entity);

                // skip if not ready yet
                if !material.borrow().ready() {
                    continue;
                }

                // selects geometry
                let geometry = match &geometry_policy {
                    GeometryPolicy::FollowEntity => entity.borrow_mut().geometry().cloned(),
                    GeometryPolicy::Overwrite(geometry) => geometry.as_ref().cloned(),
                    GeometryPolicy::Custom(func) => func(&entity),
                };
                let Some(geometry) = geometry else {
                    continue;
                };

                // check collectable
                let collectable = match &collect_policy {
                    CollectPolicy::CollectAll => true,
                    CollectPolicy::Custom(func) => func(&groups, &entity, &geometry, &material),
                };

                if !collectable {
                    continue;
                }

                // compile material collect entity
                let material_name = material.borrow().name().to_string();
                match groups.entry(material_name) {
                    Entry::Occupied(mut occupied) => {
                        occupied.get_mut().entities.push(RenderEntity {
                            entity: Rc::clone(&entity),
                            geometry,
                            material,
                        });
                    }
                    Entry::Vacant(vacant) => {
                        let item = self.program_store.use_program(&(*material.borrow()))?;
                        vacant.insert(RenderGroup {
                            program: item.program(),
                            attribute_locations: item.attribute_locations(),
                            uniform_locations: item.uniform_locations(),
                            entities: vec![RenderEntity {
                                entity: Rc::clone(&entity),
                                geometry,
                                material,
                            }],
                        });
                    }
                };
            }

            // add sub-collections to list
            collections.extend(
                collection
                    .collections_mut()
                    .iter_mut()
                    .map(|collection| (collection_model_matrix, collection)),
            );
        }

        Ok(groups)
    }

    /// Calls pre-render callback of the entity.
    fn pre_render(&self, entity: &RenderEntity) {
        // entity.material.borrow_mut().pre_render(entity);
    }

    /// Calls post-render callback of the entity.
    fn post_render(&self, state: &RenderEntity) {
        // state.material().post_render(state);
    }

    /// Binds attributes of the entity.
    fn bind_attributes(
        &mut self,
        RenderState { gl, .. }: &RenderState,
        RenderEntity {
            entity,
            geometry,
            material,
        }: &RenderEntity,
        attribute_locations: &HashMap<AttributeBinding, GLuint>,
    ) {
        for (binding, location) in attribute_locations {
            let value = match binding {
                AttributeBinding::GeometryPosition => geometry.borrow().vertices(),
                AttributeBinding::GeometryTextureCoordinate => {
                    geometry.borrow().texture_coordinates()
                }
                AttributeBinding::GeometryNormal => geometry.borrow().normals(),
                AttributeBinding::FromGeometry(name) => {
                    geometry.borrow().attribute_value(name, entity)
                }
                AttributeBinding::FromMaterial(name) => {
                    material.borrow().attribute_value(name, entity)
                }
                AttributeBinding::FromEntity(name) => {
                    entity.borrow().attribute_values().get(*name).cloned()
                }
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
                    let buffer = match self.buffer_store.use_buffer(descriptor, target) {
                        Ok(buffer) => buffer,
                        Err(err) => {
                            // should log error
                            console_log!("{}", err);
                            continue;
                        }
                    };

                    gl.bind_buffer(target.gl_enum(), Some(&buffer));
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
                    let buffer = match self.buffer_store.use_buffer(descriptor, target) {
                        Ok(buffer) => buffer,
                        Err(err) => {
                            // should log error
                            console_log!("{}", err);
                            continue;
                        }
                    };

                    gl.bind_buffer(target.gl_enum(), Some(&buffer));

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
        }
    }

    /// Binds uniform data of the entity.
    fn bind_uniforms(
        &mut self,
        RenderState { gl, .. }: &RenderState,
        stuff: &dyn RenderStuff,
        RenderEntity {
            entity,
            geometry,
            material,
        }: &RenderEntity,
        uniform_locations: &HashMap<UniformBinding, WebGlUniformLocation>,
    ) {
        for (binding, location) in uniform_locations {
            let value = match binding {
                UniformBinding::FromGeometry(name) => geometry.borrow().uniform_value(name, entity),
                UniformBinding::FromMaterial(name) => material.borrow().uniform_value(name, entity),
                UniformBinding::FromEntity(name) => {
                    entity.borrow().uniform_values().get(*name).cloned()
                }
                UniformBinding::ModelMatrix
                | UniformBinding::NormalMatrix
                | UniformBinding::ModelViewMatrix
                | UniformBinding::ModelViewProjMatrix
                | UniformBinding::ViewProjMatrix => {
                    let mat = match binding {
                        UniformBinding::ModelMatrix => entity.borrow().model_matrix().to_gl(),
                        UniformBinding::NormalMatrix => entity.borrow().normal_matrix().to_gl(),
                        UniformBinding::ModelViewMatrix => {
                            entity.borrow().model_view_matrix().to_gl()
                        }
                        UniformBinding::ModelViewProjMatrix => {
                            entity.borrow().model_view_proj_matrix().to_gl()
                        }
                        UniformBinding::ViewProjMatrix => stuff.camera().view_proj_matrix().to_gl(),
                        _ => unreachable!(),
                    };

                    Some(UniformValue::Matrix4 {
                        data: mat,
                        transpose: false,
                    })
                }
                UniformBinding::ActiveCameraPosition | UniformBinding::ActiveCameraDirection => {
                    let vec = match binding {
                        UniformBinding::ActiveCameraPosition => stuff.camera().position().to_gl(),
                        UniformBinding::ActiveCameraDirection => stuff.camera().direction().to_gl(),
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

                    let (target, texture) = match self.texture_store.use_texture(&descriptor) {
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
        }
    }

    fn draw(
        &mut self,
        RenderState { gl, .. }: &RenderState,
        RenderEntity {
            geometry, material, ..
        }: &RenderEntity,
    ) {
        let material = material.borrow();
        let geometry = geometry.borrow();

        // draws entity
        if let Some(num_instances) = material.instanced() {
            // draw instanced
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
                    let buffer = match self
                        .buffer_store
                        .use_buffer(indices, BufferTarget::ElementArrayBuffer)
                    {
                        Ok(buffer) => buffer,
                        Err(err) => {
                            // should log warning
                            console_log!("{}", err);
                            return;
                        }
                    };

                    gl.bind_buffer(BufferTarget::ElementArrayBuffer.gl_enum(), Some(&buffer));
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
                    let buffer = match self
                        .buffer_store
                        .use_buffer(indices, BufferTarget::ElementArrayBuffer)
                    {
                        Ok(buffer) => buffer,
                        Err(err) => {
                            // should log warning
                            console_log!("{}", err);
                            return;
                        }
                    };

                    gl.bind_buffer(BufferTarget::ElementArrayBuffer.gl_enum(), Some(&buffer));
                    gl.draw_elements_with_i32(
                        mode.gl_enum(),
                        num_vertices,
                        element_type.gl_enum(),
                        offset,
                    );
                }
            }
        }
    }
}
