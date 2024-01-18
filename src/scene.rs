use std::ptr::NonNull;

use gl_matrix4rust::vec3::Vec3;
use log::warn;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{
    HtmlCanvasElement, KeyboardEvent, MouseEvent, ResizeObserver, ResizeObserverEntry, WheelEvent,
};

use crate::{
    document,
    entity::Container,
    error::Error,
    event::EventAgency,
    light::{
        ambient_light::AmbientLight,
        area_light::{AreaLight, MAX_AREA_LIGHTS},
        directional_light::{DirectionalLight, MAX_DIRECTIONAL_LIGHTS},
        point_light::{PointLight, MAX_POINT_LIGHTS},
        spot_light::{SpotLight, MAX_SPOT_LIGHTS},
    },
};

pub struct Scene {
    canvas: HtmlCanvasElement,

    entity_container: Container,
    light_attenuations: Vec3::<f32>,
    ambient_light: Option<AmbientLight>,
    point_lights: Vec<PointLight>,
    directional_lights: Vec<DirectionalLight>,
    spot_lights: Vec<SpotLight>,
    area_lights: Vec<AreaLight>,

    // required for storing callback closure function
    resize_observer: Option<(ResizeObserver, Closure<dyn FnMut(Vec<ResizeObserverEntry>)>)>,
    select_start_callback: Option<Closure<dyn Fn() -> bool>>,
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
}

