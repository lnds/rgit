use std::{str, num, string};
use std::io::{IoResult, TcpStream};

pub fn read_packet_line(socket: &mut TcpStream) -> IoResult<Option<String>> {
    let header = try!(socket.read_exact(4));
    let length_str = str::from_utf8(header.as_slice()).unwrap();
    let length: uint = num::from_str_radix(length_str, 16).unwrap();

    if length > 4 {
        let pkt = try!(socket.read_exact(length - 4));
        let parsed = string::String::from_utf8(pkt).unwrap();
        print!("packet: \t<git {}", parsed);
        Ok(Some(parsed))
    } else {
        print!("packet: \t<git {}", length_str);
        Ok(None)
    }
}

pub fn receive(socket: &mut TcpStream) -> IoResult<Vec<String>> {
    let mut lines = vec![];
    loop {
        match read_packet_line(socket) {
            Ok(Some(line)) => {
                lines.push(line);
            },
            Ok(None) => {
                return Ok(lines);
            },
            Err(e) => {
                return Err(e);
            }
        }
    }
}

// pub fn with_connection<T>(host: &str, port: u16, consumer: |&mut TcpStream| -> IoResult<T>) -> IoResult<T> {
//     match TcpStream::connect((host, port)) {
//         Ok(mut sock) => {consumer(&mut sock)},
//         Err(e) => Err(e)
//     }
// }