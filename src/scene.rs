use std::{cell::RefCell, rc::Rc};

use log::warn;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{
    HtmlCanvasElement, KeyboardEvent, MouseEvent, ResizeObserver, ResizeObserverEntry, WheelEvent,
};

use crate::{
    channel::{channel, Aborter, Executor, Receiver, Sender},
    clock::{Clock, HtmlClock, Tick},
    document,
    entity::{Group, SimpleGroup},
    error::Error,
    light::{
        ambient_light::AmbientLight, area_light::AreaLight, attenuation::Attenuation,
        directional_light::DirectionalLight, point_light::PointLight, spot_light::SpotLight,
    },
    share::Share,
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

pub struct Scene {
    canvas: HtmlCanvasElement,
    canvas_handler: CanvasHandler,
    _select_start_callback: Closure<dyn Fn() -> bool>,

    clock: HtmlClock,
    _clock_aborter: Aborter<Tick>,

    entities: Share<SimpleGroup>,
    light_attenuation: Attenuation,
    ambient_light: Option<AmbientLight>,
    directional_lights: Vec<DirectionalLight>,
    point_lights: Vec<PointLight>,
    spot_lights: Vec<SpotLight>,
    area_lights: Vec<AreaLight>,
}

impl Drop for Scene {
    fn drop(&mut self) {
        self.canvas.set_onselectstart(None);
    }
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

        let entities = Rc::new(RefCell::new(SimpleGroup::new()));

        let clock = HtmlClock::new();
        let mut clock_aborter = clock.ticking().on(ClockTicking::new(Rc::clone(&entities)));
        clock_aborter.set_off_when_dropped(true);

        Ok(Self {
            canvas_handler: CanvasHandler::new(canvas.clone())?,
            _select_start_callback: select_start_callback,
            canvas,

            clock,
            _clock_aborter: clock_aborter,

            entities,
            light_attenuation: Attenuation::new(0.0, 1.0, 0.0),
            ambient_light: None,
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            area_lights: Vec::new(),
        })
    }

    /// Returns [`HtmlCanvasElement`].
    pub fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }

    pub fn canvas_handler(&mut self) -> &mut CanvasHandler {
        &mut self.canvas_handler
    }

    /// Returns [`Clock`](crate::clock::Clock) associated with this scene.
    pub fn clock(&self) -> &HtmlClock {
        &self.clock
    }

    /// Returns mutable [`Clock`](crate::clock::Clock) associated with this scene.
    pub fn clock_mut(&mut self) -> &mut HtmlClock {
        &mut self.clock
    }

    /// Returns entity group.
    pub fn entities(&self) -> &Share<SimpleGroup> {
        &self.entities
    }

    /// Returns ambient light.
    pub fn ambient_light(&self) -> &Option<AmbientLight> {
        &self.ambient_light
    }

    /// Returns mutable ambient light.
    pub fn ambient_light_mut(&mut self) -> &mut Option<AmbientLight> {
        &mut self.ambient_light
    }

    /// Sets ambient light.
    pub fn set_ambient_light(&mut self, light: Option<AmbientLight>) {
        self.ambient_light = light;
    }

    /// Returns lighting attenuation.
    pub fn light_attenuation(&self) -> &Attenuation {
        &self.light_attenuation
    }

    /// Returns mutable lighting attenuation.
    pub fn light_attenuation_mut(&mut self) -> &mut Attenuation {
        &mut self.light_attenuation
    }

    /// Sets lighting attenuation.
    pub fn set_light_attenuation(&mut self, attenuations: Attenuation) {
        self.light_attenuation = attenuations;
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

    /// Adds a point light.
    pub fn add_point_light(&mut self, light: PointLight) {
        if self.point_lights.len() >= MAX_POINT_LIGHTS {
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
}

pub struct CanvasHandler {
    canvas: HtmlCanvasElement,
    canvas_resize: (
        Sender<HtmlCanvasElement>,
        Receiver<HtmlCanvasElement>,
        ResizeObserver,
        Closure<dyn FnMut(Vec<ResizeObserverEntry>)>,
    ),
    click: (
        Sender<MouseEvent>,
        Receiver<MouseEvent>,
        Closure<dyn FnMut(MouseEvent)>,
    ),
    double_click: (
        Sender<MouseEvent>,
        Receiver<MouseEvent>,
        Closure<dyn FnMut(MouseEvent)>,
    ),
    mouse_down: (
        Sender<MouseEvent>,
        Receiver<MouseEvent>,
        Closure<dyn FnMut(MouseEvent)>,
    ),
    mouse_enter: (
        Sender<MouseEvent>,
        Receiver<MouseEvent>,
        Closure<dyn FnMut(MouseEvent)>,
    ),
    mouse_leave: (
        Sender<MouseEvent>,
        Receiver<MouseEvent>,
        Closure<dyn FnMut(MouseEvent)>,
    ),
    mouse_move: (
        Sender<MouseEvent>,
        Receiver<MouseEvent>,
        Closure<dyn FnMut(MouseEvent)>,
    ),
    mouse_out: (
        Sender<MouseEvent>,
        Receiver<MouseEvent>,
        Closure<dyn FnMut(MouseEvent)>,
    ),
    mouse_over: (
        Sender<MouseEvent>,
        Receiver<MouseEvent>,
        Closure<dyn FnMut(MouseEvent)>,
    ),
    mouse_up: (
        Sender<MouseEvent>,
        Receiver<MouseEvent>,
        Closure<dyn FnMut(MouseEvent)>,
    ),
    wheel: (
        Sender<WheelEvent>,
        Receiver<WheelEvent>,
        Closure<dyn FnMut(WheelEvent)>,
    ),
    key_down: (
        Sender<KeyboardEvent>,
        Receiver<KeyboardEvent>,
        Closure<dyn FnMut(KeyboardEvent)>,
    ),
    key_up: (
        Sender<KeyboardEvent>,
        Receiver<KeyboardEvent>,
        Closure<dyn FnMut(KeyboardEvent)>,
    ),
}

impl Drop for CanvasHandler {
    fn drop(&mut self) {
        self.canvas_resize.2.disconnect();

        let _ = self
            .canvas
            .remove_event_listener_with_callback("click", self.click.2.as_ref().unchecked_ref());

        let _ = self.canvas.remove_event_listener_with_callback(
            "dbclick",
            self.double_click.2.as_ref().unchecked_ref(),
        );

        let _ = self.canvas.remove_event_listener_with_callback(
            "mousedown",
            self.mouse_down.2.as_ref().unchecked_ref(),
        );

        let _ = self.canvas.remove_event_listener_with_callback(
            "mouseenter",
            self.mouse_enter.2.as_ref().unchecked_ref(),
        );

        let _ = self.canvas.remove_event_listener_with_callback(
            "mouseleave",
            self.mouse_leave.2.as_ref().unchecked_ref(),
        );

        let _ = self.canvas.remove_event_listener_with_callback(
            "mousemove",
            self.mouse_move.2.as_ref().unchecked_ref(),
        );

        let _ = self.canvas.remove_event_listener_with_callback(
            "mouseout",
            self.mouse_out.2.as_ref().unchecked_ref(),
        );

        let _ = self.canvas.remove_event_listener_with_callback(
            "mouseover",
            self.mouse_over.2.as_ref().unchecked_ref(),
        );

        let _ = self.canvas.remove_event_listener_with_callback(
            "mouseup",
            self.mouse_up.2.as_ref().unchecked_ref(),
        );

        let _ = self
            .canvas
            .remove_event_listener_with_callback("wheel", self.wheel.2.as_ref().unchecked_ref());

        let _ = self.canvas.remove_event_listener_with_callback(
            "keydown",
            self.key_down.2.as_ref().unchecked_ref(),
        );

        let _ = self
            .canvas
            .remove_event_listener_with_callback("keyup", self.key_up.2.as_ref().unchecked_ref());
    }
}

impl CanvasHandler {
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, Error> {
        let (resize_observer_sender, resize_observer_receiver) = channel();
        let resize_observer_sender_cloned = resize_observer_sender.clone();
        let resize_observer_callback = Closure::new(move |entries: Vec<ResizeObserverEntry>| {
            // should have only one entry
            let Some(target) = entries.get(0).map(|entry| entry.target()) else {
                return;
            };
            let Ok(canvas) = target.dyn_into::<HtmlCanvasElement>() else {
                return;
            };

            let width = canvas.client_width() as u32;
            let height = canvas.client_height() as u32;
            canvas.set_width(width);
            canvas.set_height(height);
            resize_observer_sender_cloned.send(&canvas);
        });
        let resize_observer =
            ResizeObserver::new(resize_observer_callback.as_ref().unchecked_ref())
                .or_else(|err| Err(Error::CanvasResizeObserverFailure(err.as_string())))?;
        resize_observer.observe(&canvas);

        macro_rules! io_events {
            ($(($tx:ident, $rx:ident, $tx_cloned:ident, $callback:ident, $event:expr))+) => {
                $(
                    let ($tx, $rx) = channel();
                    let $tx_cloned = $tx.clone();
                    let $callback = Closure::new(move |e| $tx_cloned.send(& e));
                    canvas
                        .add_event_listener_with_callback($event, $callback.as_ref().unchecked_ref())
                        .or_else(|err| Err(Error::AddEventCallbackFailure($event, err.as_string())))?;
                )+
            };
        }

        io_events! {
            (click_sender, click_receiver, click_sender_cloned, click_callback, "click")
            (double_click_sender, double_click_receiver, double_click_sender_cloned, double_click_callback, "dbclick")
            (mouse_down_sender, mouse_down_receiver, mouse_down_sender_cloned, mouse_down_callback, "mousedown")
            (mouse_enter_sender, mouse_enter_receiver, mouse_enter_sender_cloned, mouse_enter_callback, "mouseenter")
            (mouse_leave_sender, mouse_leave_receiver, mouse_leave_sender_cloned, mouse_leave_callback, "mouseleave")
            (mouse_move_sender, mouse_move_receiver, mouse_move_sender_cloned, mouse_move_callback, "mousemove")
            (mouse_out_sender, mouse_out_receiver, mouse_out_sender_cloned, mouse_out_callback, "mouseout")
            (mouse_over_sender, mouse_over_receiver, mouse_over_sender_cloned, mouse_over_callback, "mouseover")
            (mouse_up_sender, mouse_up_receiver, mouse_up_sender_cloned, mouse_up_callback, "mouseup")
            (wheel_sender, wheel_receiver, wheel_sender_cloned, wheel_callback, "wheel")
            (key_down_sender, key_down_receiver, key_down_sender_cloned, key_down_callback, "keydown")
            (key_up_sender, key_up_receiver, key_up_sender_cloned, key_up_callback, "keyup")
        };

        Ok(Self {
            canvas,
            canvas_resize: (
                resize_observer_sender,
                resize_observer_receiver,
                resize_observer,
                resize_observer_callback,
            ),
            click: (click_sender, click_receiver, click_callback),
            double_click: (
                double_click_sender,
                double_click_receiver,
                double_click_callback,
            ),
            mouse_down: (mouse_down_sender, mouse_down_receiver, mouse_down_callback),
            mouse_enter: (
                mouse_enter_sender,
                mouse_enter_receiver,
                mouse_enter_callback,
            ),
            mouse_leave: (
                mouse_leave_sender,
                mouse_leave_receiver,
                mouse_leave_callback,
            ),
            mouse_move: (mouse_move_sender, mouse_move_receiver, mouse_move_callback),
            mouse_out: (mouse_out_sender, mouse_out_receiver, mouse_out_callback),
            mouse_over: (mouse_over_sender, mouse_over_receiver, mouse_over_callback),
            mouse_up: (mouse_up_sender, mouse_up_receiver, mouse_up_callback),
            wheel: (wheel_sender, wheel_receiver, wheel_callback),
            key_down: (key_down_sender, key_down_receiver, key_down_callback),
            key_up: (key_up_sender, key_up_receiver, key_up_callback),
        })
    }

    pub fn canvas_resize(&self) -> Receiver<HtmlCanvasElement> {
        self.canvas_resize.1.clone()
    }

    pub fn click(&self) -> Receiver<MouseEvent> {
        self.click.1.clone()
    }

    pub fn double_click(&self) -> Receiver<MouseEvent> {
        self.double_click.1.clone()
    }

    pub fn mouse_down(&self) -> Receiver<MouseEvent> {
        self.mouse_down.1.clone()
    }

    pub fn mouse_enter(&self) -> Receiver<MouseEvent> {
        self.mouse_enter.1.clone()
    }

    pub fn mouse_leave(&self) -> Receiver<MouseEvent> {
        self.mouse_leave.1.clone()
    }

    pub fn mouse_move(&self) -> Receiver<MouseEvent> {
        self.mouse_move.1.clone()
    }

    pub fn mouse_out(&self) -> Receiver<MouseEvent> {
        self.mouse_out.1.clone()
    }

    pub fn mouse_over(&self) -> Receiver<MouseEvent> {
        self.mouse_over.1.clone()
    }

    pub fn mouse_up(&self) -> Receiver<MouseEvent> {
        self.mouse_up.1.clone()
    }

    pub fn wheel(&self) -> Receiver<WheelEvent> {
        self.wheel.1.clone()
    }

    pub fn key_down(&self) -> Receiver<KeyboardEvent> {
        self.key_down.1.clone()
    }

    pub fn key_up(&self) -> Receiver<KeyboardEvent> {
        self.key_up.1.clone()
    }
}

struct ClockTicking(Share<SimpleGroup>);

impl ClockTicking {
    fn new(entities: Share<SimpleGroup>) -> Self {
        Self(entities)
    }
}

impl Executor for ClockTicking {
    type Message = Tick;

    fn execute(&mut self, msg: &Tick) {
        (*self.0).borrow_mut().tick(msg);
    }

    fn abort(&self) -> bool {
        false
    }
}
