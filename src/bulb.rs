use std::fmt;

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

impl Bulb {
    pub fn new(search_response: &str) -> Bulb {
        let params = ["id", "model", "fw_ver", "support", "power", "bright", "color_mode", "ct", "rgb", "hue", "sat", "name"];
        let values = params
            .iter()
            .map(|p| Bulb::get_param_value(search_response, p).unwrap())
            .collect::<Vec<String>>();

        Bulb {
            id: values[0].clone(),
            model: values[1].clone(),
            fw_ver: values[2].parse::<u16>().unwrap(),
            support: values[3].clone(),
            power: values[4] == "on",
            bright: values[5].parse::<u8>().unwrap(),
            color_mode: values[6].parse::<u8>().unwrap(),
            ct: values[7].parse::<u16>().unwrap(),
            rgb: RGB::new(values[8].parse::<u32>().unwrap()),
            hue: values[9].parse::<u16>().unwrap(),
            sat: values[10].parse::<u8>().unwrap(),
            name: values[11].clone(),
            ip: Bulb::get_ip(search_response).unwrap()
        }
    }

    fn get_ip(response: &str) -> Option<String> {
        response
            .split("\r\n")
            .find(|line| line.contains("Location"))
            .map(|line| line.split("//").nth(1).unwrap().to_string())
    }

    fn get_param_value(response: &str, param: &str) -> Option<String> {
        let split = response.split("\r\n");
        for line in split {
            let mut line_split = line.split(": ");
            if line_split.next().unwrap().contains(param) {
                return Some(line_split.next().unwrap().to_string());
            }
        }
        return None;
    }
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