
use std::collections::HashSet;
use std::collections::HashMap;
use std::fmt;
use std::fs::Metadata;
// "ID":   The ID of a Yeelight WiFi LED device, 3rd party device should use this value to 
// uniquely identified a Yeelight WiFi LED device. 
 
// "MODEL":  The product model of a Yeelight smart device. Current it can be "mono", 
// "color", “stripe”, “ceiling”, “bslamp”. For "mono", it represents device that only supports 
// brightness adjustment. For "color", it represents device that support both color and color 
// temperature adjustment. “Stripe” stands for Yeelight smart LED stripe. “Ceiling” stands 
// for Yeelight Ceiling Light. More values may be added in future.
 
// "FW_VER": LED device's firmware version. 
 
// "SUPPORT": All the supported control methods separated by white space. 3Rd party device 
// can use this field to dynamically render the control view to user if necessary. Any control 
// request that invokes method that is not included in this field will be rejected by smart LED. 
 
// "POWER": Current status of the device. "on" means the device is currently turned on, "off" 
// means it's turned off (not un-powered, just software-managed off). 
 
// "BRIGHT": Current brightness, it's the percentage of maximum brightness. The range of 
// this value is 1 ~ 100. 
 
 
// "COLOR_MODE": Current light mode. 1 means color mode, 2 means color temperature 
// mode, 3 means HSV mode. 
 
// "CT": Current color temperature value. The range of this value depends on product model, 
// refert to Yeelight product description. This field is only valid if COLOR_MODE is 2. 
 
// "RGB": Current RGB value. The field is only valid if COLOR_MODE is 1. The value will be 
// explained in next section. 
 
// "HUE": Current hue value. The range of this value is 0 to 359. This field is only valid if 
// COLOR_MODE is 3. 
 
// "SAT": Current saturation value. The range of this value is 0 to 100. The field is only valid if 
// COLOR_MODE is 3. 
 
// "NAME": Name of the device. User can use “set_name” to store the name on the device. 
// The maximum length is 64 bytes. If none-ASCII character is used, it is suggested to 
// BASE64 the name first and then use “set_name” to store it on device. 

#[derive(Debug)]
pub struct SafeBulb {
    // The ID of a Yeelight WiFi LED device that uniquely identifies a Yeelight WiFi LED device. 
    pub id: i32,

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
}

fn parse_to_hashmap(search_response: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    search_response
        .split("\r\n")
        .into_iter()
        .flat_map(|line| {
            let split_line = line.split(": ");
            
            let key = split_line.next();
            if key.is_none() {
                return None;
            }

            let mut val = String::new();

            while let next = split_line.next() {
                next.map(|s| val.push_str(s));
            }

            Some((key.unwrap(), val))
        }).for_each(|pair| {
            map.insert(pair.0.to_string(), pair.1);
        });
    return map;
}

impl SafeBulb {
    pub fn parse(search_response: &str) -> Option<SafeBulb> {
        let response_map = parse_to_hashmap(search_response);

        let id: Option<i32> = response_map.get("id").map(|s| s.parse().ok()).flatten();
        // model: color  
        // fw_ver: 18  
        // support: get_prop set_default set_power toggle set_bright start_cf stop_cf set_scene 
        // cron_add cron_get cron_del set_ct_abx set_rgb  

        // power: on  
        // bright: 100  
        // color_mode: 2  
        // ct: 4000  
        // rgb: 16711680 
        // hue: 100 
        // sat: 35 
        // name: my_bulb
        let model = response_map.get("model");
        let support = response_map.get("support").map(|s| s.split(" ").flat_map(|s| Method::from_string(s)).collect());
        let power = response_map
    }

    
}


#[derive(Debug)]
pub struct HSV {
    // Current hue value. The range of this value is 0 to 359.
    pub hue: u16,
    
    // Current saturation value. The range of this value is 0 to 100.
    pub saturation: u8
}

#[derive(Debug)]
pub enum LightMode { 
    Color(RGB),
    // Current color temperature value.
    ColorTemperature(u32),
    Hsv(HSV)
}

#[derive(Debug)]
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
    On, Off
}

#[derive(Debug, Clone, Copy)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

impl RGB {
    fn new(int: u32) -> RGB {
        RGB {
            r: ((int >> 16) & 0xFF) as u8,
            g: ((int >> 8) & 0xFF) as u8,
            b: ((int >> 0) & 0xFF) as u8
        }
    }
}

impl fmt::Display for RGB {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {}, {}", self.r, self.g, self.b)
    }
}
