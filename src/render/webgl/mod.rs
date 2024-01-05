use std::ptr::NonNull;

use gl_matrix4rust::{mat4::AsMat4, vec3::AsVec3};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{
    js_sys::{ArrayBuffer, Float32Array},
    HtmlCanvasElement, ResizeObserver, ResizeObserverEntry, WebGl2RenderingContext,
};

use crate::{
    camera::Camera, document, event::EventAgency,
    render::webgl::uniform::UBO_UNIVERSAL_UNIFORMS_ENABLE_LIGHTING_BYTES_LENGTH, scene::Scene,
};

use self::{
    buffer::{BufferDescriptor, BufferSource, BufferStore, BufferUsage, MemoryPolicy},
    error::Error,
    program::ProgramStore,
    texture::TextureStore,
    uniform::{
        UBO_LIGHTS_AMBIENT_LIGHT_BYTES_LENGTH, UBO_LIGHTS_AMBIENT_LIGHT_BYTES_OFFSET,
        UBO_LIGHTS_AREA_LIGHTS_BYTES_LENGTH, UBO_LIGHTS_AREA_LIGHTS_BYTES_OFFSET,
        UBO_LIGHTS_ATTENUATIONS_BYTES_LENGTH, UBO_LIGHTS_ATTENUATIONS_BYTES_OFFSET,
        UBO_LIGHTS_BYTES_LENGTH, UBO_LIGHTS_DIRECTIONAL_LIGHTS_BYTES_LENGTH,
        UBO_LIGHTS_DIRECTIONAL_LIGHTS_BYTES_OFFSET, UBO_LIGHTS_POINT_LIGHTS_BYTES_LENGTH,
        UBO_LIGHTS_POINT_LIGHTS_BYTES_OFFSET, UBO_LIGHTS_SPOT_LIGHTS_BYTES_LENGTH,
        UBO_LIGHTS_SPOT_LIGHTS_BYTES_OFFSET, UBO_UNIVERSAL_UNIFORMS_BYTES_LENGTH,
        UBO_UNIVERSAL_UNIFORMS_CAMERA_POSITION_BYTES_LENGTH,
        UBO_UNIVERSAL_UNIFORMS_CAMERA_POSITION_BYTES_OFFSET,
        UBO_UNIVERSAL_UNIFORMS_ENABLE_LIGHTING_BYTES_OFFSET,
        UBO_UNIVERSAL_UNIFORMS_GAMMA_CORRECTION_BYTES_LENGTH,
        UBO_UNIVERSAL_UNIFORMS_GAMMA_CORRECTION_BYTES_OFFSET,
        UBO_UNIVERSAL_UNIFORMS_GAMMA_CORRECTION_INVERSE_BYTES_LENGTH,
        UBO_UNIVERSAL_UNIFORMS_GAMMA_CORRECTION_INVERSE_BYTES_OFFSET,
        UBO_UNIVERSAL_UNIFORMS_PROJ_MATRIX_BYTES_LENGTH,
        UBO_UNIVERSAL_UNIFORMS_PROJ_MATRIX_BYTES_OFFSET,
        UBO_UNIVERSAL_UNIFORMS_RENDER_TIME_BYTES_LENGTH,
        UBO_UNIVERSAL_UNIFORMS_RENDER_TIME_BYTES_OFFSET,
        UBO_UNIVERSAL_UNIFORMS_VIEW_MATRIX_BYTES_LENGTH,
        UBO_UNIVERSAL_UNIFORMS_VIEW_MATRIX_BYTES_OFFSET,
        UBO_UNIVERSAL_UNIFORMS_VIEW_PROJ_MATRIX_BYTES_LENGTH,
        UBO_UNIVERSAL_UNIFORMS_VIEW_PROJ_MATRIX_BYTES_OFFSET,
    },
};

use super::pp::{Pipeline, State};

