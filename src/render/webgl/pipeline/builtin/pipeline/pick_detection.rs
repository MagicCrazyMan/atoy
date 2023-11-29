use std::{
    any::Any,
    cell::RefCell,
    rc::{Rc, Weak},
};

use wasm_bindgen::JsCast;
use wasm_bindgen_test::console_log;
use web_sys::{
    js_sys::Uint32Array, HtmlCanvasElement, WebGl2RenderingContext, WebGlFramebuffer,
    WebGlRenderbuffer, WebGlTexture,
};

use crate::{
    entity::{Entity, RenderEntity},
    geometry::Geometry,
    material::{Material, MaterialRenderEntity},
    render::webgl::{
        attribute::{AttributeBinding, AttributeValue},
        error::Error,
        pipeline::{drawer::Drawer, RenderPipeline, RenderState, RenderStuff},
        program::ShaderSource,
        uniform::{UniformBinding, UniformValue},
    },
};

pub struct PickDetectionDrawer {
    position: Option<(i32, i32)>,
    picked: Option<Weak<RefCell<Entity>>>,
    result: Uint32Array,
    material: Rc<RefCell<PickDetectionMaterial>>,
    framebuffer: Option<WebGlFramebuffer>,
    renderbuffer: Option<(WebGlRenderbuffer, u32, u32)>,
    texture: Option<(WebGlTexture, u32, u32)>,
}

impl PickDetectionDrawer {
    pub fn new() -> Self {
        Self {
            material: Rc::new(RefCell::new(PickDetectionMaterial)),
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

impl<Pipeline> Drawer<Pipeline> for PickDetectionDrawer
where
    Pipeline: RenderPipeline,
{
    fn before_draw(
        &mut self,
        collected: &Vec<Rc<RefCell<Entity>>>,
        _: &mut Pipeline,
        state: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<Option<Vec<Rc<RefCell<Entity>>>>, Error> {
        if self.position.is_none() {
            return Ok(None);
        }

        if collected.len() > u32::MAX as usize {
            console_log!(
                "too many entities: {}, skipping pick detection",
                collected.len()
            );
            return Ok(None);
        }

        let gl = &state.gl;

        let framebuffer = self.use_framebuffer(gl)?;
        let renderbuffer = self.use_depth_renderbuffer(gl)?;
        let texture = self.use_texture(gl)?;
        gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&framebuffer));
        gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, Some(&renderbuffer));
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
        gl.framebuffer_texture_2d(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::COLOR_ATTACHMENT0,
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&texture),
            0,
        );
        gl.framebuffer_renderbuffer(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::DEPTH_COMPONENT24,
            WebGl2RenderingContext::RENDERBUFFER,
            Some(&renderbuffer),
        );

        Ok(Some(collected.clone()))
    }

    fn before_each_draw(
        &mut self,
        entity: &Rc<RefCell<Entity>>,
        _: &Vec<Rc<RefCell<Entity>>>,
        _: &Vec<Rc<RefCell<Entity>>>,
        _: &mut Pipeline,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<
        Option<(
            Rc<RefCell<Entity>>,
            Rc<RefCell<dyn Geometry>>,
            Rc<RefCell<dyn Material>>,
        )>,
        Error,
    > {
        if let Some(geometry) = entity.borrow().geometry() {
            Ok(Some((
                Rc::clone(entity),
                Rc::clone(geometry),
                Rc::clone(&self.material) as Rc<RefCell<dyn Material>>,
            )))
        } else {
            Ok(None)
        }
    }

    fn after_each_draw(
        &mut self,
        _: &RenderEntity,
        _: &Vec<Rc<RefCell<Entity>>>,
        _: &Vec<Rc<RefCell<Entity>>>,
        _: &mut Pipeline,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn after_draw(
        &mut self,
        filtered: &Vec<Rc<RefCell<Entity>>>,
        _: &Vec<Rc<RefCell<Entity>>>,
        _: &mut Pipeline,
        state: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<(), Error> {
        let gl = &state.gl;
        let (x, y) = &self.position.unwrap(); // safe unwrap
        gl.read_pixels_with_opt_array_buffer_view(
            *x,
            self.canvas_from_gl(gl)?.height() as i32 - *y,
            1,
            1,
            WebGl2RenderingContext::RED_INTEGER,
            WebGl2RenderingContext::UNSIGNED_INT,
            Some(&self.result),
        )
        .or_else(|err| Err(Error::CommonWebGLError(err.as_string())))?;

        self.picked = filtered
            .get(self.result.get_index(0) as usize)
            .map(|entity| Rc::downgrade(entity));

        gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);
        gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, None);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
        Ok(())
    }
}

const VERTEX_SHADER_SOURCE: &'static str = include_str!("./shaders/pick_detection_vertex.glsl");
const FRAGMENT_SHADER_SOURCE: &'static str = include_str!("./shaders/pick_detection_fragment.glsl");

struct PickDetectionMaterial;

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
            "u_Index" => Some(UniformValue::UnsignedInteger1(
                entity.filtered_index() as u32
            )),
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
}
