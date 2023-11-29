use std::{
    borrow::Cow,
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use gl_matrix4rust::{mat4::AsMat4, vec3::AsVec3};
use wasm_bindgen::{closure::Closure, JsCast};
use wasm_bindgen_test::console_log;
use web_sys::{
    HtmlCanvasElement, HtmlElement, ResizeObserver, ResizeObserverEntry, WebGl2RenderingContext,
    WebGlUniformLocation,
};

use crate::{
    document,
    entity::{Entity, RenderEntity},
    geometry::GeometryRenderEntity,
    material::MaterialRenderEntity,
};

use self::{
    attribute::{AttributeBinding, AttributeValue},
    buffer::{BufferStore, BufferTarget},
    conversion::{GLint, GLuint, ToGlEnum},
    draw::Draw,
    error::Error,
    pipeline::{
        flow::{BeforeDrawFlow, BeforeEachDrawFlow, PreparationFlow},
        RenderPipeline, RenderState, RenderStuff,
    },
    program::{ProgramItem, ProgramStore},
    texture::TextureStore,
    uniform::{UniformBinding, UniformValue},
};

pub mod attribute;
pub mod buffer;
pub mod conversion;
pub mod draw;
pub mod error;
pub mod pipeline;
pub mod program;
pub mod stencil;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct WebGL2ContextOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    alpha: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    depth: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stencil: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    desynchronized: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    antialias: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fail_if_major_performance_caveat: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    power_preference: Option<WebGL2ContextPowerPerformance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    premultiplied_alpha: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    preserve_drawing_buffer: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    xr_compatible: Option<bool>,
}