pub mod attribute;
pub mod buffer;
pub mod conversion;
pub mod draw;
pub mod error;
pub mod offscreen;
pub mod pipeline;
pub mod program;
pub mod renderbuffer;
pub mod shader;
pub mod stencil;
pub mod texture;
pub mod uniform;

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
    // required for storing callback closure function
    resize_observer: Option<(ResizeObserver, Closure<dyn FnMut(Vec<ResizeObserverEntry>)>)>,
    gl: WebGl2RenderingContext,
    canvas: HtmlCanvasElement,
    gamma: f64,
    universal_ubo: BufferDescriptor,
    lights_ubo: BufferDescriptor,
    program_store: ProgramStore,
    buffer_store: BufferStore,
    texture_store: TextureStore,
    event: EventAgency<Event>,
}

impl WebGL2Render {
    /// Constructs a new WebGL2 render.
    pub fn new(options: Option<WebGL2ContextOptions>) -> Result<Self, Error> {
        let canvas = document()
            .create_element("canvas")
            .ok()
            .and_then(|ele| ele.dyn_into::<HtmlCanvasElement>().ok())
            .ok_or(Error::CreateCanvasFailed)?;
        canvas
            .set_attribute("tabindex", "0")
            .map_err(|_| Error::CreateCanvasFailed)?;
        canvas
            .style()
            .set_css_text("width: 100%; height: 100%; outline: none;");

        let options = options.unwrap_or(WebGL2ContextOptions::default());
        let gl = canvas
            .get_context_with_context_options(
                "webgl2",
                &serde_wasm_bindgen::to_value(&options).unwrap(),
            )
            .ok()
            .and_then(|context| context)
            .and_then(|context| context.dyn_into::<WebGl2RenderingContext>().ok())
            .ok_or(Error::WebGL2Unsupported)?;

        let mut render = Self {
            resize_observer: None,
            universal_ubo: BufferDescriptor::with_memory_policy(
                BufferSource::preallocate(UBO_UNIVERSAL_UNIFORMS_BYTES_LENGTH as i32),
                BufferUsage::DynamicDraw,
                MemoryPolicy::Unfree,
            ),
            lights_ubo: BufferDescriptor::with_memory_policy(
                BufferSource::preallocate(UBO_LIGHTS_BYTES_LENGTH as i32),
                BufferUsage::DynamicDraw,
                MemoryPolicy::Unfree,
            ),
            gamma: 2.2,
            program_store: ProgramStore::new(gl.clone()),
            buffer_store: BufferStore::new(gl.clone()),
            // buffer_store: BufferStore::with_max_memory(gl.clone(), 1000),
            texture_store: TextureStore::new(gl.clone()),
            gl,
            canvas,
            event: EventAgency::new(),
        };

        render.observer_canvas_size();

        Ok(render)
    }

    fn observer_canvas_size(&mut self) {
        let mut event = self.event.clone();
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
            event.raise(Event::ChangeCanvasSize(
                canvas.client_width() as u32,
                canvas.client_height() as u32,
            ));
        });

        let resize_observer =
            ResizeObserver::new(resize_observer_callback.as_ref().unchecked_ref()).unwrap();
        resize_observer.observe(&self.canvas);

        self.resize_observer = Some((resize_observer, resize_observer_callback))
    }

    /// Returns [`HtmlCanvasElement`].
    pub fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }

    /// Returns event agency.
    pub fn event(&mut self) -> &mut EventAgency<Event> {
        &mut self.event
    }
}

