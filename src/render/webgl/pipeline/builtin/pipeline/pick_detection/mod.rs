use std::{
    any::Any,
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

use smallvec::SmallVec;
use uuid::Uuid;
use wasm_bindgen::JsCast;
use web_sys::{
    js_sys::Uint32Array, HtmlCanvasElement, WebGl2RenderingContext, WebGlFramebuffer,
    WebGlRenderbuffer, WebGlTexture,
};

use crate::{
    entity::Entity,
    material::{Material, MaterialRenderEntity},
    render::webgl::{
        attribute::{AttributeBinding, AttributeValue},
        draw::CullFace,
        error::Error,
        pipeline::{
            builtin::processor::{
                EnableCullFace, EnableDepthTest, SetCullFaceMode, UpdateCamera, UpdateViewport, Reset,
            },
            policy::{CollectPolicy, GeometryPolicy, MaterialPolicy, PreparationPolicy},
            process::Processor,
            RenderPipeline, RenderState, RenderStuff,
        },
        program::ShaderSource,
        uniform::{UniformBinding, UniformValue},
    },
};

pub struct PickDetection {
    position: Option<(i32, i32)>,
    picked: Option<Weak<RefCell<Entity>>>,
    result: Uint32Array,
    material: Rc<RefCell<PickDetectionMaterial>>,
    framebuffer: Option<WebGlFramebuffer>,
    renderbuffer: Option<(WebGlRenderbuffer, u32, u32)>,
    texture: Option<(WebGlTexture, u32, u32)>,
}

impl PickDetection {
    pub fn new() -> Self {
        Self {
            material: Rc::new(RefCell::new(PickDetectionMaterial::new())),
            result: Uint32Array::new_with_length(1),
            framebuffer: None,
            renderbuffer: None,
            texture: None,
            position: None,
            picked: None,
        }
    }

    pub fn set_position(&mut self, x: i32, y: i32) {
        self.position = Some((x, y));
    }

    pub fn pick(&self) -> Option<Rc<RefCell<Entity>>> {
        self.picked.as_ref().and_then(|entity| entity.upgrade())
    }

    fn canvas_from_gl(&self, gl: &WebGl2RenderingContext) -> Result<HtmlCanvasElement, Error> {
        gl.canvas()
            .ok_or(Error::CanvasNotFound)?
            .dyn_into::<HtmlCanvasElement>()
            .or(Err(Error::CanvasNotFound))
    }

    fn use_framebuffer(&mut self, gl: &WebGl2RenderingContext) -> Result<WebGlFramebuffer, Error> {
        let framebuffer = &mut self.framebuffer;
        let framebuffer = match framebuffer {
            Some(framebuffer) => framebuffer.clone(),
            None => {
                let fbo = gl
                    .create_framebuffer()
                    .ok_or(Error::CreateFramebufferFailure)?;
                *framebuffer = Some(fbo.clone());
                fbo
            }
        };

        Ok(framebuffer)
    }

    fn use_depth_renderbuffer(
        &mut self,
        gl: &WebGl2RenderingContext,
    ) -> Result<WebGlRenderbuffer, Error> {
        let canvas = self.canvas_from_gl(gl)?;
        let w = canvas.width();
        let h = canvas.height();

        if let Some((renderbuffer, width, height)) = &self.renderbuffer {
            if w == *width || h == *height {
                return Ok(renderbuffer.clone());
            } else {
                gl.delete_renderbuffer(Some(renderbuffer));
            }
        }

        let rb = gl
            .create_renderbuffer()
            .ok_or(Error::CreateRenderbufferFailure)?;

        gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, Some(&rb));
        gl.renderbuffer_storage(
            WebGl2RenderingContext::RENDERBUFFER,
            WebGl2RenderingContext::DEPTH_COMPONENT16,
            w as i32,
            h as i32,
        );
        gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, None);

        self.renderbuffer = Some((rb.clone(), w, h));

        Ok(rb)
    }

    fn use_texture(&mut self, gl: &WebGl2RenderingContext) -> Result<WebGlTexture, Error> {
        let canvas = self.canvas_from_gl(gl)?;
        let w = canvas.width();
        let h = canvas.height();

        if let Some((texture, width, height)) = &self.texture {
            if w == *width || h == *height {
                return Ok(texture.clone());
            } else {
                gl.delete_texture(Some(texture));
            }
        }

        let tx = gl.create_texture().ok_or(Error::CreateTextureFailure)?;

        gl.active_texture(WebGl2RenderingContext::TEXTURE0);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&tx));
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            WebGl2RenderingContext::R32UI as i32,
            w as i32,
            h as i32,
            0,
            WebGl2RenderingContext::RED_INTEGER,
            WebGl2RenderingContext::UNSIGNED_INT,
            None,
        )
        .or_else(|err| Err(Error::TexImageFailure(err.as_string())))?;
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

        self.texture = Some((tx.clone(), w, h));

        Ok(tx)
    }
}

