use anyhow::Result;
use bytes::{BufMut, Bytes, BytesMut};
use serializer::Serializable;
use std::{
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream, UdpSocket},
    thread,
};

use parser::{BytesBuf, Parsable};
use types::{Domain, Header, Message, OpCode, Question, RecordClass, RecordType, ResCode};

mod parser;
mod serializer;
mod types;

static ROOT_SOURCE: SocketAddr =
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(192, 41, 162, 30), 53));

fn make_request(question: Question, source: SocketAddr) -> Result<Message> {
    let mut _buf = BytesMut::new();
    Message {
        header: Header {
            id: 0,
            is_response: false,
            opcode: OpCode::Query,
            is_authoritative: false,
            is_truncated: false,
            should_recurse: false,
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
    .serialize(&mut _buf)?;

    let mut buf = BytesMut::new();
    buf.put_u16(_buf.len().try_into()?);
    buf.put(_buf);

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

fn resolve_domain(
    id: u16,
    request: Domain,
    qtype: RecordType,
    qclass: RecordClass,
    source: SocketAddr,
) -> Message {
    let res = match make_request(
        Question {
            name: request.clone(),
            qtype,
            qclass,
        },
        source,
    ) {
        Ok(res) => res,
        Err(err) => {
            eprintln!("Error when making request, propogating to client: {err}");

            return Message {
                header: Header {
                    id,
                    is_response: true,
                    opcode: OpCode::Query,
                    is_authoritative: false,
                    is_truncated: false,
                    should_recurse: false,
                    recursion_available: true,
                    _z: 0,
                    rescode: ResCode::ServerFailure,
                    questions: 0,
                    answer_records: 0,
                    authority_records: 0,
                    additional_records: 0,
                },
                questions: vec![],
                answers: vec![],
                authorities: vec![],
                additional: vec![],
            };
        }
    };

    dbg!(&res);

    if res.header.answer_records > 0 {
        return Message {
            header: Header {
                id,
                is_response: true,
                opcode: OpCode::Query,
                is_authoritative: false,
                is_truncated: false,
                should_recurse: false,
                recursion_available: true,
                _z: 0,
                rescode: ResCode::NoError,
                questions: 0,
                answer_records: res.header.answer_records,
                authority_records: 0,
                additional_records: 0,
            },
            questions: vec![],
            answers: res.answers,
            authorities: vec![],
            additional: vec![],
        };
    }

    if res.header.authority_records > 0 && res.header.additional_records > 0 {
        // dbg!(&res);
        let mut authority_sources = vec![];
        for authority in &res.authorities {
            if let Some(domain) = &authority.domain_data {
                for additional in &res.additional {
                    if additional.name == *domain && additional.rtype == RecordType::A {
                        authority_sources.push(Ipv4Addr::new(
                            additional.data[0],
                            additional.data[1],
                            additional.data[2],
                            additional.data[3],
                        ));
                    }
                }
            }
        }

        assert!(!authority_sources.is_empty());
        dbg!(&authority_sources);

        return resolve_domain(
            id,
            request,
            qtype,
            qclass,
            SocketAddr::V4(SocketAddrV4::new(authority_sources[0], 53)),
        );
    }

    Message {
        header: Header {
            id,
            is_response: true,
            opcode: OpCode::Query,
            is_authoritative: false,
            is_truncated: false,
            should_recurse: false,
            recursion_available: true,
            _z: 0,
            rescode: ResCode::NameError,
            questions: 0,
            answer_records: 0,
            authority_records: 0,
            additional_records: 0,
        },
        questions: vec![],
        answers: vec![],
        authorities: vec![],
        additional: vec![],
    }
}

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

        let mut msg = Message::parse(&mut BytesBuf::new(data))?;
        if msg.header.questions != 1 || !msg.header.should_recurse {
            let mut buf = BytesMut::new();
            Message {
                header: Header {
                    id: msg.header.id,
                    is_response: true,
                    opcode: types::OpCode::Query,
                    is_authoritative: false,
                    is_truncated: false,
                    should_recurse: false,
                    recursion_available: true,
                    _z: 0,
                    rescode: types::ResCode::Refused,
                    questions: 0,
                    answer_records: 0,
                    authority_records: 0,
                    additional_records: 0,
                },
                questions: vec![],
                answers: vec![],
                authorities: vec![],
                additional: vec![],
            }
            .serialize(&mut buf)?;
            stream.write_all(&buf)?;
        }
        println!("{msg:#?}");

        let q = msg.questions.remove(0);

        let res = resolve_domain(msg.header.id, q.name, q.qtype, q.qclass, ROOT_SOURCE);
        let mut buf = BytesMut::new();
        res.serialize(&mut buf)?;

        stream.write_all(&u16::to_be_bytes(buf.len() as u16))?;

        stream.write_all(&buf)?;
    }

    Ok(())
}

fn main() -> Result<()> {
    let tcp = thread::spawn(|| tcp_server().unwrap());

    // For now don't handle UDP
    // thread::spawn(udp_server).join().unwrap()?;
    tcp.join().unwrap();

    Ok(())
}
