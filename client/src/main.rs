use std::{
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, TcpStream},
};

use anyhow::Result;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use clap::Parser;
use parser::{BytesBuf, Parsable};
use serializer::Serializable;
use types::*;

static DEFAULT_NAMESERVER: IpAddr = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));

macro_rules! record_type {
    (
        $vis:vis enum $name:ident[$unknown:ident] {
            $($field:ident,)*
        }
    ) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        #[allow(clippy::upper_case_acronyms)]
        $vis enum $name {
            $($field,)*
            $unknown(String)
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                match s.as_str() {
                    $(stringify!($field) => Self::$field,)*
                    _ => Self::$unknown(s),
                }
            }
        }

        impl From<$name> for RecordType {
            fn from(s: $name) -> Self {
                match s {
                    $($name::$field => Self::$field,)*
                    $name::$unknown(_) => Self::Unknown(0),
                }
            }
        }

        impl ToString for $name {
            fn to_string(&self) -> String {
                match self {
                    $($name::$field => stringify!($field).to_string(),)*
                    $name::$unknown(inner) => inner.clone(),
                }
            }
        }
    };
}

record_type! {
    enum ArgRecordType[Unknown] {
        A,
        NS,
        ALL,
        AAAA,
        CNAME,
    }
}

// impl TryFrom<String> for ArgRecordType {
//     type Error = io::Error;

//     fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
//         todo!()
//     }
// }

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    domain: String,

    // #[clap(parse(try_from_str))]
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

fn make_request(question: Question, source: SocketAddr) -> Result<Message> {
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

    let mut buf = BytesMut::new();
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

fn format_data(rtype: RecordType, mut data: Bytes, domain: Option<Domain>) -> String {
    match rtype {
        RecordType::CNAME | RecordType::NS => {
            format!("{}", domain.unwrap())
        }
        RecordType::AAAA => {
            format!(
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
            )
        }
        RecordType::A => {
            format!("{}", Ipv4Addr::new(data[0], data[1], data[2], data[3]))
        }
        RecordType::TXT => {
            format!(
                "\"{}\"",
                std::str::from_utf8(&data).expect("TODO: deal w/ this")
            )
        }
        _ => {
            format!("{data:#?}")
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let no_color =
        (!atty::is(atty::Stream::Stdout) || std::env::var_os("NO_COLOR").is_some() || cli.no_color)
            && std::env::var_os("FORCE_COLOR").is_none();

    // TODO: find a way to remove the Unknown param type
    // clap is kinda annoying
    assert!(!matches!(cli.record_type, Some(ArgRecordType::Unknown(..))));

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

    // TODO: change based on protocol
    let default_port = 53;

    let port = match cli.port {
        Some(port) => port,
        None => default_port,
    };

    let nameserver_ip = cli.nameserver.unwrap_or(DEFAULT_NAMESERVER);

    let source = match nameserver_ip {
        IpAddr::V4(v4) => SocketAddr::V4(SocketAddrV4::new(v4, port)),
        IpAddr::V6(v6) => SocketAddr::V6(SocketAddrV6::new(v6, port, 0, 0)),
    };

    let res = make_request(
        Question {
            name: Domain(domain),
            qtype,
            qclass: RecordClass::IN,
        },
        source,
    )
    .unwrap();

    if res.answers.is_empty() {
        println!("womp womp");
    }

    for record in res.answers {
        if record.rclass != RecordClass::IN {
            println!("womp womp womp");
            continue;
        }
        println!(
            "{} {:#?} {:#?} {}",
            record.name,
            record.rtype,
            record.rclass,
            format_data(record.rtype, record.data, record.domain_data)
        );
    }
}
