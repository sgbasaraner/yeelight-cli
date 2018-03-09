#[derive(Debug, Clone)]
pub struct Bulb {
    pub id: String,
    pub model: String,
    pub fw_ver: u16,
    pub support: String,
    pub power: bool,
    pub bright: u8,
    pub color_mode: u8,
    pub ct: u16,
    pub rgb: RGB,
    pub hue: u16,
    pub sat: u8,
    pub name: String,
    pub ip: String
}

#[derive(Debug, Clone, Copy)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8
}