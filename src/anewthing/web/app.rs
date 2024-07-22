use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub struct App(crate::anewthing::app::App);

impl App {
    pub fn new(app: crate::anewthing::app::App) -> Self {
        Self(app)
    }
}

#[wasm_bindgen]
impl App {
    pub fn run(&mut self) {
        self.0.run()
    }

    pub fn terminate(&mut self) {
        self.0.terminate()
    }
}
