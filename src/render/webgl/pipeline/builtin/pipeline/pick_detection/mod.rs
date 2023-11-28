use std::{cell::RefCell, collections::HashMap, rc::Rc};

use uuid::Uuid;
use wasm_bindgen::JsCast;
use web_sys::{
    HtmlCanvasElement, WebGl2RenderingContext, WebGlFramebuffer, WebGlRenderbuffer, WebGlTexture,
};

use crate::{
    entity::Entity,
    material::Material,
    render::webgl::{
        attribute::{AttributeBinding, AttributeValue},
        draw::CullFace,
        error::Error,
        pipeline::{
            builtin::{
                postprocessor::{
                    ClearColor, ClearDepth, EnableCullFace, EnableDepthTest, SetCullFaceMode,
                    UpdateCamera, UpdateViewport,
                },
                preprocessor::Reset,
            },
            policy::{CollectPolicy, GeometryPolicy, MaterialPolicy},
            postprocess::PostProcessor,
            preprocess::PreProcessor,
            RenderPipeline, RenderState, RenderStuff,
        },
        program::ShaderSource,
        uniform::{UniformBinding, UniformValue},
    },
};

pub struct PickDetection {
    position: Option<(i32, i32)>,
    picked: Option<Rc<RefCell<Entity>>>,
    material: Rc<RefCell<PickDetectionMaterial>>,
    framebuffer: Option<WebGlFramebuffer>,
    renderbuffer: Option<(WebGlRenderbuffer, u32, u32)>,
    texture: Option<(WebGlTexture, u32, u32)>,
}

impl PickDetection {
    pub fn new() -> Self {
        Self {
            material: Rc::new(RefCell::new(PickDetectionMaterial::new())),
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

    pub fn pick(&self) -> Option<&Rc<RefCell<Entity>>> {
        self.picked.as_ref()
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

    fn prepare(&mut self, _: &mut RenderState, _: &mut dyn RenderStuff) -> Result<(), Error> {
        self.picked = None;
        self.material.borrow_mut().clear();
        Ok(())
    }

    fn pre_process(
        &mut self,
        state: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<Vec<Box<dyn PreProcessor>>, Error> {
        Ok(vec![
            Box::new(PickDetectionPreProcessor::new(
                self.use_framebuffer(&state.gl)?,
                self.use_depth_renderbuffer(&state.gl)?,
                self.use_texture(&state.gl)?,
            )),
            Box::new(UpdateCamera),
            Box::new(UpdateViewport),
            Box::new(EnableDepthTest),
            Box::new(EnableCullFace),
            Box::new(ClearColor::new(0.0, 0.0, 0.0, 0.0)),
            Box::new(ClearDepth::new(1.0)),
            Box::new(SetCullFaceMode::new(CullFace::Back)),
        ])
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

    fn post_precess(
        &mut self,
        _: &mut RenderState,
        _: &mut dyn RenderStuff,
    ) -> Result<Vec<Box<dyn PostProcessor>>, Error> {
        Ok(vec![Box::new(Reset)])
    }
}

struct PickDetectionPreProcessor {
    framebuffer: WebGlFramebuffer,
    renderbuffer: WebGlRenderbuffer,
    texture: WebGlTexture,
}

impl PickDetectionPreProcessor {
    fn new(
        framebuffer: WebGlFramebuffer,
        renderbuffer: WebGlRenderbuffer,
        texture: WebGlTexture,
    ) -> Self {
        Self {
            framebuffer,
            renderbuffer,
            texture,
        }
    }
}

impl PreProcessor for PickDetectionPreProcessor {
    fn name(&self) -> &str {
        "PickDetectionPreProcessor"
    }

    fn pre_process(&mut self, state: &RenderState, _: &mut dyn RenderStuff) -> Result<(), Error> {
        let gl = &state.gl;
        gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&self.framebuffer));
        gl.bind_renderbuffer(
            WebGl2RenderingContext::RENDERBUFFER,
            Some(&self.renderbuffer),
        );
        gl.active_texture(WebGl2RenderingContext::TEXTURE0);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.texture));
        gl.framebuffer_renderbuffer(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::DEPTH_ATTACHMENT,
            WebGl2RenderingContext::RENDERBUFFER,
            Some(&self.renderbuffer),
        );
        gl.framebuffer_texture_2d(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::COLOR_ATTACHMENT0,
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&self.texture),
            0,
        );

        Ok(())
    }
}

struct PickDetectionPostProcessor {
    position: (i32, i32),
}