impl Default for WebGL2ContextOptions {
    fn default() -> Self {
        Self {
            alpha: Some(true),
            depth: Some(true),
            stencil: Some(true),
            desynchronized: None,
            antialias: Some(true),
            fail_if_major_performance_caveat: None,
            power_preference: None,
            premultiplied_alpha: None,
            preserve_drawing_buffer: None,
            xr_compatible: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum WebGL2ContextPowerPerformance {
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "high-performance")]
    HighPerformance,
    #[serde(rename = "low-power")]
    LowPower,
}

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
        Self::new_inner(None, None)
    }

    pub fn with_mount(mount: &str) -> Result<Self, Error> {
        Self::new_inner(Some(mount), None)
    }

    /// Constructs a new WebGL2 render.
    fn new_inner(
        mount: Option<&str>,
        options: Option<WebGL2ContextOptions>,
    ) -> Result<Self, Error> {
        let canvas = document()
            .create_element("canvas")
            .ok()
            .and_then(|ele| ele.dyn_into::<HtmlCanvasElement>().ok())
            .ok_or(Error::CreateCanvasFailure)?;
        canvas.style().set_css_text("width: 100%; height: 100%;");

        let resize_observer = Self::observer_canvas_size(&canvas);

        let options = options.unwrap_or(WebGL2ContextOptions::default());
        let gl = canvas
            .get_context_with_context_options(
                "webgl2",
                &serde_wasm_bindgen::to_value(&options).unwrap(),
            )
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
        let mut state = RenderState::new(self.canvas.clone(), self.gl.clone(), stuff, frame_time);
        let state = &mut state;

        // prepares stage, obtains a render stuff
        match pipeline.prepare(state, stuff)? {
            PreparationFlow::Continue => {}
            PreparationFlow::Abort => return Ok(()),
        };

        // collects render groups
        let collected_entities = self.collect_entities(stuff)?;

        // pre-process stages
        for processor in pipeline.pre_processors(&collected_entities, state, stuff)? {
            processor.borrow_mut().process(pipeline, state, stuff)?;
        }

        // draw stage
        let drawers = pipeline.drawers(&collected_entities, state, stuff)?;
        let mut last_program = None as Option<ProgramItem>;
        for drawer in drawers {
            let mut drawer = drawer.borrow_mut();
            let drawing_entities =
                match drawer.before_draw(&collected_entities, pipeline, state, stuff)? {
                    BeforeDrawFlow::Skip => continue,
                    BeforeDrawFlow::FollowCollectedEntities => Cow::Borrowed(&collected_entities),
                    BeforeDrawFlow::Custom(entities) => Cow::Owned(entities),
                };

            for (index, entity) in drawing_entities.iter().enumerate() {
                unsafe {
                    // before each draw of drawer
                    let (geometry, material) = match drawer.before_each_draw(
                        entity,
                        &drawing_entities,
                        &collected_entities,
                        pipeline,
                        state,
                        stuff,
                    )? {
                        BeforeEachDrawFlow::Skip => continue,
                        BeforeEachDrawFlow::FollowEntity => {
                            let mut entity = entity.borrow_mut();
                            (entity.geometry_raw(), entity.material_raw())
                        }
                        BeforeEachDrawFlow::OverwriteMaterial(material) => {
                            (entity.borrow_mut().geometry_raw(), Some(material))
                        }
                        BeforeEachDrawFlow::OverwriteGeometry(geometry) => {
                            (Some(geometry), entity.borrow_mut().material_raw())
                        }
                        BeforeEachDrawFlow::Overwrite(geometry, material) => {
                            (Some(geometry), Some(material))
                        }
                    };
                    let (Some(geometry), Some(material)) = (geometry, material) else {
                        continue;
                    };

                    // prepare material and geometry
                    (&mut *material).prepare(state, &entity);
                    (&mut *geometry).prepare(state, &entity);

                    // skip if not ready yet
                    if !(&mut *material).ready() {
                        continue;
                    }

                    let render_entity = RenderEntity::new(
                        Rc::clone(&entity),
                        geometry,
                        material,
                        &collected_entities,
                        &drawing_entities,
                        index,
                    );
                    let geometry_render_entity = GeometryRenderEntity::new(
                        Rc::clone(render_entity.entity()),
                        material,
                        &collected_entities,
                        &drawing_entities,
                        index,
                    );
                    let material_render_entity = MaterialRenderEntity::new(
                        Rc::clone(render_entity.entity()),
                        geometry,
                        &collected_entities,
                        &drawing_entities,
                        index,
                    );

                    // compile and bind program only when last program isn't equals the material
                    if last_program
                        .as_ref()
                        .map(|last_program| last_program.name() != (&*material).name())
                        .unwrap_or(true)
                    {
                        let p = self.program_store.use_program(&*material)?;
                        self.gl.use_program(Some(p.program()));
                        last_program = Some(p.clone());
                    }

                    let program_item = last_program.as_ref().unwrap();

                    // binds attributes
                    self.bind_attributes(
                        &state,
                        &render_entity,
                        &geometry_render_entity,
                        &material_render_entity,
                        program_item.attribute_locations(),
                    );
                    // binds uniforms
                    self.bind_uniforms(
                        &state,
                        stuff,
                        &render_entity,
                        &geometry_render_entity,
                        &material_render_entity,
                        program_item.uniform_locations(),
                    );

                    // before draw of material and geometry
                    (&mut *material).before_draw(state, &material_render_entity);
                    (&mut *geometry).before_draw(state, &geometry_render_entity);
                    // draws
                    self.draw(&state, &render_entity);
                    // after draw of material and geometry
                    (&mut *material).after_draw(state, &material_render_entity);
                    (&mut *geometry).after_draw(state, &geometry_render_entity);
                    // after each draw of drawer
                    drawer.after_each_draw(
                        &render_entity,
                        &drawing_entities,
                        &collected_entities,
                        pipeline,
                        state,
                        stuff,
                    )?;
                }
            }
            drawer.after_draw(
                &drawing_entities,
                &collected_entities,
                pipeline,
                state,
                stuff,
            )?;
        }

        // post-process stages
        for processor in pipeline.post_processors(&collected_entities, state, stuff)? {
            processor.borrow_mut().process(pipeline, state, stuff)?;
        }

        Ok(())
    }

    /// Prepares graphic scene.
    /// Updates entities matrices using current frame status, collects and groups all entities.
    fn collect_entities<Stuff>(
        &mut self,
        stuff: &mut Stuff,
    ) -> Result<Vec<Rc<RefCell<Entity>>>, Error>
    where
        Stuff: RenderStuff,
    {
        // let mut collected = HashMap::new();
        let mut collected = Vec::new();
        // entities collections waits for collecting. If parent model does not changed, set matrix to None.
        let mut collections = VecDeque::from([(None, stuff.entity_collection_mut())]);
        while let Some((parent_model_matrix, collection)) = collections.pop_front() {
            // update collection matrices
            let mut collection_model_matrix = None;
            if collection.update_frame_matrices(parent_model_matrix) {
                collection_model_matrix = Some(*collection.model_matrix());
            }

            for entity in collection.entities() {
                // update matrices
                if let Err(err) = entity
                    .borrow_mut()
                    .update_frame_matrices(collection_model_matrix)
                {
                    // should log warning
                    console_log!("{}", err);
                    continue;
                }
                collected.push(Rc::clone(entity));
            }

            // add sub-collections to list
            collections.extend(
                collection
                    .collections_mut()
                    .iter_mut()
                    .map(|collection| (collection_model_matrix, collection)),
            );
        }

        Ok(collected)
    }

    /// Binds attributes of the entity.
    fn bind_attributes(
        &mut self,
        state: &RenderState,
        render_entity: &RenderEntity,
        geometry_render_entity: &GeometryRenderEntity,
        material_render_entity: &MaterialRenderEntity,
        attribute_locations: &HashMap<AttributeBinding, GLuint>,
    ) {
        let gl = state.gl();

        for (binding, location) in attribute_locations {
            let value = match binding {
                AttributeBinding::GeometryPosition => render_entity.geometry().vertices(),
                AttributeBinding::GeometryTextureCoordinate => {
                    render_entity.geometry().texture_coordinates()
                }
                AttributeBinding::GeometryNormal => render_entity.geometry().normals(),
                AttributeBinding::FromGeometry(name) => render_entity
                    .geometry()
                    .attribute_value(name, geometry_render_entity),
                AttributeBinding::FromMaterial(name) => render_entity
                    .material()
                    .attribute_value(name, material_render_entity),
                AttributeBinding::FromEntity(name) => render_entity
                    .entity()
                    .borrow()
                    .attribute_values()
                    .get(*name)
                    .cloned(),
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
        state: &RenderState,
        stuff: &dyn RenderStuff,
        render_entity: &RenderEntity,
        geometry_render_entity: &GeometryRenderEntity,
        material_render_entity: &MaterialRenderEntity,
        uniform_locations: &HashMap<UniformBinding, WebGlUniformLocation>,
    ) {
        let gl = state.gl();

        for (binding, location) in uniform_locations {
            let value = match binding {
                UniformBinding::FromGeometry(name) => render_entity
                    .geometry()
                    .uniform_value(name, geometry_render_entity),
                UniformBinding::FromMaterial(name) => render_entity
                    .material()
                    .uniform_value(name, material_render_entity),
                UniformBinding::FromEntity(name) => render_entity
                    .entity()
                    .borrow()
                    .uniform_values()
                    .get(*name)
                    .cloned(),
                UniformBinding::ModelMatrix
                | UniformBinding::ViewMatrix
                | UniformBinding::ProjMatrix
                | UniformBinding::NormalMatrix
                | UniformBinding::ViewProjMatrix => {
                    let mat = match binding {
                        UniformBinding::ModelMatrix => {
                            render_entity.entity().borrow().model_matrix().to_gl()
                        }
                        UniformBinding::NormalMatrix => {
                            render_entity.entity().borrow().normal_matrix().to_gl()
                        }
                        UniformBinding::ViewMatrix => stuff.camera().view_matrix().to_gl(),
                        UniformBinding::ProjMatrix => stuff.camera().proj_matrix().to_gl(),
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

    fn draw(&mut self, state: &RenderState, entity: &RenderEntity) {
        let gl = state.gl();

        // draws entity
        if let Some(num_instances) = entity.material().instanced() {
            // draw instanced
            match entity.geometry().draw() {
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
            match entity.geometry().draw() {
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
