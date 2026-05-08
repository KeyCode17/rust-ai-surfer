use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
    pub device_scale_factor: f32,
    pub is_mobile: bool,
}

impl Default for Viewport {
    fn default() -> Self {
        Self { width: 1280, height: 720, device_scale_factor: 1.0, is_mobile: false }
    }
}