impl WebGL2Render {
    /// Renders a frame with stuff and a pipeline.
    pub fn render<C, P, E>(
        &mut self,
        pipeline: &mut P,
        camera: &mut C,
        scene: &mut Scene,
        timestamp: f64,
    ) -> Result<(), E>
    where
        C: Camera + 'static,
        P: Pipeline<Error = E>,
    {
        self.update_universal_ubo(camera, scene, timestamp);
        self.update_lights_ubo(scene);

        let mut state = State::new(
            timestamp,
            camera,
            &mut self.gl,
            &mut self.canvas,
            &mut self.universal_ubo,
            &mut self.lights_ubo,
            &mut self.program_store,
            &mut self.buffer_store,
            &mut self.texture_store,
        );

        self.event
            .raise(unsafe { Event::PreRender(NonNull::new_unchecked(&mut state)) });
        pipeline.execute(&mut state, scene)?;
        self.event
            .raise(unsafe { Event::PostRender(NonNull::new_unchecked(&mut state)) });

        Ok(())
    }

    fn update_universal_ubo(&mut self, camera: &dyn Camera, scene: &mut Scene, timestamp: f64) {
        let data = ArrayBuffer::new(UBO_UNIVERSAL_UNIFORMS_BYTES_LENGTH);

        // u_RenderTime
        Float32Array::new_with_byte_offset_and_length(
            &data,
            UBO_UNIVERSAL_UNIFORMS_RENDER_TIME_BYTES_OFFSET,
            UBO_UNIVERSAL_UNIFORMS_RENDER_TIME_BYTES_LENGTH / 4,
        )
        .set_index(0, timestamp as f32);

        // u_EnableLighting
        Float32Array::new_with_byte_offset_and_length(
            &data,
            UBO_UNIVERSAL_UNIFORMS_ENABLE_LIGHTING_BYTES_OFFSET,
            UBO_UNIVERSAL_UNIFORMS_ENABLE_LIGHTING_BYTES_LENGTH / 4,
        )
        .set_index(0, if scene.lighting_enabled() { 1.0 } else { 0.0 });

        // u_GammaCorrection
        Float32Array::new_with_byte_offset_and_length(
            &data,
            UBO_UNIVERSAL_UNIFORMS_GAMMA_CORRECTION_BYTES_OFFSET,
            UBO_UNIVERSAL_UNIFORMS_GAMMA_CORRECTION_BYTES_LENGTH / 4,
        )
        .set_index(0, (1.0 / self.gamma) as f32);

        // u_GammaCorrectionInverse
        Float32Array::new_with_byte_offset_and_length(
            &data,
            UBO_UNIVERSAL_UNIFORMS_GAMMA_CORRECTION_INVERSE_BYTES_OFFSET,
            UBO_UNIVERSAL_UNIFORMS_GAMMA_CORRECTION_INVERSE_BYTES_LENGTH / 4,
        )
        .set_index(0, self.gamma as f32);

        // u_CameraPosition
        Float32Array::new_with_byte_offset_and_length(
            &data,
            UBO_UNIVERSAL_UNIFORMS_CAMERA_POSITION_BYTES_OFFSET,
            UBO_UNIVERSAL_UNIFORMS_CAMERA_POSITION_BYTES_LENGTH / 4,
        )
        .copy_from(&camera.position().to_gl());

        // u_ViewMatrix
        Float32Array::new_with_byte_offset_and_length(
            &data,
            UBO_UNIVERSAL_UNIFORMS_VIEW_MATRIX_BYTES_OFFSET,
            UBO_UNIVERSAL_UNIFORMS_VIEW_MATRIX_BYTES_LENGTH / 4,
        )
        .copy_from(&camera.view_matrix().to_gl());

        // u_ProjMatrix
        Float32Array::new_with_byte_offset_and_length(
            &data,
            UBO_UNIVERSAL_UNIFORMS_PROJ_MATRIX_BYTES_OFFSET,
            UBO_UNIVERSAL_UNIFORMS_PROJ_MATRIX_BYTES_LENGTH / 4,
        )
        .copy_from(&camera.proj_matrix().to_gl());

        // u_ProjViewMatrix
        Float32Array::new_with_byte_offset_and_length(
            &data,
            UBO_UNIVERSAL_UNIFORMS_VIEW_PROJ_MATRIX_BYTES_OFFSET,
            UBO_UNIVERSAL_UNIFORMS_VIEW_PROJ_MATRIX_BYTES_LENGTH / 4,
        )
        .copy_from(&camera.view_proj_matrix().to_gl());

        self.universal_ubo
            .buffer_sub_data(BufferSource::from_array_buffer(data), 0);
    }

