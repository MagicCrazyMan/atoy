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
    light::{
        ambient_light::AmbientLight,
        area_light::{AreaLight, MAX_AREA_LIGHTS},
        directional_light::{DirectionalLight, MAX_DIRECTIONAL_LIGHTS},
        point_light::{PointLight, MAX_POINT_LIGHTS},
        spot_light::{SpotLight, MAX_SPOT_LIGHTS},
    },
    notify::Notifier,
};

pub struct Scene {
    canvas: HtmlCanvasElement,
    canvas_handler: SceneCanvasHandler,
    _select_start_callback: Closure<dyn Fn() -> bool>,

    entity_container: Container,
    light_attenuations: Vec3<f32>,
    ambient_light: Option<AmbientLight>,
    point_lights: Vec<PointLight>,
    directional_lights: Vec<DirectionalLight>,
    spot_lights: Vec<SpotLight>,
    area_lights: Vec<AreaLight>,
}

impl Scene {
    /// Constructs a new scene using initialization options.
    pub fn new() -> Result<Self, Error> {
        let canvas = document()
            .create_element("canvas")
            .ok()
            .and_then(|ele| ele.dyn_into::<HtmlCanvasElement>().ok())
            .ok_or(Error::CreateCanvasFailure)?;
        canvas
            .set_attribute("tabindex", "0")
            .map_err(|_| Error::CreateCanvasFailure)?;
        canvas
            .style()
            .set_css_text("width: 100%; height: 100%; outline: none;");

        let select_start_callback = Closure::new(|| false);
        canvas.set_onselectstart(Some(select_start_callback.as_ref().unchecked_ref()));

        Ok(Self {
            canvas_handler: SceneCanvasHandler::new(canvas.clone())?,
            _select_start_callback: select_start_callback,
            canvas,

            entity_container: Container::new(),
            light_attenuations: Vec3::new(0.0, 1.0, 0.0),
            ambient_light: None,
            point_lights: Vec::new(),
            directional_lights: Vec::new(),
            spot_lights: Vec::new(),
            area_lights: Vec::new(),
        })
    }

