use nightshade::prelude::*;

pub const TRANSPARENT: Vec4 = Vec4::new(0.0, 0.0, 0.0, 0.0);
pub const WHITE: Vec4 = Vec4::new(1.0, 1.0, 1.0, 1.0);

pub const TEXT_COLOR: Vec4 = Vec4::new(0.95, 0.92, 0.90, 1.0);
pub const TEXT_DIM: Vec4 = Vec4::new(0.80, 0.62, 0.55, 1.0);
pub const TEXT_FAINT: Vec4 = Vec4::new(0.62, 0.46, 0.42, 1.0);

pub const ACCENT: Vec4 = Vec4::new(0.95, 0.52, 0.22, 1.0);
pub const ACCENT_DIM: Vec4 = Vec4::new(0.36, 0.14, 0.08, 1.0);
pub const ACCENT_HOT: Vec4 = Vec4::new(1.0, 0.74, 0.36, 1.0);

pub const PANEL_BG: Vec4 = Vec4::new(0.10, 0.05, 0.05, 0.10);
pub const PANEL_BG_DEEP: Vec4 = Vec4::new(0.10, 0.05, 0.05, 0.62);
pub const PANEL_BORDER: Vec4 = Vec4::new(0.86, 0.40, 0.24, 0.85);
pub const PANEL_HOVER: Vec4 = Vec4::new(0.20, 0.09, 0.07, 0.58);
pub const PANEL_PRESSED: Vec4 = Vec4::new(0.10, 0.05, 0.05, 0.82);

pub const VIGNETTE: Vec4 = Vec4::new(0.04, 0.0, 0.0, 0.50);
pub const BACKDROP: Vec4 = Vec4::new(0.04, 0.0, 0.0, 0.60);

pub const CROSSHAIR: Vec4 = Vec4::new(0.98, 0.96, 0.94, 0.85);

pub const HEALTH: Vec4 = Vec4::new(0.95, 0.32, 0.28, 1.0);
pub const AMMO: Vec4 = Vec4::new(0.98, 0.82, 0.30, 1.0);
pub const DAMAGE_FLASH: Vec4 = Vec4::new(0.78, 0.05, 0.05, 0.45);

pub const MENU_BUTTON_HEIGHT: f32 = 48.0;
pub const MENU_BUTTON_SIZE: Vec2 = Vec2::new(320.0, MENU_BUTTON_HEIGHT);
