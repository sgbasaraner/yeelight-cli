mod bulb;

#[macro_use] extern crate prettytable;
use prettytable::Table;

use std::str;
use std::process::exit;
use std::{thread, time};
use std::net::{TcpStream, UdpSocket};
use std::sync::mpsc::{Sender, Receiver, channel, TryRecvError};
use bulb::{Bulb, RGB};
use std::io::{self, Write, BufRead};

const MULTICAST_ADDR: &'static str = "239.255.255.250:1982";

fn main() {
    // Search for bulbs on a separate thread
    let socket = create_socket();
    send_search_broadcast(&socket);
    let mut bulbs: Vec<Bulb> = Vec::new();
    let (sender, receiver): (Sender<Bulb>, Receiver<Bulb>) = channel();
    thread::spawn(move || {
        let mut buf = [0; 2048];
        loop {
            match socket.recv_from(&mut buf) {
                Ok(_) => {
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

    // Give the other thread some time to find the bulbs
    thread::sleep(time::Duration::from_millis(1200));

    // Transfer the found bulbs to this thread
    loop {
        match receiver.try_recv() {
            Ok(b) => bulbs.push(b),
            Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => break,
        }
    }

    if bulbs.len() == 0 {
        println!("No bulbs found.");
        exit(1);
    }
    bulbs = remove_duplicates(bulbs);
    print_pretty_table(&bulbs);
    print_usage_instructions();
    // Main program loop
    loop {
        print!("Command: ");
        io::stdout().flush().unwrap();
        let stdin = io::stdin();
        let mut prompt = String::new();
        stdin.lock().read_line(&mut prompt).expect("Couldn't process the command.");
        if prompt.contains("quit") { break; }
        let mut current_operation_id = 0;
        let space_split = prompt.split(" ").collect::<Vec<&str>>();
        let bulb_index: usize = space_split[0].parse::<usize>().unwrap() - 1;
        let mut params = String::new();
        let mut tmp = 0;
        for arg in &space_split {
            if tmp < 2 { 
                tmp += 1;
                continue; 
            }
            params.push_str(&arg);
        }
        operate_on_bulb(&mut current_operation_id, &bulbs[bulb_index], &space_split[1], &params);
    }
}

fn print_pretty_table(bulbs: &Vec<Bulb>) {
    let mut id = 1;
    let mut table = Table::new();
    table.add_row(row!["ID", "NAME", "IP", "MODEL"]);
    for bulb in bulbs {
        table.add_row(row![id.to_string(), bulb.name, bulb.ip, bulb.model]);
        id += 1;
    }
    table.printstd();
}

fn print_usage_instructions() {
    println!(
        "To operate on bulbs, try prompting:
        bulb_id method param1 param2 param3 param4
        For example, you can try:
        1 set_power \"on\" \"smooth\" 500
        You can quit by typing quit.
        For a list of all available methods, you can check out: http://www.yeelight.com/download/Yeelight_Inter-Operation_Spec.pdf");
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
    // Parse params
    let mut parsed_params = String::new();
    let params_split = params.split(" ");
    for param in params_split {
        match param.parse::<i32>(){
            Ok(_) => parsed_params.push_str(param),
            Err(_) => {
                parsed_params.push_str("\"");
                parsed_params.push_str(param);
                parsed_params.push_str("\"");
            }
        };
        parsed_params.push_str(", ");
    }
    let new_len = parsed_params.len() - 2;
    parsed_params.truncate(new_len);

    // Send message to the bulb
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
    stream.write(message.as_bytes()).expect("Couldn't send to the stream");
}