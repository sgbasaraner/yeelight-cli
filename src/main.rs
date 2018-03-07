use std::str;
use std::thread;
use std::net::UdpSocket;

const MULTICAST_ADDR: &'static str = "239.255.255.250:1982";

fn main() {
    let socket = match UdpSocket::bind("0.0.0.0:34254") {
        Ok(s) => s,
        Err(e) => panic!("couldn't bind socket: {}", e)
    };

    send_search_broadcast(&socket);
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
    loop {
        match socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                thread::spawn(move || {
                    println!("amt: {}", amt);
                    println!("src: {}", src);
                });
                process_search_response(str::from_utf8(&buf).unwrap_or(""))
            },
            Err(e) => {
                println!("couldn't receive a datagram: {}", e);
            }
        }
    }
}

fn process_search_response(response: &str) {
    println!("{}", response);
}