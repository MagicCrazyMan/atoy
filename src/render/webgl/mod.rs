use std::ptr::NonNull;

use gl_matrix4rust::{mat4::AsMat4, vec3::AsVec3};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{
    js_sys::{ArrayBuffer, Float32Array},
    HtmlCanvasElement, KeyboardEvent, MouseEvent, ResizeObserver, ResizeObserverEntry,
    WebGl2RenderingContext, WheelEvent,
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

use super::{
    pp::{Pipeline, State},
    Render,
};

pub mod attribute;
pub mod buffer;
pub mod client_wait;
pub mod conversion;
pub mod draw;
pub mod error;
pub mod framebuffer;
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
    gl: WebGl2RenderingContext,
    canvas: HtmlCanvasElement,
    gamma: f64,
    universal_ubo: BufferDescriptor,
    lights_ubo: BufferDescriptor,
    program_store: ProgramStore,
    buffer_store: BufferStore,
    texture_store: TextureStore,

    // required for storing callback closure function
    resize_observer: Option<(ResizeObserver, Closure<dyn FnMut(Vec<ResizeObserverEntry>)>)>,
    click_callback: Option<Closure<dyn FnMut(MouseEvent)>>,
    double_click_callback: Option<Closure<dyn FnMut(MouseEvent)>>,
    mouse_down_callback: Option<Closure<dyn FnMut(MouseEvent)>>,
    mouse_enter_callback: Option<Closure<dyn FnMut(MouseEvent)>>,
    mouse_leave_callback: Option<Closure<dyn FnMut(MouseEvent)>>,
    mouse_move_callback: Option<Closure<dyn FnMut(MouseEvent)>>,
    mouse_out_callback: Option<Closure<dyn FnMut(MouseEvent)>>,
    mouse_over_callback: Option<Closure<dyn FnMut(MouseEvent)>>,
    mouse_up_callback: Option<Closure<dyn FnMut(MouseEvent)>>,
    wheel_callback: Option<Closure<dyn FnMut(WheelEvent)>>,
    key_down_callback: Option<Closure<dyn FnMut(KeyboardEvent)>>,
    key_up_callback: Option<Closure<dyn FnMut(KeyboardEvent)>>,

    click_event: EventAgency<MouseEvent>,
    double_click_event: EventAgency<MouseEvent>,
    mouse_down_event: EventAgency<MouseEvent>,
    mouse_enter_event: EventAgency<MouseEvent>,
    mouse_leave_event: EventAgency<MouseEvent>,
    mouse_move_event: EventAgency<MouseEvent>,
    mouse_out_event: EventAgency<MouseEvent>,
    mouse_over_event: EventAgency<MouseEvent>,
    mouse_up_event: EventAgency<MouseEvent>,
    wheel_event: EventAgency<WheelEvent>,
    key_down_event: EventAgency<KeyboardEvent>,
    key_up_event: EventAgency<KeyboardEvent>,
    canvas_changed_event: EventAgency<CanvasChangedEvent>,
    pre_render_event: EventAgency<RenderEvent>,
    post_render_event: EventAgency<RenderEvent>,
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

            resize_observer: None,
            click_callback: None,
            double_click_callback: None,
            mouse_down_callback: None,
            mouse_enter_callback: None,
            mouse_leave_callback: None,
            mouse_move_callback: None,
            mouse_out_callback: None,
            mouse_over_callback: None,
            mouse_up_callback: None,
            wheel_callback: None,
            key_down_callback: None,
            key_up_callback: None,

            click_event: EventAgency::new(),
            double_click_event: EventAgency::new(),
            mouse_down_event: EventAgency::new(),
            mouse_enter_event: EventAgency::new(),
            mouse_leave_event: EventAgency::new(),
            mouse_move_event: EventAgency::new(),
            mouse_out_event: EventAgency::new(),
            mouse_over_event: EventAgency::new(),
            mouse_up_event: EventAgency::new(),
            wheel_event: EventAgency::new(),
            key_down_event: EventAgency::new(),
            key_up_event: EventAgency::new(),
            canvas_changed_event: EventAgency::new(),
            pre_render_event: EventAgency::new(),
            post_render_event: EventAgency::new(),
        };

        render.observer_canvas_size()?;
        render.register_callbacks()?;

        Ok(render)
    }

    fn observer_canvas_size(&mut self) -> Result<(), Error> {
        let event = self.canvas_changed_event.clone();
        // create observer observing size change event of canvas
        let resize_observer_callback = Closure::new(move |entries: Vec<ResizeObserverEntry>| {
            // should have only one entry
            let Some(target) = entries.get(0).map(|entry| entry.target()) else {
                return;
            };
            let Ok(mut canvas) = target.dyn_into::<HtmlCanvasElement>() else {
                return;
            };

            canvas.set_width(canvas.client_width() as u32);
            canvas.set_height(canvas.client_height() as u32);
            event.raise(CanvasChangedEvent::new(&mut canvas));
        });

        let resize_observer =
            ResizeObserver::new(resize_observer_callback.as_ref().unchecked_ref())
                .or_else(|err| Err(Error::CanvasResizeObserverFailed(err.as_string())))?;
        resize_observer.observe(&self.canvas);

        self.resize_observer = Some((resize_observer, resize_observer_callback));

        Ok(())
    }

    fn register_callbacks(&mut self) -> Result<(), Error> {
        let click_event = self.click_event.clone();
        let click_callback = Closure::new(move |e| click_event.raise(e));
        self.canvas
            .add_event_listener_with_callback("click", click_callback.as_ref().unchecked_ref())
            .or_else(|err| Err(Error::AddEventCallbackFailed("click", err.as_string())))?;

        let double_click_event = self.double_click_event.clone();
        let double_click_callback = Closure::new(move |e| double_click_event.raise(e));
        self.canvas
            .add_event_listener_with_callback(
                "dbclick",
                double_click_callback.as_ref().unchecked_ref(),
            )
            .or_else(|err| Err(Error::AddEventCallbackFailed("dbclick", err.as_string())))?;

        let mouse_down_event = self.mouse_down_event.clone();
        let mouse_down_callback = Closure::new(move |e| mouse_down_event.raise(e));
        self.canvas
            .add_event_listener_with_callback(
                "mousedown",
                mouse_down_callback.as_ref().unchecked_ref(),
            )
            .or_else(|err| Err(Error::AddEventCallbackFailed("mousedown", err.as_string())))?;

        let mouse_enter_event = self.mouse_enter_event.clone();
        let mouse_enter_callback = Closure::new(move |e| mouse_enter_event.raise(e));
        self.canvas
            .add_event_listener_with_callback(
                "mouseenter",
                mouse_enter_callback.as_ref().unchecked_ref(),
            )
            .or_else(|err| Err(Error::AddEventCallbackFailed("mouseenter", err.as_string())))?;

        let mouse_leave_event = self.mouse_leave_event.clone();
        let mouse_leave_callback = Closure::new(move |e| mouse_leave_event.raise(e));
        self.canvas
            .add_event_listener_with_callback(
                "mouseleave",
                mouse_leave_callback.as_ref().unchecked_ref(),
            )
            .or_else(|err| Err(Error::AddEventCallbackFailed("mouseleave", err.as_string())))?;

        let mouse_move_event = self.mouse_move_event.clone();
        let mouse_move_callback = Closure::new(move |e| mouse_move_event.raise(e));
        self.canvas
            .add_event_listener_with_callback(
                "mousemove",
                mouse_move_callback.as_ref().unchecked_ref(),
            )
            .or_else(|err| Err(Error::AddEventCallbackFailed("mousemove", err.as_string())))?;

        let mouse_out_event = self.mouse_out_event.clone();
        let mouse_out_callback = Closure::new(move |e| mouse_out_event.raise(e));
        self.canvas
            .add_event_listener_with_callback(
                "mouseout",
                mouse_out_callback.as_ref().unchecked_ref(),
            )
            .or_else(|err| Err(Error::AddEventCallbackFailed("mouseout", err.as_string())))?;

        let mouse_over_event = self.mouse_over_event.clone();
        let mouse_over_callback = Closure::new(move |e| mouse_over_event.raise(e));
        self.canvas
            .add_event_listener_with_callback(
                "mouseover",
                mouse_over_callback.as_ref().unchecked_ref(),
            )
            .or_else(|err| Err(Error::AddEventCallbackFailed("mouseover", err.as_string())))?;

        let mouse_up_event = self.mouse_up_event.clone();
        let mouse_up_callback = Closure::new(move |e| mouse_up_event.raise(e));
        self.canvas
            .add_event_listener_with_callback("mouseup", mouse_up_callback.as_ref().unchecked_ref())
            .or_else(|err| Err(Error::AddEventCallbackFailed("mouseup", err.as_string())))?;

        let wheel_event = self.wheel_event.clone();
        let wheel_callback = Closure::new(move |e| wheel_event.raise(e));
        self.canvas
            .add_event_listener_with_callback("wheel", wheel_callback.as_ref().unchecked_ref())
            .or_else(|err| Err(Error::AddEventCallbackFailed("wheel", err.as_string())))?;

        let key_down_event = self.key_down_event.clone();
        let key_down_callback = Closure::new(move |e| key_down_event.raise(e));
        self.canvas
            .add_event_listener_with_callback("keydown", key_down_callback.as_ref().unchecked_ref())
            .or_else(|err| Err(Error::AddEventCallbackFailed("keydown", err.as_string())))?;

        let key_up_event = self.key_up_event.clone();
        let key_up_callback = Closure::new(move |e| key_up_event.raise(e));
        self.canvas
            .add_event_listener_with_callback("keyup", key_up_callback.as_ref().unchecked_ref())
            .or_else(|err| Err(Error::AddEventCallbackFailed("keyup", err.as_string())))?;

        self.click_callback = Some(click_callback);
        self.double_click_callback = Some(double_click_callback);
        self.mouse_down_callback = Some(mouse_down_callback);
        self.mouse_enter_callback = Some(mouse_enter_callback);
        self.mouse_leave_callback = Some(mouse_leave_callback);
        self.mouse_move_callback = Some(mouse_move_callback);
        self.mouse_out_callback = Some(mouse_out_callback);
        self.mouse_over_callback = Some(mouse_over_callback);
        self.mouse_up_callback = Some(mouse_up_callback);
        self.wheel_callback = Some(wheel_callback);
        self.key_down_callback = Some(key_down_callback);
        self.key_up_callback = Some(key_up_callback);

        Ok(())
    }

    /// Returns [`HtmlCanvasElement`].
    pub fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
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

    pub fn click_event(&mut self) -> &mut EventAgency<MouseEvent> {
        &mut self.click_event
    }

    pub fn double_click_event(&mut self) -> &mut EventAgency<MouseEvent> {
        &mut self.double_click_event
    }

    pub fn mouse_down_event(&mut self) -> &mut EventAgency<MouseEvent> {
        &mut self.mouse_down_event
    }

    pub fn mouse_enter_event(&mut self) -> &mut EventAgency<MouseEvent> {
        &mut self.mouse_enter_event
    }

    pub fn mouse_leave_event(&mut self) -> &mut EventAgency<MouseEvent> {
        &mut self.mouse_leave_event
    }

    pub fn mouse_move_event(&mut self) -> &mut EventAgency<MouseEvent> {
        &mut self.mouse_move_event
    }

    pub fn mouse_out_event(&mut self) -> &mut EventAgency<MouseEvent> {
        &mut self.mouse_out_event
    }

    pub fn mouse_over_event(&mut self) -> &mut EventAgency<MouseEvent> {
        &mut self.mouse_over_event
    }

    pub fn mouse_up_event(&mut self) -> &mut EventAgency<MouseEvent> {
        &mut self.mouse_up_event
    }

    pub fn wheel_event(&mut self) -> &mut EventAgency<WheelEvent> {
        &mut self.wheel_event
    }

    pub fn key_down_event(&mut self) -> &mut EventAgency<KeyboardEvent> {
        &mut self.key_down_event
    }

    pub fn key_up_event(&mut self) -> &mut EventAgency<KeyboardEvent> {
        &mut self.key_up_event
    }

    pub fn canvas_changed_event(&mut self) -> &mut EventAgency<CanvasChangedEvent> {
        &mut self.canvas_changed_event
    }

    pub fn pre_render_event(&mut self) -> &mut EventAgency<RenderEvent> {
        &mut self.pre_render_event
    }

    pub fn post_render_event(&mut self) -> &mut EventAgency<RenderEvent> {
        &mut self.post_render_event
    }
}