impl Scene {
    /// Constructs a new scene using initialization options.
    pub fn new() -> Result<Self, Error> {
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

        let mut scene = Self {
            canvas,

            entity_container: Container::new(),
            light_attenuations: Vec3::new(0.0, 1.0, 0.0),
            ambient_light: None,
            point_lights: Vec::new(),
            directional_lights: Vec::new(),
            spot_lights: Vec::new(),
            area_lights: Vec::new(),

            resize_observer: None,
            select_start_callback: None,
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
        };

        scene.observer_canvas_size()?;
        scene.register_callbacks()?;

        Ok(scene)
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
        let select_start_callback = Closure::new(|| false);
        self.canvas
            .set_onselectstart(Some(select_start_callback.as_ref().unchecked_ref()));

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

        self.select_start_callback = Some(select_start_callback);
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

    /// Returns root entities collection.
    pub fn entity_container(&self) -> &Container {
        &self.entity_container
    }

    /// Returns mutable root entities collection.
    pub fn entity_container_mut(&mut self) -> &mut Container {
        &mut self.entity_container
    }

    /// Returns ambient light.
    pub fn ambient_light(&self) -> Option<&AmbientLight> {
        self.ambient_light.as_ref()
    }

    /// Returns mutable ambient light.
    pub fn ambient_light_mut(&mut self) -> Option<&mut AmbientLight> {
        self.ambient_light.as_mut()
    }

    /// Sets ambient light.
    pub fn set_ambient_light(&mut self, light: Option<AmbientLight>) {
        self.ambient_light = light;
    }

    /// Returns lighting attenuation.
    pub fn light_attenuations(&self) -> &Vec3::<f32> {
        &self.light_attenuations
    }

    /// Sets lighting attenuation.
    pub fn set_light_attenuations(&mut self, attenuations: Vec3::<f32>) {
        self.light_attenuations = attenuations;
    }

    /// Adds a point light.
    pub fn add_point_light(&mut self, light: PointLight) {
        if self.point_lights.len() == MAX_POINT_LIGHTS {
            warn!(
                "only {} point lights are available, ignored",
                MAX_POINT_LIGHTS
            );
            return;
        }

        self.point_lights.push(light);
    }

    /// Removes a point light by index.
    pub fn remove_point_light(&mut self, index: usize) -> Option<PointLight> {
        if index < self.point_lights.len() {
            return None;
        }

        Some(self.point_lights.remove(index))
    }

    /// Returns point lights.
    pub fn point_lights(&self) -> &[PointLight] {
        &self.point_lights
    }

    /// Returns mutable point lights.
    pub fn point_lights_mut(&mut self) -> &mut [PointLight] {
        &mut self.point_lights
    }

    /// Returns a point light by index.
    pub fn point_light(&self, index: usize) -> Option<&PointLight> {
        self.point_lights.get(index)
    }

    /// Returns a mutable point light by index.
    pub fn point_light_mut(&mut self, index: usize) -> Option<&mut PointLight> {
        self.point_lights.get_mut(index)
    }

    /// Adds a directional light.
    pub fn add_directional_light(&mut self, light: DirectionalLight) {
        if self.directional_lights.len() == MAX_DIRECTIONAL_LIGHTS {
            warn!(
                "only {} directional lights are available, ignored",
                MAX_DIRECTIONAL_LIGHTS
            );
            return;
        }

        self.directional_lights.push(light);
    }

    /// Removes a directional light by index.
    pub fn remove_directional_light(&mut self, index: usize) -> Option<DirectionalLight> {
        if index < self.directional_lights.len() {
            return None;
        }

        Some(self.directional_lights.remove(index))
    }

    /// Returns directional lights.
    pub fn directional_lights(&self) -> &[DirectionalLight] {
        &self.directional_lights
    }

    /// Returns mutable directional lights.
    pub fn directional_lights_mut(&mut self) -> &mut [DirectionalLight] {
        &mut self.directional_lights
    }

    /// Returns a directional light by index.
    pub fn directional_light(&self, index: usize) -> Option<&DirectionalLight> {
        self.directional_lights.get(index)
    }

    /// Returns a mutable directional light by index.
    pub fn directional_light_mut(&mut self, index: usize) -> Option<&mut DirectionalLight> {
        self.directional_lights.get_mut(index)
    }

    /// Adds a spot light.
    pub fn add_spot_light(&mut self, light: SpotLight) {
        if self.spot_lights.len() == MAX_SPOT_LIGHTS {
            warn!(
                "only {} spot lights are available, ignored",
                MAX_SPOT_LIGHTS
            );
            return;
        }

        self.spot_lights.push(light);
    }

    /// Removes a spot light by index.
    pub fn remove_spot_light(&mut self, index: usize) -> Option<SpotLight> {
        if index < self.spot_lights.len() {
            return None;
        }

        Some(self.spot_lights.remove(index))
    }

    /// Returns spot lights.
    pub fn spot_lights(&self) -> &[SpotLight] {
        &self.spot_lights
    }

    /// Returns mutable spot lights.
    pub fn spot_lights_mut(&mut self) -> &mut [SpotLight] {
        &mut self.spot_lights
    }

    /// Returns a spot light by index.
    pub fn spot_light(&self, index: usize) -> Option<&SpotLight> {
        self.spot_lights.get(index)
    }

    /// Returns a mutable spot light by index.
    pub fn spot_light_mut(&mut self, index: usize) -> Option<&mut SpotLight> {
        self.spot_lights.get_mut(index)
    }

    /// Adds a area light.
    pub fn add_area_light(&mut self, light: AreaLight) {
        if self.spot_lights.len() == MAX_AREA_LIGHTS {
            warn!(
                "only {} area lights are available, ignored",
                MAX_AREA_LIGHTS
            );
            return;
        }

        self.area_lights.push(light);
    }

    /// Removes a area light by index.
    pub fn remove_area_light(&mut self, index: usize) -> Option<AreaLight> {
        if index < self.area_lights.len() {
            return None;
        }

        Some(self.area_lights.remove(index))
    }

    /// Returns area lights.
    pub fn area_lights(&self) -> &[AreaLight] {
        &self.area_lights
    }

    /// Returns mutable area lights.
    pub fn area_lights_mut(&mut self) -> &mut [AreaLight] {
        &mut self.area_lights
    }

    /// Returns a area light by index.
    pub fn area_light(&self, index: usize) -> Option<&AreaLight> {
        self.area_lights.get(index)
    }

    /// Returns a mutable area light by index.
    pub fn area_light_mut(&mut self, index: usize) -> Option<&mut AreaLight> {
        self.area_lights.get_mut(index)
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
}

impl Drop for Scene {
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