    fn update_lights_ubo(&mut self, scene: &mut Scene) {
        let data = ArrayBuffer::new(UBO_LIGHTS_BYTES_LENGTH);

        // u_Attenuations
        Float32Array::new_with_byte_offset_and_length(
            &data,
            UBO_LIGHTS_ATTENUATIONS_BYTES_OFFSET,
            UBO_LIGHTS_ATTENUATIONS_BYTES_LENGTH / 4,
        )
        .copy_from(&scene.light_attenuations().to_gl());

        // u_AmbientLight
        if let Some(light) = scene.ambient_light() {
            Float32Array::new_with_byte_offset_and_length(
                &data,
                UBO_LIGHTS_AMBIENT_LIGHT_BYTES_OFFSET,
                UBO_LIGHTS_AMBIENT_LIGHT_BYTES_LENGTH / 4,
            )
            .copy_from(&light.gl_ubo());
        }

        // u_DirectionalLights
        for (index, light) in scene.directional_lights().into_iter().enumerate() {
            let index = index as u32;
            Float32Array::new_with_byte_offset_and_length(
                &data,
                UBO_LIGHTS_DIRECTIONAL_LIGHTS_BYTES_OFFSET
                    + index * UBO_LIGHTS_DIRECTIONAL_LIGHTS_BYTES_LENGTH,
                UBO_LIGHTS_DIRECTIONAL_LIGHTS_BYTES_LENGTH / 4,
            )
            .copy_from(&light.gl_ubo());
        }

        // u_PointLights
        for (index, light) in scene.point_lights().into_iter().enumerate() {
            let index = index as u32;
            Float32Array::new_with_byte_offset_and_length(
                &data,
                UBO_LIGHTS_POINT_LIGHTS_BYTES_OFFSET + index * UBO_LIGHTS_POINT_LIGHTS_BYTES_LENGTH,
                UBO_LIGHTS_POINT_LIGHTS_BYTES_LENGTH / 4,
            )
            .copy_from(&light.gl_ubo());
        }

        // u_SpotLights
        for (index, light) in scene.spot_lights().into_iter().enumerate() {
            let index = index as u32;
            Float32Array::new_with_byte_offset_and_length(
                &data,
                UBO_LIGHTS_SPOT_LIGHTS_BYTES_OFFSET + index * UBO_LIGHTS_SPOT_LIGHTS_BYTES_LENGTH,
                UBO_LIGHTS_SPOT_LIGHTS_BYTES_LENGTH / 4,
            )
            .copy_from(&light.gl_ubo());
        }

        // u_AreaLights
        for (index, light) in scene.area_lights().into_iter().enumerate() {
            let index = index as u32;
            Float32Array::new_with_byte_offset_and_length(
                &data,
                UBO_LIGHTS_AREA_LIGHTS_BYTES_OFFSET + index * UBO_LIGHTS_AREA_LIGHTS_BYTES_LENGTH,
                UBO_LIGHTS_AREA_LIGHTS_BYTES_LENGTH / 4,
            )
            .copy_from(&light.gl_ubo());
        }

        self.lights_ubo
            .buffer_sub_data(BufferSource::from_array_buffer(data), 0);
    }
}

impl Drop for WebGL2Render {
    fn drop(&mut self) {
        // cleanups observers
        if let Some((observer, _)) = &self.resize_observer {
            observer.disconnect();
        }
    }
}

#[derive(Clone)]
pub enum Event {
    ChangeCanvasSize(u32, u32),
    PreRender(NonNull<State>),
    PostRender(NonNull<State>),
}
