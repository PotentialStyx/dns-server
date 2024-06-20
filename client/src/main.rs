use std::{
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream},
};

use anyhow::Result;
use bytes::{BufMut, Bytes, BytesMut};
use clap::Parser;
use parser::{BytesBuf, Parsable};
use serializer::Serializable;
use types::*;

static ROOT_SOURCE: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(1, 1, 1, 1), 53));

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

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    domain: String,

    // #[arg(value_enum)]
    record_type: Option<ArgRecordType>,

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

fn main() {
    let cli = Cli::parse();

    if let Some(rtype) = cli.record_type {
        println!(
            "Requesting all {} records for {}",
            rtype.to_string(),
            cli.domain
        );

        let mut domain = vec![];
        for part in cli.domain.clone().split('.') {
            domain.push(part.to_owned());
        }

        let res = make_request(
            Question {
                name: Domain(domain),
                qtype: rtype.into(),
                qclass: RecordClass::IN,
            },
            ROOT_SOURCE,
        )
        .unwrap();

        dbg!(res);
    } else {
        println!("Requesting all records for {}", cli.domain);
    }
}