impl Render for WebGL2Render {
    type Error = Error;

    fn render(
        &mut self,
        pipeline: &mut (dyn Pipeline<Error = Self::Error> + 'static),
        camera: &mut (dyn Camera + 'static),
        scene: &mut Scene,
        timestamp: f64,
    ) -> Result<(), Self::Error> {
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

        self.pre_render_event.raise(RenderEvent::new(&mut state));
        pipeline.execute(&mut state, scene)?;
        self.post_render_event.raise(RenderEvent::new(&mut state));

        Ok(())
    }
}

impl Drop for WebGL2Render {
    fn drop(&mut self) {
        // cleanups observers
        if let Some((observer, _)) = self.resize_observer.take() {
            observer.disconnect();
        }

        if let Some(callback) = self.click_callback.take() {
            let _ = self
                .canvas
                .remove_event_listener_with_callback("click", callback.as_ref().unchecked_ref());
        }
        if let Some(callback) = self.double_click_callback.take() {
            let _ = self
                .canvas
                .remove_event_listener_with_callback("dbclick", callback.as_ref().unchecked_ref());
        }
        if let Some(callback) = self.mouse_down_callback.take() {
            let _ = self.canvas.remove_event_listener_with_callback(
                "mousedown",
                callback.as_ref().unchecked_ref(),
            );
        }
        if let Some(callback) = self.mouse_enter_callback.take() {
            let _ = self.canvas.remove_event_listener_with_callback(
                "mouseenter",
                callback.as_ref().unchecked_ref(),
            );
        }
        if let Some(callback) = self.mouse_leave_callback.take() {
            let _ = self.canvas.remove_event_listener_with_callback(
                "mouseleave",
                callback.as_ref().unchecked_ref(),
            );
        }
        if let Some(callback) = self.mouse_move_callback.take() {
            let _ = self.canvas.remove_event_listener_with_callback(
                "mousemove",
                callback.as_ref().unchecked_ref(),
            );
        }
        if let Some(callback) = self.mouse_out_callback.take() {
            let _ = self
                .canvas
                .remove_event_listener_with_callback("mouseout", callback.as_ref().unchecked_ref());
        }
        if let Some(callback) = self.mouse_over_callback.take() {
            let _ = self.canvas.remove_event_listener_with_callback(
                "mouseover",
                callback.as_ref().unchecked_ref(),
            );
        }
        if let Some(callback) = self.mouse_up_callback.take() {
            let _ = self
                .canvas
                .remove_event_listener_with_callback("mouseup", callback.as_ref().unchecked_ref());
        }
        if let Some(callback) = self.wheel_callback.take() {
            let _ = self
                .canvas
                .remove_event_listener_with_callback("wheel", callback.as_ref().unchecked_ref());
        }
        if let Some(callback) = self.key_down_callback.take() {
            let _ = self
                .canvas
                .remove_event_listener_with_callback("keydown", callback.as_ref().unchecked_ref());
        }
        if let Some(callback) = self.key_up_callback.take() {
            let _ = self
                .canvas
                .remove_event_listener_with_callback("keyup", callback.as_ref().unchecked_ref());
        }
    }
}

pub struct CanvasChangedEvent(NonNull<HtmlCanvasElement>);

impl CanvasChangedEvent {
    fn new(canvas: &mut HtmlCanvasElement) -> Self {
        Self(unsafe { NonNull::new_unchecked(canvas) })
    }

    pub fn canvas(&self) -> &HtmlCanvasElement {
        unsafe { self.0.as_ref() }
    }
}

pub struct RenderEvent(NonNull<State>);

impl RenderEvent {
    fn new(state: &mut State) -> Self {
        Self(unsafe { NonNull::new_unchecked(state) })
    }

    pub fn state(&self) -> &State {
        unsafe { self.0.as_ref() }
    }

    pub fn state_mut(&mut self) -> &mut State {
        unsafe { self.0.as_mut() }
    }
}
