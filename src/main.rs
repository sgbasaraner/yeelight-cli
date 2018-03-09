mod bulb;

use std::str;
use std::{thread, time};
use std::net::UdpSocket;
use bulb::{Bulb, RGB};

const MULTICAST_ADDR: &'static str = "239.255.255.250:1982";

fn main() {
    let socket = create_socket();
    send_search_broadcast(&socket);
    thread::spawn(move || {detect_bulbs(&socket); });
    thread::sleep(time::Duration::from_secs(2));
}

fn send_search_broadcast(socket: &UdpSocket) {
    let message = 
                    "M-SEARCH * HTTP/1.1\r\n
                    HOST: 239.255.255.250:1982\r\n
                    MAN: \"ssdp:discover\"\r\n
                    ST: wifi_bulb".as_bytes();

    socket.send_to(message, MULTICAST_ADDR).expect("couldn't send to socket");
}

fn detect_bulbs(socket: &UdpSocket) {
    let mut buf = [0; 2048];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                    println!("amt: {}", amt);
                    println!("src: {}", src);
                    process_search_response(str::from_utf8(&buf).unwrap_or(""));
            },
            Err(e) => {
                println!("Couldn't receive a datagram: {}", e);
                break;
            }
        }
        thread::sleep(time::Duration::from_millis(200));
    }
}

fn process_search_response(response: &str) {
    let params = ["id", "model", "fw_ver", "support", "power", "bright", "color_mode", "ct", "rgb", "hue", "sat", "name"];
    let mut values = Vec::new();
    for param in params.iter() {
        values.push(get_param_value(response, param).unwrap())
    }
    let mut power = false;
    if values[4] == "on" { power = true; }
    let bulb = Bulb {
        id: values[0].clone(),
        model: values[1].clone(),
        fw_ver: values[2].parse::<u16>().unwrap(),
        support: values[3].clone(),
        power: power,
        bright: values[5].parse::<u8>().unwrap(),
        color_mode: values[6].parse::<u8>().unwrap(),
        ct: values[7].parse::<u16>().unwrap(),
        rgb: parse_rgb(values[8].parse::<u32>().unwrap()),
        hue: values[9].parse::<u16>().unwrap(),
        sat: values[10].parse::<u8>().unwrap(),
        name: values[11].clone()
    };
    
}

fn create_socket() -> UdpSocket {
    match UdpSocket::bind("0.0.0.0:34254") {
        Ok(s) => { return s },
        Err(e) => panic!("couldn't bind socket: {}", e)
    };
}

fn get_param_value(response: &str, param: &str) -> Option<String> {
    let split = response.split("\r\n");
    for line in split {
        let vec = line.split(": ").collect::<Vec<&str>>();
        if vec[0].contains(param) {
            return Some(String::from(vec[1]));
        }
    }
    return None;
}

fn parse_rgb(int: u32) -> RGB {
    RGB {
        r: ((int >> 16) & 0xFF) as u8,
        g: ((int >> 8) & 0xFF) as u8,
        b: ((int >> 0) & 0xFF) as u8
    }
}