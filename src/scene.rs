use std::{cell::RefCell, rc::Rc};

use log::warn;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{
    HtmlCanvasElement, KeyboardEvent, MouseEvent, ResizeObserver, ResizeObserverEntry, WheelEvent,
};

use crate::{
    clock::{Clock, Tick, WebClock},
    document,
    entity::{Group, SceneGroup},
    error::Error,
    light::{
        ambient_light::AmbientLight, area_light::AreaLight, attenuation::Attenuation,
        directional_light::DirectionalLight, point_light::PointLight, spot_light::SpotLight,
    },
    notify::{Notifiee, Notifier},
};

/// Maximum area lights.
pub const MAX_AREA_LIGHTS: usize = 12;
pub(crate) const MAX_AREA_LIGHTS_STRING: &'static str = "12";
pub(crate) const AREA_LIGHTS_COUNT_DEFINE: &'static str = "AREA_LIGHTS_COUNT";
/// Maximum directional lights.
pub const MAX_DIRECTIONAL_LIGHTS: usize = 12;
pub(crate) const MAX_DIRECTIONAL_LIGHTS_STRING: &'static str = "12";
pub(crate) const DIRECTIONAL_LIGHTS_COUNT_DEFINE: &'static str = "DIRECTIONAL_LIGHTS_COUNT";
/// Maximum point lights.
pub const MAX_POINT_LIGHTS: usize = 40;
pub(crate) const MAX_POINT_LIGHTS_STRING: &'static str = "40";
pub(crate) const POINT_LIGHTS_COUNT_DEFINE: &'static str = "POINT_LIGHTS_COUNT";
/// Maximum spot lights.
pub const MAX_SPOT_LIGHTS: usize = 12;
pub(crate) const MAX_SPOT_LIGHTS_STRING: &'static str = "12";
pub(crate) const SPOT_LIGHTS_COUNT_DEFINE: &'static str = "SPOT_LIGHTS_COUNT";

pub struct Scene<Clock> {
    canvas: HtmlCanvasElement,
    canvas_handler: SceneCanvasHandler,
    _select_start_callback: Closure<dyn Fn() -> bool>,

    clock: Clock,
    entities: *mut SceneGroup,
    light_attenuations: Attenuation,
    ambient_light: Option<AmbientLight>,
    point_lights: Vec<PointLight>,
    directional_lights: Vec<DirectionalLight>,
    spot_lights: Vec<SpotLight>,
    area_lights: Vec<AreaLight>,
}

impl<Clock> Drop for Scene<Clock> {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.entities));
        }
    }
}

impl Scene<WebClock> {
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

        let entities = Box::leak(Box::new(SceneGroup::new()));

        let mut clock = WebClock::new();
        clock.on_tick(SceneTicking::new(entities));

        Ok(Self {
            canvas_handler: SceneCanvasHandler::new(canvas.clone())?,
            _select_start_callback: select_start_callback,
            canvas,

            clock,
            entities,
            light_attenuations: Attenuation::new(0.0, 1.0, 0.0),
            ambient_light: None,
            point_lights: Vec::new(),
            directional_lights: Vec::new(),
            spot_lights: Vec::new(),
            area_lights: Vec::new(),
        })
    }
}

