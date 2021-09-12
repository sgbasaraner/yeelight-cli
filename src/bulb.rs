use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;

#[derive(Debug)]
pub struct Bulb {
    // The ID of a Yeelight WiFi LED device that uniquely identifies a Yeelight WiFi LED device.
    pub id: String,

    // The product model of a Yeelight smart device.
    pub model: String,

    // LED device's firmware version.
    pub fw_ver: String,

    // All the supported control methods.
    pub support: HashSet<Method>,

    // Current status of the device.
    pub power: Power,

    // Current brightness, it's the percentage of maximum brightness. Must be between 0 and 100.
    pub bright: u8,

    // Current light mode.
    pub color_mode: LightMode,

    // Name of the device. User can use “set_name” to store the name on the device.
    // The maximum length is 64 bytes. If none-ASCII character is used, it is suggested to
    // BASE64 the name first and then use “set_name” to store it on device.
    pub name: String,

    pub ip_address: String,
}

fn parse_to_hashmap(search_response: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    search_response
        .split("\r\n")
        .into_iter()
        .flat_map(|line| {
            let split_line: Vec<&str> = line.split(": ").collect();

            let key = split_line.first();
            if key.is_none() {
                return None;
            }

            let val = split_line.iter().skip(1).map(|s| *s).collect();
            Some((key.unwrap().clone(), val))
        })
        .for_each(|pair| {
            map.insert(pair.0.to_string(), pair.1);
        });
    return map;
}

impl Bulb {
    pub fn parse(search_response: &str) -> Option<Bulb> {
        let response_map = parse_to_hashmap(search_response);

        let id = response_map.get("id");
        let model = response_map.get("model");
        let fw_ver = response_map.get("fw_ver");
        let support = response_map.get("support").map(|s| {
            s.split(" ")
                .flat_map(|s| Method::from_string(s))
                .collect::<HashSet<Method>>()
        });
        let power = response_map
            .get("power")
            .map(|s| Power::from_string(s))
            .flatten();
        let brightness = response_map
            .get("bright")
            .map(|s| s.parse::<u8>().ok())
            .flatten();

        let light_mode = LightMode::parse(&response_map);

        let name = response_map.get("name");

        let ip = response_map
            .get("Location")
            .map(|s| s.split("//").nth(1))
            .flatten();

        if let (
            Some(model),
            Some(id),
            Some(support),
            Some(power),
            Some(brightness),
            Some(light_mode),
            Some(fw_ver),
            Some(name),
            Some(ip),
        ) = (
            model, id, support, power, brightness, light_mode, fw_ver, name, ip,
        ) {
            let bulb = Bulb {
                bright: brightness,
                color_mode: light_mode,
                fw_ver: fw_ver.clone(),
                id: id.clone(),
                model: model.clone(),
                name: name.clone(),
                power: power,
                support: support,
                ip_address: ip.to_string(),
            };
            Some(bulb)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct HSV {
    // Current hue value. The range of this value is 0 to 359.
    pub hue: u16,

    // Current saturation value. The range of this value is 0 to 100.
    pub saturation: u8,
}

#[derive(Debug)]
pub enum LightMode {
    Color(RGB),
    // Current color temperature value.
    ColorTemperature(u32),
    Hsv(HSV),
}

impl fmt::Display for LightMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl LightMode {
    fn parse(response_map: &HashMap<String, String>) -> Option<LightMode> {
        response_map
            .get("color_mode")
            .map(|cm| cm.parse::<u8>().ok())
            .flatten()
            .map(|cm| match cm {
                1 => response_map
                    .get("rgb")
                    .map(|rgb| rgb.parse::<u32>().ok())
                    .flatten()
                    .map(|rgb| RGB::new(rgb))
                    .map(|rgb| LightMode::Color(rgb)),
                2 => response_map
                    .get("ct")
                    .map(|ct| ct.parse::<u32>().ok())
                    .flatten()
                    .map(|ct| LightMode::ColorTemperature(ct)),
                3 => response_map
                    .get("hue")
                    .map(|hue| {
                        response_map
                            .get("sat")
                            .map(|sat| {
                                hue.parse::<u16>().ok().map(|hue| {
                                    sat.parse::<u8>().ok().map(|sat| {
                                        LightMode::Hsv(HSV {
                                            hue: hue,
                                            saturation: sat,
                                        })
                                    })
                                })
                            })
                            .flatten()
                            .flatten()
                    })
                    .flatten(),
                _ => None,
            })
            .flatten()
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum Method {
    GetProp,
    SetDefault,
    SetPower,
    Toggle,
    SetBright,
    StartCf,
    StopCf,
    SetScene,
    CronAdd,
    CronGet,
    CronDel,
    SetCtAbx,
    SetRgb,
}

impl Method {
    fn from_string(str: &str) -> Option<Method> {
        match str {
            "get_prop" => Some(Method::GetProp),
            "set_default" => Some(Method::SetDefault),
            "set_power" => Some(Method::SetPower),
            "toggle" => Some(Method::Toggle),
            "set_bright" => Some(Method::SetBright),
            "start_cf" => Some(Method::StartCf),
            "stop_cf" => Some(Method::StopCf),
            "set_scene" => Some(Method::SetScene),
            "cron_add" => Some(Method::CronAdd),
            "cron_get" => Some(Method::CronGet),
            "cron_del" => Some(Method::CronDel),
            "set_ct_abx" => Some(Method::SetCtAbx),
            "set_rgb" => Some(Method::SetRgb),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum Power {
    On,
    Off,
}

impl Power {
    fn from_string(str: &str) -> Option<Power> {
        match str {
            "on" => Some(Power::On),
            "off" => Some(Power::Off),
            _ => None,
        }
    }
}

impl fmt::Display for Power {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Power::On => write!(f, "on"),
            Power::Off => write!(f, "off"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGB {
    fn new(int: u32) -> RGB {
        RGB {
            r: ((int >> 16) & 0xFF) as u8,
            g: ((int >> 8) & 0xFF) as u8,
            b: ((int >> 0) & 0xFF) as u8,
        }
    }
}

impl fmt::Display for RGB {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {}, {}", self.r, self.g, self.b)
    }
}