impl PostProcessor for PickDetectionPostProcessor {
    fn name(&self) -> &str {
        "PickDetectionPostProcessor"
    }

    fn post_process(&mut self, state: &RenderState, stuff: &mut dyn RenderStuff) -> Result<(), Error> {
        todo!()
    }
}

// struct BindFramebuffer {
//     gl: WebGl2RenderingContext,
//     framebuffer: Option<WebGlFramebuffer>,
//     renderbuffer: Option<(WebGlRenderbuffer, u32, u32)>,
//     texture: Option<(WebGlTexture, u32, u32)>,
// }

// impl BindFramebuffer {
//     fn new(gl: WebGl2RenderingContext) -> Self {
//         Self {
//             gl,
//             framebuffer: None,
//             renderbuffer: None,
//             texture: None,
//         }
//     }

//     fn canvas_from_gl(&self) -> Result<HtmlCanvasElement, Error> {
//         self.gl
//             .canvas()
//             .ok_or(Error::CanvasNotFound)?
//             .dyn_into::<HtmlCanvasElement>()
//             .or(Err(Error::CanvasNotFound))
//     }

//     fn use_framebuffer(&mut self) -> Result<WebGlFramebuffer, Error> {
//         let framebuffer = &mut self.framebuffer;
//         let framebuffer = match framebuffer {
//             Some(framebuffer) => framebuffer.clone(),
//             None => {
//                 let fbo = self
//                     .gl
//                     .create_framebuffer()
//                     .ok_or(Error::CreateFramebufferFailure)?;
//                 *framebuffer = Some(fbo.clone());
//                 fbo
//             }
//         };

//         Ok(framebuffer)
//     }

//     fn use_depth_renderbuffer(&mut self) -> Result<WebGlRenderbuffer, Error> {
//         let canvas = self.canvas_from_gl()?;
//         let w = canvas.width();
//         let h = canvas.height();

//         if let Some((renderbuffer, width, height)) = &self.renderbuffer {
//             if w == *width || h == *height {
//                 return Ok(renderbuffer.clone());
//             }
//         }

//         let rb = self
//             .gl
//             .create_renderbuffer()
//             .ok_or(Error::CreateRenderbufferFailure)?;

//         self.gl
//             .bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, Some(&rb));
//         self.gl.renderbuffer_storage(
//             WebGl2RenderingContext::RENDERBUFFER,
//             WebGl2RenderingContext::DEPTH_COMPONENT16,
//             w as i32,
//             h as i32,
//         );
//         self.gl
//             .bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, None);

//         self.renderbuffer = Some((rb.clone(), w, h));

//         Ok(rb)
//     }

//     fn use_texture(&mut self) -> Result<WebGlTexture, Error> {
//         let canvas = self.canvas_from_gl()?;
//         let w = canvas.width();
//         let h = canvas.height();

//         if let Some((texture, width, height)) = &self.texture {
//             if w == *width || h == *height {
//                 return Ok(texture.clone());
//             }
//         }

//         let tx = self
//             .gl
//             .create_texture()
//             .ok_or(Error::CreateTextureFailure)?;

//         self.gl.active_texture(WebGl2RenderingContext::TEXTURE0);
//         self.gl
//             .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&tx));
//         self.gl
//             .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
//                 WebGl2RenderingContext::TEXTURE_2D,
//                 0,
//                 WebGl2RenderingContext::R32UI as i32,
//                 w as i32,
//                 h as i32,
//                 0,
//                 WebGl2RenderingContext::RED_INTEGER,
//                 WebGl2RenderingContext::UNSIGNED_INT,
//                 None,
//             )
//             .or_else(|err| Err(Error::TexImageFailure(err.as_string())))?;
//         self.gl
//             .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

//         self.texture = Some((tx.clone(), w, h));

//         Ok(tx)
//     }
// }

// impl PreProcessor for BindFramebuffer {
//     fn name(&self) -> &str {
//         "ReplaceFramebuffer"
//     }

//     fn pre_process(
//         &mut self,
//         state: &RenderState,
//         stuff: &mut dyn RenderStuff,
//     ) -> Result<(), Error> {
//         todo!()
//     }
// }

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

    fn attribute_value(&self, _: &str, _: &Rc<RefCell<Entity>>) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, name: &str, state: &Rc<RefCell<Entity>>) -> Option<UniformValue> {
        match name {
            "u_Index" => self.id2index.get(state.borrow().id()).cloned(),
            _ => None,
        }
    }

    fn ready(&self) -> bool {
        true
    }

    fn instanced(&self) -> Option<i32> {
        None
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
