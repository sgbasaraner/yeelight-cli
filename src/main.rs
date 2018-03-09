mod bulb;

use std::str;
use std::{thread, time};
use std::net::{TcpStream, UdpSocket};
use std::sync::mpsc::{Sender, Receiver, channel, TryRecvError};
use bulb::{Bulb, RGB};
use std::io::Write;

const MULTICAST_ADDR: &'static str = "239.255.255.250:1982";

fn main() {
    let socket = create_socket();
    send_search_broadcast(&socket);
    let mut bulbs: Vec<Bulb> = Vec::new();
    let (sender, receiver): (Sender<Bulb>, Receiver<Bulb>) = channel();
    thread::spawn(move || {
        let mut buf = [0; 2048];
        loop {
            match socket.recv_from(&mut buf) {
                Ok((amt, src)) => {
                    println!("amt: {}", amt);
                    println!("src: {}", src);
                    let _ = sender.send(process_search_response(str::from_utf8(&buf).unwrap()));
                },
                Err(e) => {
                    println!("Couldn't receive a datagram: {}", e);
                    break;
                }
            }
            thread::sleep(time::Duration::from_millis(200));
        }
    });
    thread::sleep(time::Duration::from_secs(2));
    loop {
        match receiver.try_recv() {
            Ok(b) => bulbs.push(b),
            Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => break,
        }
    }
    bulbs = remove_duplicates(bulbs);
    println!("{:?}", bulbs);
    let mut current_command_id: u32 = 0;
    for i in 0..10 {
        operate_on_bulb(&mut current_command_id, &bulbs[0], "set_bright", &(i * 10).to_string()[..]);
        thread::sleep(time::Duration::from_secs(1));
    }
}

fn send_search_broadcast(socket: &UdpSocket) {
    let message = 
                    "M-SEARCH * HTTP/1.1\r\n
                    HOST: 239.255.255.250:1982\r\n
                    MAN: \"ssdp:discover\"\r\n
                    ST: wifi_bulb".as_bytes();

    socket.send_to(message, MULTICAST_ADDR).expect("couldn't send to socket");
}

fn process_search_response(response: &str) -> Bulb {
    let params = ["id", "model", "fw_ver", "support", "power", "bright", "color_mode", "ct", "rgb", "hue", "sat", "name"];
    let mut values = Vec::new();
    for i in 0..12 {
        values.push(get_param_value(response, params[i]).unwrap());
    }
    let mut power = false;
    if values[4] == "on" { power = true; }
    Bulb {
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
        name: values[11].clone(),
        ip: get_ip(response).unwrap()
    }
}

fn create_socket() -> UdpSocket {
    match UdpSocket::bind("0.0.0.0:34254") {
        Ok(s) => { return s },
        Err(e) => panic!("couldn't bind socket: {}", e)
    };
}

fn get_ip(response: &str) -> Option<String> {
    let split = response.split("\r\n");
    for line in split {
        if line.contains("Location") {
            let vec = line.split("//").collect::<Vec<&str>>();
            return Some(String::from(vec[1]));
        }
    }
    return None;
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

fn remove_duplicates(bulbs: Vec<Bulb>) -> Vec<Bulb> {
    let mut new = Vec::new();
    let mut ids = Vec::new();
    for bulb in bulbs {
        if ids.contains(&bulb.id) { continue }
        ids.push(bulb.id.clone());
        new.push(bulb);
    }
    new
}

fn get_next_cmd(cur: &mut u32) -> &u32 {
    *cur += 1;
    cur
}

fn operate_on_bulb(cur: &mut u32, bulb: &Bulb, method: &str, params: &str) {
    let ip = &bulb.ip.to_owned()[..];
    let mut stream = TcpStream::connect(ip).expect("Couldn't start the stream.");
    let mut message = String::new();
    message.push_str("{\"id\":");
    message.push_str(&get_next_cmd(cur).to_string()[..]);
    message.push_str(",\"method\":\"");
    message.push_str(method);
    message.push_str("\",\"params\":[");
    message.push_str(params);
    message.push_str("]}\r\n");
    println!("{}", message);
    let ip = &bulb.ip.to_owned()[..];
    println!("{}", ip);
    stream.write(message.as_bytes()).expect("Couldn't send to the stream");
}