    /// Returns [`HtmlCanvasElement`].
    pub fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }

    pub fn canvas_handler(&mut self) -> &mut SceneCanvasHandler {
        &mut self.canvas_handler
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
    pub fn set_ambient_light(&mut self, mut light: Option<AmbientLight>) {
        if let Some(light) = light.as_mut() {
            light.set_ubo_dirty();
        }
        self.ambient_light = light;
    }

    /// Returns lighting attenuation.
    pub fn light_attenuations(&self) -> &Vec3<f32> {
        &self.light_attenuations
    }

    /// Sets lighting attenuation.
    pub fn set_light_attenuations(&mut self, attenuations: Vec3<f32>) {
        self.light_attenuations = attenuations;
    }

    /// Adds a point light.
    pub fn add_point_light(&mut self, mut light: PointLight) {
        if self.point_lights.len() == MAX_POINT_LIGHTS {
            warn!(
                "only {} point lights are available, ignored",
                MAX_POINT_LIGHTS
            );
            return;
        }

        light.set_ubo_dirty();
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
    pub fn add_directional_light(&mut self, mut light: DirectionalLight) {
        if self.directional_lights.len() == MAX_DIRECTIONAL_LIGHTS {
            warn!(
                "only {} directional lights are available, ignored",
                MAX_DIRECTIONAL_LIGHTS
            );
            return;
        }

        light.set_ubo_dirty();
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
    pub fn add_spot_light(&mut self, mut light: SpotLight) {
        if self.spot_lights.len() == MAX_SPOT_LIGHTS {
            warn!(
                "only {} spot lights are available, ignored",
                MAX_SPOT_LIGHTS
            );
            return;
        }

        light.set_ubo_dirty();
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
    pub fn add_area_light(&mut self, mut light: AreaLight) {
        if self.spot_lights.len() == MAX_AREA_LIGHTS {
            warn!(
                "only {} area lights are available, ignored",
                MAX_AREA_LIGHTS
            );
            return;
        }

        light.set_ubo_dirty();
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
}

pub struct SceneCanvasHandler {
    canvas: HtmlCanvasElement,
    canvas_resize: (
        Notifier<HtmlCanvasElement>,
        ResizeObserver,
        Closure<dyn FnMut(Vec<ResizeObserverEntry>)>,
    ),
    click: (Notifier<MouseEvent>, Closure<dyn FnMut(MouseEvent)>),
    double_click: (Notifier<MouseEvent>, Closure<dyn FnMut(MouseEvent)>),
    mouse_down: (Notifier<MouseEvent>, Closure<dyn FnMut(MouseEvent)>),
    mouse_enter: (Notifier<MouseEvent>, Closure<dyn FnMut(MouseEvent)>),
    mouse_leave: (Notifier<MouseEvent>, Closure<dyn FnMut(MouseEvent)>),
    mouse_move: (Notifier<MouseEvent>, Closure<dyn FnMut(MouseEvent)>),
    mouse_out: (Notifier<MouseEvent>, Closure<dyn FnMut(MouseEvent)>),
    mouse_over: (Notifier<MouseEvent>, Closure<dyn FnMut(MouseEvent)>),
    mouse_up: (Notifier<MouseEvent>, Closure<dyn FnMut(MouseEvent)>),
    wheel: (Notifier<WheelEvent>, Closure<dyn FnMut(WheelEvent)>),
    key_down: (Notifier<KeyboardEvent>, Closure<dyn FnMut(KeyboardEvent)>),
    key_up: (Notifier<KeyboardEvent>, Closure<dyn FnMut(KeyboardEvent)>),
}

impl Drop for SceneCanvasHandler {
    fn drop(&mut self) {
        self.canvas_resize.1.disconnect();

        let _ = self
            .canvas
            .remove_event_listener_with_callback("click", self.click.1.as_ref().unchecked_ref());

        let _ = self.canvas.remove_event_listener_with_callback(
            "dbclick",
            self.double_click.1.as_ref().unchecked_ref(),
        );

        let _ = self.canvas.remove_event_listener_with_callback(
            "mousedown",
            self.mouse_down.1.as_ref().unchecked_ref(),
        );

        let _ = self.canvas.remove_event_listener_with_callback(
            "mouseenter",
            self.mouse_enter.1.as_ref().unchecked_ref(),
        );

        let _ = self.canvas.remove_event_listener_with_callback(
            "mouseleave",
            self.mouse_leave.1.as_ref().unchecked_ref(),
        );

        let _ = self.canvas.remove_event_listener_with_callback(
            "mousemove",
            self.mouse_move.1.as_ref().unchecked_ref(),
        );

        let _ = self.canvas.remove_event_listener_with_callback(
            "mouseout",
            self.mouse_out.1.as_ref().unchecked_ref(),
        );

        let _ = self.canvas.remove_event_listener_with_callback(
            "mouseover",
            self.mouse_over.1.as_ref().unchecked_ref(),
        );

        let _ = self.canvas.remove_event_listener_with_callback(
            "mouseup",
            self.mouse_up.1.as_ref().unchecked_ref(),
        );

        let _ = self
            .canvas
            .remove_event_listener_with_callback("wheel", self.wheel.1.as_ref().unchecked_ref());

        let _ = self.canvas.remove_event_listener_with_callback(
            "keydown",
            self.key_down.1.as_ref().unchecked_ref(),
        );

        let _ = self
            .canvas
            .remove_event_listener_with_callback("keyup", self.key_up.1.as_ref().unchecked_ref());
    }
}

impl SceneCanvasHandler {
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, Error> {
        let resize_observer_notifier = Notifier::new();
        let resize_observer_notifier_cloned = resize_observer_notifier.clone();
        let resize_observer_callback = Closure::new(move |entries: Vec<ResizeObserverEntry>| {
            // should have only one entry
            let Some(target) = entries.get(0).map(|entry| entry.target()) else {
                return;
            };
            let Ok(mut canvas) = target.dyn_into::<HtmlCanvasElement>() else {
                return;
            };

            let width = canvas.client_width() as u32;
            let height = canvas.client_height() as u32;
            canvas.set_width(width);
            canvas.set_height(height);
            resize_observer_notifier_cloned.notify(&mut canvas);
        });
        let resize_observer =
            ResizeObserver::new(resize_observer_callback.as_ref().unchecked_ref())
                .or_else(|err| Err(Error::CanvasResizeObserverFailure(err.as_string())))?;
        resize_observer.observe(&canvas);

        let click_notifier = Notifier::new();
        let click_notifier_cloned = click_notifier.clone();
        let click_callback = Closure::new(move |mut e| click_notifier_cloned.notify(&mut e));
        canvas
            .add_event_listener_with_callback("click", click_callback.as_ref().unchecked_ref())
            .or_else(|err| Err(Error::AddEventCallbackFailure("click", err.as_string())))?;

        let double_click_notifier = Notifier::new();
        let double_click_notifier_cloned = double_click_notifier.clone();
        let double_click_callback =
            Closure::new(move |mut e| double_click_notifier_cloned.notify(&mut e));
        canvas
            .add_event_listener_with_callback(
                "dbclick",
                double_click_callback.as_ref().unchecked_ref(),
            )
            .or_else(|err| Err(Error::AddEventCallbackFailure("dbclick", err.as_string())))?;

        let mouse_down_notifier = Notifier::new();
        let mouse_down_notifier_cloned = mouse_down_notifier.clone();
        let mouse_down_callback =
            Closure::new(move |mut e| mouse_down_notifier_cloned.notify(&mut e));
        canvas
            .add_event_listener_with_callback(
                "mousedown",
                mouse_down_callback.as_ref().unchecked_ref(),
            )
            .or_else(|err| Err(Error::AddEventCallbackFailure("mousedown", err.as_string())))?;

        let mouse_enter_notifier = Notifier::new();
        let mouse_enter_notifier_cloned = mouse_enter_notifier.clone();
        let mouse_enter_callback =
            Closure::new(move |mut e| mouse_enter_notifier_cloned.notify(&mut e));
        canvas
            .add_event_listener_with_callback(
                "mouseenter",
                mouse_enter_callback.as_ref().unchecked_ref(),
            )
            .or_else(|err| {
                Err(Error::AddEventCallbackFailure(
                    "mouseenter",
                    err.as_string(),
                ))
            })?;

        let mouse_leave_notifier = Notifier::new();
        let mouse_leave_notifier_cloned = mouse_leave_notifier.clone();
        let mouse_leave_callback =
            Closure::new(move |mut e| mouse_leave_notifier_cloned.notify(&mut e));
        canvas
            .add_event_listener_with_callback(
                "mouseleave",
                mouse_leave_callback.as_ref().unchecked_ref(),
            )
            .or_else(|err| {
                Err(Error::AddEventCallbackFailure(
                    "mouseleave",
                    err.as_string(),
                ))
            })?;

        let mouse_move_notifier = Notifier::new();
        let mouse_move_notifier_cloned = mouse_move_notifier.clone();
        let mouse_move_callback =
            Closure::new(move |mut e| mouse_move_notifier_cloned.notify(&mut e));
        canvas
            .add_event_listener_with_callback(
                "mousemove",
                mouse_move_callback.as_ref().unchecked_ref(),
            )
            .or_else(|err| Err(Error::AddEventCallbackFailure("mousemove", err.as_string())))?;

        let mouse_out_notifier = Notifier::new();
        let mouse_out_notifier_cloned = mouse_out_notifier.clone();
        let mouse_out_callback =
            Closure::new(move |mut e| mouse_out_notifier_cloned.notify(&mut e));
        canvas
            .add_event_listener_with_callback(
                "mouseout",
                mouse_out_callback.as_ref().unchecked_ref(),
            )
            .or_else(|err| Err(Error::AddEventCallbackFailure("mouseout", err.as_string())))?;

        let mouse_over_notifier = Notifier::new();
        let mouse_over_notifier_cloned = mouse_over_notifier.clone();
        let mouse_over_callback =
            Closure::new(move |mut e| mouse_over_notifier_cloned.notify(&mut e));
        canvas
            .add_event_listener_with_callback(
                "mouseover",
                mouse_over_callback.as_ref().unchecked_ref(),
            )
            .or_else(|err| Err(Error::AddEventCallbackFailure("mouseover", err.as_string())))?;

        let mouse_up_notifier = Notifier::new();
        let mouse_up_notifier_cloned = mouse_up_notifier.clone();
        let mouse_up_callback = Closure::new(move |mut e| mouse_up_notifier_cloned.notify(&mut e));
        canvas
            .add_event_listener_with_callback("mouseup", mouse_up_callback.as_ref().unchecked_ref())
            .or_else(|err| Err(Error::AddEventCallbackFailure("mouseup", err.as_string())))?;

        let wheel_notifier = Notifier::new();
        let wheel_notifier_cloned = wheel_notifier.clone();
        let wheel_callback = Closure::new(move |mut e| wheel_notifier_cloned.notify(&mut e));
        canvas
            .add_event_listener_with_callback("wheel", wheel_callback.as_ref().unchecked_ref())
            .or_else(|err| Err(Error::AddEventCallbackFailure("wheel", err.as_string())))?;

        let key_down_notifier = Notifier::new();
        let key_down_notifier_cloned = key_down_notifier.clone();
        let key_down_callback = Closure::new(move |mut e| key_down_notifier_cloned.notify(&mut e));
        canvas
            .add_event_listener_with_callback("keydown", key_down_callback.as_ref().unchecked_ref())
            .or_else(|err| Err(Error::AddEventCallbackFailure("keydown", err.as_string())))?;

        let key_up_notifier = Notifier::new();
        let key_up_notifier_cloned = key_up_notifier.clone();
        let key_up_callback = Closure::new(move |mut e| key_up_notifier_cloned.notify(&mut e));
        canvas
            .add_event_listener_with_callback("keyup", key_up_callback.as_ref().unchecked_ref())
            .or_else(|err| Err(Error::AddEventCallbackFailure("keyup", err.as_string())))?;

        Ok(Self {
            canvas,
            canvas_resize: (
                resize_observer_notifier,
                resize_observer,
                resize_observer_callback,
            ),
            click: (click_notifier, click_callback),
            double_click: (double_click_notifier, double_click_callback),
            mouse_down: (mouse_down_notifier, mouse_down_callback),
            mouse_enter: (mouse_enter_notifier, mouse_enter_callback),
            mouse_leave: (mouse_leave_notifier, mouse_leave_callback),
            mouse_move: (mouse_move_notifier, mouse_move_callback),
            mouse_out: (mouse_out_notifier, mouse_out_callback),
            mouse_over: (mouse_over_notifier, mouse_over_callback),
            mouse_up: (mouse_up_notifier, mouse_up_callback),
            wheel: (wheel_notifier, wheel_callback),
            key_down: (key_down_notifier, key_down_callback),
            key_up: (key_up_notifier, key_up_callback),
        })
    }

    pub fn canvas_resize(&mut self) -> &mut Notifier<HtmlCanvasElement> {
        &mut self.canvas_resize.0
    }

    pub fn click(&mut self) -> &mut Notifier<MouseEvent> {
        &mut self.click.0
    }

    pub fn double_click(&mut self) -> &mut Notifier<MouseEvent> {
        &mut self.double_click.0
    }

    pub fn mouse_down(&mut self) -> &mut Notifier<MouseEvent> {
        &mut self.mouse_down.0
    }

    pub fn mouse_enter(&mut self) -> &mut Notifier<MouseEvent> {
        &mut self.mouse_enter.0
    }

    pub fn mouse_leave(&mut self) -> &mut Notifier<MouseEvent> {
        &mut self.mouse_leave.0
    }

    pub fn mouse_move(&mut self) -> &mut Notifier<MouseEvent> {
        &mut self.mouse_move.0
    }

    pub fn mouse_out(&mut self) -> &mut Notifier<MouseEvent> {
        &mut self.mouse_out.0
    }

    pub fn mouse_over(&mut self) -> &mut Notifier<MouseEvent> {
        &mut self.mouse_over.0
    }

    pub fn mouse_up(&mut self) -> &mut Notifier<MouseEvent> {
        &mut self.mouse_up.0
    }

    pub fn wheel(&mut self) -> &mut Notifier<WheelEvent> {
        &mut self.wheel.0
    }

    pub fn key_down(&mut self) -> &mut Notifier<KeyboardEvent> {
        &mut self.key_down.0
    }

    pub fn key_up(&mut self) -> &mut Notifier<KeyboardEvent> {
        &mut self.key_up.0
    }
}
