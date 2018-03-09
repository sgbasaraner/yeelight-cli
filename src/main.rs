use std::str;
use std::{thread, time};
use std::net::UdpSocket;

const MULTICAST_ADDR: &'static str = "239.255.255.250:1982";

fn main() {
    let socket = create_socket();
    detect_bulbs(&socket);
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
    let mut time_elapsed: i32 = 0;
    let search_interval: i32 = 300;
    let read_interval: i32 = 1;

    loop {
        if time_elapsed % search_interval == 0 {
            send_search_broadcast(socket);
        }
        match socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                thread::spawn(move || {
                    println!("amt: {}", amt);
                    println!("src: {}", src);
                });
                process_search_response(str::from_utf8(&buf).unwrap_or(""));
                let sleep_time = time::Duration::from_millis((read_interval as f32 / 10.0) as u64 * 1000);
                thread::sleep(sleep_time);
            },
            Err(e) => {
                println!("couldn't receive a response: {}", e);
                break;
            }
        }
        time_elapsed += read_interval;
        println!("{}", time_elapsed);
    }
}

fn process_search_response(response: &str) {
    println!("{}", response);
}

fn create_socket() -> UdpSocket {
    match UdpSocket::bind("0.0.0.0:34254") {
        Ok(s) => return s,
        Err(e) => panic!("couldn't bind socket: {}", e)
    };
}