use std::{
    io::{Read, Write},
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, TcpStream, UdpSocket},
};

use anyhow::{format_err, Result};
use bytes::{BufMut, Bytes, BytesMut};
use types::{
    parser::{BytesBuf, Parsable},
    serializer::Serializable,
    Header, Message, OpCode, Question, ResCode,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Transport {
    Tcp,
    Udp,
    Tls,
    Https,
    Unspecified,
}

pub fn make_request(
    question: Question,
    source: SocketAddr,
    transport: Transport,
) -> Result<Message> {
    let mut msg_buf = BytesMut::new();
    Message {
        header: Header {
            id: 0,
            is_response: false,
            opcode: OpCode::Query,
            is_authoritative: false,
            is_truncated: false,
            should_recurse: true,
            recursion_available: false,
            _z: 0,
            rescode: ResCode::NoError,
            questions: 1,
            answer_records: 0,
            authority_records: 0,
            additional_records: 0,
        },
        questions: vec![question],
        answers: vec![],
        authorities: vec![],
        additional: vec![],
    }
    .serialize(&mut msg_buf)?;

    match transport {
        Transport::Https | Transport::Tls | Transport::Unspecified => {
            Err(format_err!("WIP transport"))
        }
        Transport::Tcp => {
            let mut buf = BytesMut::new();

            buf.reserve(msg_buf.len() + 2);

            buf.put_u16(msg_buf.len().try_into()?);
            buf.put(msg_buf);

            let mut stream = TcpStream::connect(source)?;

            stream.write_all(&buf)?;

            let mut size = [0; 2];
            stream.read_exact(&mut size)?;

            let size = u16::from_be_bytes(size) as usize;

            let mut data = vec![0; size];
            stream.read_exact(&mut data)?;

            stream.shutdown(std::net::Shutdown::Both)?;

            let buf: Bytes = data.into();

            Ok(Message::parse(&mut BytesBuf::from_bytes(buf))?)
        }
        Transport::Udp => {
            let local_bind = match source {
                SocketAddr::V4(_) => SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)),
                SocketAddr::V6(_) => {
                    SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, 0, 0, 0))
                }
            };

            let socket = UdpSocket::bind(local_bind)?;

            socket.send_to(&msg_buf, source)?;

            let mut data = vec![0; 512];
            socket.recv(&mut data)?;

            drop(socket);

            let buf: Bytes = data.into();

            let ret = Message::parse(&mut BytesBuf::from_bytes(buf))?;

            if ret.header.is_truncated {
                Err(format_err!("Data was truncated, try again over TCP"))
            } else {
                Ok(ret)
            }
        }
    }
}
