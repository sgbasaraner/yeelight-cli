mod bulb;

#[macro_use] extern crate prettytable;
use prettytable::Table;

use std::str;
use std::env;
use std::process::exit;
use std::{thread, time};
use std::net::{TcpStream, UdpSocket};
use std::sync::mpsc::{Sender, Receiver, channel};
use bulb::Bulb;
use std::io::{self, Write, BufRead, Read};

const MULTICAST_ADDR: &str = "239.255.255.250:1982";

fn main() {
    // Search for bulbs on a separate thread
    let socket = create_socket();
    send_search_broadcast(&socket);
    let receiver = find_bulbs(socket);
    
    let bulbs: Vec<Bulb> = remove_duplicates(receiver.try_iter().collect());

    if bulbs.is_empty() {
        println!("No bulbs found.");
        exit(1);
    }

    // Deal with command line usage
    if perform_command_line_ops(&bulbs) {
        return;
    }

    print_pretty_table(&bulbs);
    print_usage_instructions();

    start_program_loop(&bulbs);
}

fn start_program_loop(bulbs: &[Bulb]) {
    // Main program loop
    let mut current_operation_id = 0;
    print!("Command: ");
    io::stdout().flush().unwrap();

    for prompt in io::stdin().lock().lines().map(Result::unwrap) {
        let operation_result: Result<(), &str> = match prompt.trim() {
            "quit" => break,
            "print" => {
                print_bulb_details(bulbs);
                Ok(())
            },
            prompt => { 
                let prompt_components = prompt.split(' ').collect::<Vec<&str>>();
                match prompt_components.as_slice() {
                    [bulb_index, method, params @ ..] => match bulb_index.parse::<usize>() {
                        Ok(r) if 1 <= r && r <= bulbs.len() => {
                            operate_on_bulb(&current_operation_id, &bulbs[r - 1], method, &parse_params(&params.join(" ")));
                            current_operation_id += 1;
                            Ok(())
                        },
                        _ => Err("Invalid command or bulb id.")
                    },
                    _ => Err("Please input at least 2 arguments.")
                }
            }
        };

        if let Err(msg) = operation_result {
             println!("{}", msg);
        }
        print!("Command: ");
        io::stdout().flush().unwrap();
    }
}

fn perform_command_line_ops(bulbs: &[Bulb]) -> bool {
    // Returns true if an operation was performed
    let args: Vec<String> = env::args().collect();
    if args.len() <= 2 {
        return false
    }
    let bulb_name = &args[1];
    let method_name = &args[2];
    for bulb in bulbs {
        if bulb.name != *bulb_name { continue; }
        let mut params = String::new();
        if args.len() > 3 {
            params.reserve(args.len() * 2); // at least 2 characters per arg
            for arg in args.iter().skip(3) {
                params.push_str(arg);
                params.push_str(" ");
            }
            let new_len = params.len() - 1;
            params.truncate(new_len); // get rid of trailing whitespace
            params = parse_params(&params);
        }
        operate_on_bulb(&0, &bulb, method_name, &params);
        return true;
    }
    return false
}

fn find_bulbs(socket: UdpSocket) -> Receiver<Bulb> {
    let (sender, receiver): (Sender<Bulb>, Receiver<Bulb>) = channel();
    thread::spawn(move || {
        let mut buf = [0; 2048];
        loop {
            match socket.recv_from(&mut buf) {
                Ok(_) => {
                    let _ = sender.send(Bulb::new(str::from_utf8(&buf).unwrap()));
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
    receiver
}

fn parse_params(params: &str) -> String {
    // Parses params, allowing the user to input on instead of "on"
    let mut parsed_params = String::new();
    let params_split = params.split(' ');
    for param in params_split {
        // Check if param is an integer or not
        match param.parse::<i32>() {
            Ok(_) => parsed_params.push_str(param),
            Err(_) => {
                parsed_params.push_str("\"");
                parsed_params.push_str(param);
                parsed_params.push_str("\"");
            }
        };
        parsed_params.push_str(", ");
    }
    let new_len = parsed_params.len() - 2; // get rid of the trailing ", "
    parsed_params.truncate(new_len);
    parsed_params
}

fn print_pretty_table(bulbs: &[Bulb]) {
    let mut id = 1;
    let mut table = Table::new();
    table.add_row(row!["ID", "NAME", "IP", "MODEL"]);
    for bulb in bulbs {
        table.add_row(row![id.to_string(), bulb.name, bulb.ip, bulb.model]);
        id += 1;
    }
    table.printstd();
}

fn print_bulb_details(bulbs: &[Bulb]) {
    println!("Warning: Bulb details may be outdated."); // TODO: fix this
    let mut table = Table::new();
    // This also does not print support variable
    table.add_row(row!["UNIQUE ID", "MODEL", "FW VER", "POWER", "BRIGHT", "COLOR MODE", "CT", "RGB", "HUE", "SAT", "NAME", "IP"]);
    for b in bulbs {
        table.add_row(row![b.id, b.model, b.fw_ver, b.power, b.bright, b.color_mode, b.ct, b.rgb, b.hue, b.sat, b.name, b.ip]);
    }
    table.printstd();
}

fn print_usage_instructions() {
    println!(
        "To operate on bulbs, try prompting without using double quotes:
        bulb_id method param1 param2 param3 param4
        For example, you can try:
        1 set_power on smooth 500
        You can quit by typing quit.
        You can print bulb details by typing print.
        For a list of all available methods, you can check out: http://www.yeelight.com/download/Yeelight_Inter-Operation_Spec.pdf");
}

fn send_search_broadcast(socket: &UdpSocket) {
    let message = b"M-SEARCH * HTTP/1.1\r\n
                    HOST: 239.255.255.250:1982\r\n
                    MAN: \"ssdp:discover\"\r\n
                    ST: wifi_bulb";

    socket.send_to(message, MULTICAST_ADDR).expect("Couldn't send to socket");
}

fn create_socket() -> UdpSocket {
    match UdpSocket::bind("0.0.0.0:34254") {
        Ok(s) => s,
        Err(e) => panic!("couldn't bind socket: {}", e)
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

fn create_message(id: &u32, method: &str, params: &str) -> String {
    let strs = [
        "{\"id\":",
        &id.to_string()[..],
        ",\"method\":\"",
        method,
        "\",\"params\":[",
        params,
        "]}\r\n"
    ];
    strs.join("")
}

fn operate_on_bulb(id: &u32, bulb: &Bulb, method: &str, params: &str) {
    // Send message to the bulb
    let message = create_message(id, method, params);

    let ip = &bulb.ip.to_owned()[..];
    let mut stream = TcpStream::connect(ip).expect("Couldn't start the stream.");
    match stream.write(message.as_bytes()) {
        Ok(_) => {
            print!("The message sent to the bulb is: {}", message);
            io::stdout().flush().unwrap();
        },
        Err(_) => {
            println!("Couldn't send to the stream");
            return;
        }
    }
    let mut buf = [0; 2048];
    match stream.read(&mut buf) {
        Ok(_) => {
            print!("The bulb returns: {}", str::from_utf8(&buf).unwrap());
            io::stdout().flush().unwrap();
        },
        Err(_) => {
            println!("Couldn't read from the stream.");
        }
    }
}
