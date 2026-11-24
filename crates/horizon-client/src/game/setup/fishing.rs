use std::sync::Arc;

pub const RATE: u32 = 44100;

#[derive(Default)]
pub struct Fishing;

impl Fishing {
    pub fn new() -> Self { Self }
    pub fn update(&mut self, _now: std::time::Instant) {}
    pub fn take_sounds(&mut self) -> Vec<Sound> { Vec::new() }
    pub fn draw_water(&self, _ui: &mut horizon_client::ui_theme::Ui, _w: f32, _h: f32, _now: std::time::Instant) {}
    pub fn draw(&self, _ui: &mut horizon_client::ui_theme::Ui, _w: f32, _h: f32, _y: f32, _now: std::time::Instant) {}
    pub fn press(&mut self, _now: std::time::Instant, _p: [f32; 2]) {}
    pub fn release(&mut self, _now: std::time::Instant) {}
}

pub struct Sound;

impl Sound {
    pub fn samples(&self) -> Arc<Vec<f32>> { Arc::new(Vec::new()) }
}
