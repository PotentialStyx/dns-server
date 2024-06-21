use std::{
    fmt::Display,
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Transport {
    Tcp,
    Udp,
    Tls,
    Https,
    Unspecified,
}

impl Display for Transport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Transport::Unspecified => "Unspecified",
            Transport::Https => "HTTPS",
            Transport::Tls => "TLS",
            Transport::Tcp => "TCP",
            Transport::Udp => "UDP",
        })
    }
}

fn make_tcp_req(data: Bytes, source: SocketAddr) -> Result<Message> {
    let mut buf = BytesMut::new();

    buf.reserve(data.len() + 2);

    buf.put_u16(data.len().try_into()?);
    buf.put(data);

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

fn make_udp_req(data: &Bytes, source: SocketAddr) -> Result<Option<Message>> {
    let local_bind = match source {
        SocketAddr::V4(_) => SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)),
        SocketAddr::V6(_) => SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, 0, 0, 0)),
    };

    let socket = UdpSocket::bind(local_bind)?;

    socket.send_to(data, source)?;

    let mut data = vec![0; 512];
    socket.recv(&mut data)?;

    drop(socket);

    let buf: Bytes = data.into();

    let ret = Message::parse(&mut BytesBuf::from_bytes(buf))?;

    if ret.header.is_truncated {
        Ok(None) // Err(format_err!("Data was truncated, try again over TCP"))
    } else {
        Ok(Some(ret))
    }
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

    let data: Bytes = msg_buf.into();

    match transport {
        Transport::Https | Transport::Tls => Err(format_err!("WIP transport")),
        Transport::Tcp => make_tcp_req(data, source),
        Transport::Udp => {
            let ret = make_udp_req(&data, source)?;

            if let Some(response) = ret {
                Ok(response)
            } else {
                Err(format_err!("Data was truncated, try again over TCP"))
            }
        }
        Transport::Unspecified => {
            // TODO: log error when tracing is setup
            if let Ok(Some(response)) = make_udp_req(&data, source) {
                Ok(response)
            } else {
                make_tcp_req(data, source)
            }
        }
    }
}
