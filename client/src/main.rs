#![warn(clippy::pedantic, clippy::all)]
#![deny(clippy::unwrap_used)]

use std::{
    io::{Read, Write},
    net::{
        IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, TcpStream, UdpSocket,
    },
};

use anyhow::{format_err, Result};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use clap::Parser;
use parser::{BytesBuf, Parsable};
use serializer::Serializable;
use types::{
    parser, serializer, Domain, Header, Message, OpCode, Question, RecordClass, RecordType, ResCode,
};

static DEFAULT_NAMESERVER: IpAddr = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));

macro_rules! record_type {
    (
        $vis:vis enum $name:ident {
            $($field:ident,)*
        }
    ) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        #[allow(clippy::upper_case_acronyms)]
        $vis enum $name {
            $($field,)*
        }

        impl $name {
            fn parse(arg: &str) -> anyhow::Result<$name> {
                match arg {
                    $(stringify!($field) => Ok($name::$field),)*
                    _ => Err(anyhow::format_err!("Invalid {}: \"{}\"", stringify!($name), arg)),
                }
            }
        }

        impl From<$name> for RecordType {
            fn from(s: $name) -> Self {
                match s {
                    $($name::$field => Self::$field,)*
                }
            }
        }

        impl ToString for $name {
            fn to_string(&self) -> String {
                match self {
                    $($name::$field => stringify!($field).to_string(),)*
                }
            }
        }
    };
}

record_type! {
    enum ArgRecordType {
        A,
        NS,
        ALL,
        AAAA,
        CNAME,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Transport {
    Tcp,
    Udp,
    Tls,
    Https,
    Unspecified,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[allow(clippy::struct_excessive_bools)]
struct Cli {
    domain: String,

    #[clap(value_parser=ArgRecordType::parse)]
    record_type: Option<ArgRecordType>,

    nameserver: Option<IpAddr>,

    #[clap(short = 'p', long)]
    port: Option<u16>,

    #[clap(
        long = "tcp",
        conflicts_with = "udp",
        conflicts_with = "tls",
        conflicts_with = "https"
    )]
    tcp: bool,
    #[clap(
        long = "udp",
        conflicts_with = "tcp",
        conflicts_with = "tls",
        conflicts_with = "https"
    )]
    udp: bool,
    #[clap(
        long = "tls",
        conflicts_with = "udp",
        conflicts_with = "tcp",
        conflicts_with = "https"
    )]
    tls: bool,
    #[clap(
        long = "https",
        conflicts_with = "tcp",
        conflicts_with = "tls",
        conflicts_with = "udp"
    )]
    https: bool,

    #[clap(long = "no-color")]
    no_color: bool,
}

fn make_request(question: Question, source: SocketAddr, transport: Transport) -> Result<Message> {
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

fn format_data(rtype: RecordType, mut data: Bytes, domain: Option<Domain>) -> Option<String> {
    match rtype {
        RecordType::CNAME | RecordType::NS => Some(format!(
            "{}",
            domain.expect("This is garunteed to be Some(...) by the parser")
        )),
        RecordType::AAAA => Some(format!(
            "{}",
            Ipv6Addr::new(
                data.get_u16(),
                data.get_u16(),
                data.get_u16(),
                data.get_u16(),
                data.get_u16(),
                data.get_u16(),
                data.get_u16(),
                data.get_u16()
            )
        )),
        RecordType::A => Some(format!(
            "{}",
            Ipv4Addr::new(data[0], data[1], data[2], data[3])
        )),
        RecordType::TXT => {
            match std::str::from_utf8(&data) {
                Ok(data) => Some(format!("\"{data}\"")),
                Err(err) => {
                    // TODO: handle this
                    eprintln!("uhoh - {err}");
                    None
                }
            }
        }
        _ => {
            None //format!("{data:#?}")
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let _no_color =
        (!atty::is(atty::Stream::Stdout) || std::env::var_os("NO_COLOR").is_some() || cli.no_color)
            && std::env::var_os("FORCE_COLOR").is_none();

    let qtype = if let Some(qtype) = cli.record_type {
        println!(
            "Requesting all {} records for {}",
            qtype.to_string(),
            cli.domain
        );

        qtype.into()
    } else {
        println!("Requesting all records for {}", cli.domain);

        RecordType::ALL
    };

    let mut domain = vec![];
    for part in cli.domain.clone().split('.') {
        domain.push(part.to_owned());
    }

    let transport = if cli.https {
        Transport::Https
    } else if cli.tls {
        Transport::Tls
    } else if cli.udp {
        Transport::Udp
    } else if cli.tcp {
        Transport::Tcp
    } else {
        Transport::Unspecified
    };

    let default_port = match transport {
        Transport::Udp | Transport::Tcp | Transport::Unspecified => 53,
        Transport::Tls => 853,
        Transport::Https => 443,
    };

    let port = match cli.port {
        Some(port) => port,
        None => default_port,
    };

    let nameserver_ip = cli.nameserver.unwrap_or(DEFAULT_NAMESERVER);

    let source = match nameserver_ip {
        IpAddr::V4(v4) => SocketAddr::V4(SocketAddrV4::new(v4, port)),
        IpAddr::V6(v6) => SocketAddr::V6(SocketAddrV6::new(v6, port, 0, 0)),
    };

    let res = match make_request(
        Question {
            name: Domain(domain),
            qtype,
            qclass: RecordClass::IN,
        },
        source,
        transport,
    ) {
        Ok(res) => res,
        Err(err) => {
            // TODO: handle this
            eprintln!("{err}");
            return;
        }
    };

    if res.answers.is_empty() {
        // TODO: handle this
        println!("womp womp");
    }

    for record in res.answers {
        if record.rclass != RecordClass::IN {
            // TODO: handle this
            println!("womp womp womp");
            continue;
        }

        if let Some(display) = format_data(record.rtype, record.data, record.domain_data) {
            println!(
                "{} {:#?} {:#?} {}",
                record.name, record.rtype, record.rclass, display
            );
        }
    }
}
