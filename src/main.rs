use std::{io, net::UdpSocket};

use parser::{BytesBuf, Parsable};
use types::Message;

mod parser;
mod serializer;
mod types;

fn main() -> io::Result<()> {
    {
        let socket = UdpSocket::bind("127.0.0.1:8080")?;

        loop {
            let mut buf = [0; 1024];

            let _ = socket.recv_from(&mut buf)?;

            if buf.is_empty() {
                continue;
            }

            let msg = Message::parse(&mut BytesBuf::new(buf.into())).unwrap();
            println!("{msg:#?}");
        }
    }
}
