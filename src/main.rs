use anyhow::Result;
use std::{
    io::Read,
    net::{TcpListener, UdpSocket},
    thread,
};

use parser::{BytesBuf, Parsable};
use types::Message;

mod parser;
mod serializer;
mod types;

fn udp_server() -> Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:8080")?;

    loop {
        // https://www.rfc-editor.org/std/std75.txt
        // "The maximum allowable size of a DNS message over UDP not using the extensions described in this document is 512 bytes."
        let mut buf = [0; 512];

        let _ = socket.recv_from(&mut buf)?;

        if buf.is_empty() {
            continue;
        }

        let msg = Message::parse(&mut BytesBuf::new(buf.into()))?;
        println!("{msg:#?}");
    }
}

fn tcp_server() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    for stream in listener.incoming() {
        let mut stream = stream?;

        // These 2 pesky bytes only mentioned once in RFC 1035
        let mut size = [0; 2];
        stream.read_exact(&mut size)?;

        let size = u16::from_be_bytes(size) as usize;

        let mut data = vec![0; size];
        stream.read_exact(&mut data)?;

        if data.is_empty() {
            continue;
        }

        let msg = Message::parse(&mut BytesBuf::new(data))?;
        println!("{msg:#?}");
    }

    Ok(())
}

fn main() -> Result<()> {
    let tcp = thread::spawn(|| tcp_server().unwrap());

    thread::spawn(udp_server).join().unwrap()?;
    tcp.join().unwrap();

    Ok(())
}