impl RenderPipeline for PickDetection {
    fn dependencies(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn prepare(
        &mut self,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<PreparationPolicy, Error> {
        self.picked = None;
        self.material.borrow_mut().clear();
        // aborts if no position specified.
        if self.position.is_some() {
            Ok(PreparationPolicy::Continue)
        } else {
            Ok(PreparationPolicy::Abort)
        }
    }

    fn pre_processors(
        &mut self,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<SmallVec<[Box<dyn Processor<Self>>; 12]>, Error> {
        let mut processors: SmallVec<[Box<dyn Processor<Self>>; 12]> = SmallVec::new();
        processors.push(Box::new(UsePickDetectionFramebuffer));
        processors.push(Box::new(UpdateCamera));
        processors.push(Box::new(UpdateViewport));
        processors.push(Box::new(EnableDepthTest));
        processors.push(Box::new(EnableCullFace));
        processors.push(Box::new(PickDetectionClear));
        processors.push(Box::new(SetCullFaceMode::new(CullFace::Back)));
        Ok(processors)
    }

    fn material_policy(
        &mut self,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<MaterialPolicy, Error> {
        Ok(MaterialPolicy::Overwrite(Some(
            Rc::clone(&self.material) as Rc<RefCell<dyn Material>>
        )))
    }

    fn geometry_policy(
        &mut self,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<GeometryPolicy, Error> {
        Ok(GeometryPolicy::FollowEntity)
    }

    fn collect_policy(
        &mut self,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<CollectPolicy, Error> {
        Ok(CollectPolicy::CollectAll)
    }

    fn post_processors(
        &mut self,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<SmallVec<[Box<dyn Processor<Self>>; 12]>, Error> {
        let mut processors: SmallVec<[Box<dyn Processor<Self>>; 12]> = SmallVec::new();
        processors.push(Box::new(PickDetectionPickEntity));
        processors.push(Box::new(Reset));
        Ok(processors)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

struct UsePickDetectionFramebuffer;

impl Processor<PickDetection> for UsePickDetectionFramebuffer {
    fn name(&self) -> &str {
        "UsePickDetectionFramebuffer"
    }

    fn process(
        &mut self,
        pipeline: &mut PickDetection,
        state: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
        let gl = &state.gl;

        let framebuffer = pipeline.use_framebuffer(gl)?;
        let renderbuffer = pipeline.use_depth_renderbuffer(gl)?;
        let texture = pipeline.use_texture(gl)?;

        gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&framebuffer));
        gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, Some(&renderbuffer));
        gl.active_texture(WebGl2RenderingContext::TEXTURE0);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
        gl.framebuffer_renderbuffer(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::DEPTH_ATTACHMENT,
            WebGl2RenderingContext::RENDERBUFFER,
            Some(&renderbuffer),
        );
        gl.framebuffer_texture_2d(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::COLOR_ATTACHMENT0,
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&texture),
            0,
        );

        Ok(())
    }
}

struct PickDetectionClear;

impl Processor<PickDetection> for PickDetectionClear {
    fn name(&self) -> &str {
        "PickDetectionClear"
    }

    fn process(
        &mut self,
        _: &mut PickDetection,
        state: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
        state
            .gl
            .clear_bufferuiv_with_u32_array(WebGl2RenderingContext::COLOR, 0, &[0, 0, 0, 0]);
        state
            .gl
            .clear_bufferfv_with_f32_array(WebGl2RenderingContext::DEPTH, 0, &[1.0]);
        Ok(())
    }
}

struct PickDetectionPickEntity;

impl Processor<PickDetection> for PickDetectionPickEntity {
    fn name(&self) -> &str {
        "PickDetectionPickEntity"
    }

    fn process(
        &mut self,
        pipeline: &mut PickDetection,
        state: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
        let gl = &state.gl;

        let (x, y) = pipeline.position.as_ref().unwrap(); // safe unwrap

        let canvas = pipeline.canvas_from_gl(gl)?;

        gl.read_pixels_with_opt_array_buffer_view(
            *x,
            canvas.height() as i32 - *y,
            1,
            1,
            WebGl2RenderingContext::RED_INTEGER,
            WebGl2RenderingContext::UNSIGNED_INT,
            Some(&pipeline.result),
        )
        .or_else(|err| Err(Error::CommonWebGLError(err.as_string())))?;

        pipeline.picked = pipeline
            .material
            .borrow()
            .index2entity
            .get(&pipeline.result.get_index(0))
            .map(|entity| Rc::downgrade(&entity));

        Ok(())
    }
}

const VERTEX_SHADER_SOURCE: &'static str = include_str!("./vertex.glsl");
const FRAGMENT_SHADER_SOURCE: &'static str = include_str!("./fragment.glsl");

struct PickDetectionMaterial {
    id2index: HashMap<Uuid, UniformValue>,
    index2entity: HashMap<u32, Rc<RefCell<Entity>>>,
}

impl PickDetectionMaterial {
    fn new() -> Self {
        Self {
            id2index: HashMap::new(),
            index2entity: HashMap::new(),
        }
    }

    fn clear(&mut self) {
        self.id2index.clear();
        self.index2entity.clear();
    }
}

impl Material for PickDetectionMaterial {
    fn name(&self) -> &'static str {
        "PickDetectionMaterial"
    }

    fn attribute_bindings(&self) -> &[AttributeBinding] {
        &[AttributeBinding::GeometryPosition]
    }

    fn uniform_bindings(&self) -> &[UniformBinding] {
        &[
            UniformBinding::ModelViewProjMatrix,
            UniformBinding::FromMaterial("u_Index"),
        ]
    }

    fn sources<'a>(&'a self) -> &[ShaderSource<'a>] {
        &[
            ShaderSource::Vertex(VERTEX_SHADER_SOURCE),
            ShaderSource::Fragment(FRAGMENT_SHADER_SOURCE),
        ]
    }

    fn attribute_value(&self, _: &str, _: &MaterialRenderEntity) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, name: &str, entity: &MaterialRenderEntity) -> Option<UniformValue> {
        match name {
            "u_Index" => self.id2index.get(entity.entity().borrow().id()).cloned(),
            _ => None,
        }
    }

    fn ready(&self) -> bool {
        true
    }

    fn instanced(&self) -> Option<i32> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn prepare(&mut self, _: &RenderState, entity: &Rc<RefCell<Entity>>) {
        let index = self.id2index.len() + 1; // index 0 as nothing
        if index >= u32::MAX as usize {
            panic!("too may entities in scene");
        }

        let index = index as u32;
        self.id2index
            .insert(*entity.borrow().id(), UniformValue::UnsignedInteger1(index));
        self.index2entity.insert(index, Rc::clone(entity));
    }
}