impl<Clock> Scene<Clock> {
    /// Returns [`HtmlCanvasElement`].
    pub fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }

    pub fn canvas_handler(&mut self) -> &mut SceneCanvasHandler {
        &mut self.canvas_handler
    }

    /// Returns [`Clock`](crate::clock::Clock) associated with this scene.
    pub fn clock(&self) -> &Clock {
        &self.clock
    }

    /// Returns mutable [`Clock`](crate::clock::Clock) associated with this scene.
    pub fn clock_mut(&mut self) -> &mut Clock {
        &mut self.clock
    }

    /// Returns entity group.
    pub fn entity_group(&self) -> &SceneGroup {
        unsafe { &*self.entities }
    }

    /// Returns mutable entity group.
    pub fn entity_group_mut(&mut self) -> &mut SceneGroup {
        unsafe { &mut *self.entities }
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
    pub fn light_attenuations(&self) -> &Attenuation {
        &self.light_attenuations
    }

    /// Sets lighting attenuation.
    pub fn set_light_attenuations(&mut self, attenuations: Attenuation) {
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
}

pub struct SceneCanvasHandler {
    canvas: HtmlCanvasElement,
    canvas_resize: (
        Rc<RefCell<Notifier<HtmlCanvasElement>>>,
        ResizeObserver,
        Closure<dyn FnMut(Vec<ResizeObserverEntry>)>,
    ),
    click: (
        Rc<RefCell<Notifier<MouseEvent>>>,
        Closure<dyn FnMut(MouseEvent)>,
    ),
    double_click: (
        Rc<RefCell<Notifier<MouseEvent>>>,
        Closure<dyn FnMut(MouseEvent)>,
    ),
    mouse_down: (
        Rc<RefCell<Notifier<MouseEvent>>>,
        Closure<dyn FnMut(MouseEvent)>,
    ),
    mouse_enter: (
        Rc<RefCell<Notifier<MouseEvent>>>,
        Closure<dyn FnMut(MouseEvent)>,
    ),
    mouse_leave: (
        Rc<RefCell<Notifier<MouseEvent>>>,
        Closure<dyn FnMut(MouseEvent)>,
    ),
    mouse_move: (
        Rc<RefCell<Notifier<MouseEvent>>>,
        Closure<dyn FnMut(MouseEvent)>,
    ),
    mouse_out: (
        Rc<RefCell<Notifier<MouseEvent>>>,
        Closure<dyn FnMut(MouseEvent)>,
    ),
    mouse_over: (
        Rc<RefCell<Notifier<MouseEvent>>>,
        Closure<dyn FnMut(MouseEvent)>,
    ),
    mouse_up: (
        Rc<RefCell<Notifier<MouseEvent>>>,
        Closure<dyn FnMut(MouseEvent)>,
    ),
    wheel: (
        Rc<RefCell<Notifier<WheelEvent>>>,
        Closure<dyn FnMut(WheelEvent)>,
    ),
    key_down: (
        Rc<RefCell<Notifier<KeyboardEvent>>>,
        Closure<dyn FnMut(KeyboardEvent)>,
    ),
    key_up: (
        Rc<RefCell<Notifier<KeyboardEvent>>>,
        Closure<dyn FnMut(KeyboardEvent)>,
    ),
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
        let resize_observer_notifier = Rc::new(RefCell::new(Notifier::new()));
        let resize_observer_notifier_cloned = Rc::clone(&resize_observer_notifier);
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
            resize_observer_notifier_cloned
                .borrow_mut()
                .notify(&mut canvas);
        });
        let resize_observer =
            ResizeObserver::new(resize_observer_callback.as_ref().unchecked_ref())
                .or_else(|err| Err(Error::CanvasResizeObserverFailure(err.as_string())))?;
        resize_observer.observe(&canvas);

        macro_rules! io_events {
            ($(($name:ident, $name_cloned:ident, $callback:ident, $event:expr))+) => {
                $(
                    let $name = Rc::new(RefCell::new(Notifier::new()));
                    let $name_cloned = Rc::clone(&$name);
                    let $callback = Closure::new(move |mut e| $name_cloned.borrow_mut().notify(&mut e));
                    canvas
                        .add_event_listener_with_callback($event, $callback.as_ref().unchecked_ref())
                        .or_else(|err| Err(Error::AddEventCallbackFailure($event, err.as_string())))?;
                )+
            };
        }

        io_events! {
            (click_notifier, click_notifier_cloned, click_callback, "click")
            (double_click_notifier, double_click_notifier_cloned, double_click_callback, "dbclick")
            (mouse_down_notifier, mouse_down_notifier_cloned, mouse_down_callback, "mousedown")
            (mouse_enter_notifier, mouse_enter_notifier_cloned, mouse_enter_callback, "mouseenter")
            (mouse_leave_notifier, mouse_leave_notifier_cloned, mouse_leave_callback, "mouseleave")
            (mouse_move_notifier, mouse_move_notifier_cloned, mouse_move_callback, "mousemove")
            (mouse_out_notifier, mouse_out_notifier_cloned, mouse_out_callback, "mouseout")
            (mouse_over_notifier, mouse_over_notifier_cloned, mouse_over_callback, "mouseover")
            (mouse_up_notifier, mouse_up_notifier_cloned, mouse_up_callback, "mouseup")
            (wheel_notifier, wheel_notifier_cloned, wheel_callback, "wheel")
            (key_down_notifier, key_down_notifier_cloned, key_down_callback, "keydown")
            (key_up_notifier, key_up_notifier_cloned, key_up_callback, "keyup")
        };

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

    pub fn canvas_resize(&mut self) -> &Rc<RefCell<Notifier<HtmlCanvasElement>>> {
        &mut self.canvas_resize.0
    }

    pub fn click(&mut self) -> &Rc<RefCell<Notifier<MouseEvent>>> {
        &mut self.click.0
    }

    pub fn double_click(&mut self) -> &Rc<RefCell<Notifier<MouseEvent>>> {
        &mut self.double_click.0
    }

    pub fn mouse_down(&mut self) -> &Rc<RefCell<Notifier<MouseEvent>>> {
        &mut self.mouse_down.0
    }

    pub fn mouse_enter(&mut self) -> &Rc<RefCell<Notifier<MouseEvent>>> {
        &mut self.mouse_enter.0
    }

    pub fn mouse_leave(&mut self) -> &Rc<RefCell<Notifier<MouseEvent>>> {
        &mut self.mouse_leave.0
    }

    pub fn mouse_move(&mut self) -> &Rc<RefCell<Notifier<MouseEvent>>> {
        &mut self.mouse_move.0
    }

    pub fn mouse_out(&mut self) -> &Rc<RefCell<Notifier<MouseEvent>>> {
        &mut self.mouse_out.0
    }

    pub fn mouse_over(&mut self) -> &Rc<RefCell<Notifier<MouseEvent>>> {
        &mut self.mouse_over.0
    }

    pub fn mouse_up(&mut self) -> &Rc<RefCell<Notifier<MouseEvent>>> {
        &mut self.mouse_up.0
    }

    pub fn wheel(&mut self) -> &Rc<RefCell<Notifier<WheelEvent>>> {
        &mut self.wheel.0
    }

    pub fn key_down(&mut self) -> &Rc<RefCell<Notifier<KeyboardEvent>>> {
        &mut self.key_down.0
    }

    pub fn key_up(&mut self) -> &Rc<RefCell<Notifier<KeyboardEvent>>> {
        &mut self.key_up.0
    }
}

struct SceneTicking {
    entities: *mut SceneGroup,
}

impl SceneTicking {
    fn new(entities: *mut SceneGroup) -> Self {
        Self { entities }
    }
}

impl Notifiee<Tick> for SceneTicking {
    fn notify(&mut self, msg: &Tick) {
        unsafe {
            if (*self.entities).tick(msg) {
                log::info!("3333");
                (*self.entities).set_resync();
            }
        }
    }
}
