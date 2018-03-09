pub struct Bulb<'a> {
    id: &'a str,
    model: &'a str,
    fw_ver: u16,
    support: &'a str,
    power: bool,
    bright: u8,
    color_mode: u8,
    ct: u16,
    rgb: RGB,
    hue: u16,
    sat: u8,
    name: &'a str
}

pub struct RGB {
    r: u8,
    g: u8,
    b: u8
}