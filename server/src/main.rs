#![warn(clippy::pedantic)]
#![allow(clippy::too_many_lines)]

use anyhow::Result;
use bytes::{BufMut, Bytes, BytesMut};
use std::{
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream, UdpSocket},
    thread,
    time::Duration,
};

use types::{
    parser::{BytesBuf, Parsable},
    serializer::Serializable,
    Domain, Header, Message, OpCode, Question, RecordClass, RecordType, ResCode,
};

static ROOT_SOURCE: SocketAddr =
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(192, 41, 162, 30), 53));

const UDP_MAX_SIZE: usize = 512;

fn make_request(question: Question, source: SocketAddr) -> Result<Message> {
    let mut msg_buf = BytesMut::new();
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

// TODO: move error message and NXDOMAIN logic out of this func
// by returning Result<Option<Message>> and having an outer func
// that turns that into a Message
fn resolve_domain(
    id: u16,
    request: Domain,
    qtype: RecordType,
    qclass: RecordClass,
    source: SocketAddr,
) -> Result<Option<Message>> {
    let res = make_request(
        Question {
            name: request.clone(),
            qtype,
            qclass,
        },
        source,
    )?;

    if res.header.answer_records > 0 {
        return Ok(Some(Message {
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
        }));
    }

    if res.header.authority_records > 0 && res.header.additional_records > 0 {
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

        if authority_sources.is_empty() {
            return Ok(None);
        }

        // TODO: maybe backtrack and try a different authority if one returns NXDOMAIN
        return resolve_domain(
            id,
            request,
            qtype,
            qclass,
            SocketAddr::V4(SocketAddrV4::new(authority_sources[0], 53)),
        );
    }

    Ok(None)
}

fn _recursive_resolve(
    transport: &'static str,
    mut data: BytesBuf,
) -> std::result::Result<Message, (Option<u16>, anyhow::Error)> {
    let mut msg = match Message::parse(&mut data) {
        Ok(msg) => msg,
        Err(err) => return Err((None, err.into())),
    };

    if msg.header.questions != 1 || !msg.header.should_recurse {
        return Ok(Message {
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
        });
    }

    let q = msg.questions.remove(0);
    println!("New {transport} lookup for: {}", q.name);

    match resolve_domain(msg.header.id, q.name, q.qtype, q.qclass, ROOT_SOURCE) {
        Ok(res) => match res {
            Some(msg) => Ok(msg),
            None => Ok(Message {
                header: Header {
                    id: msg.header.id,
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
            }),
        },
        Err(err) => Err((Some(msg.header.id), err)),
    }
}

fn recursive_resolve(transport: &'static str, data: BytesBuf) -> Option<Message> {
    match _recursive_resolve(transport, data) {
        Ok(msg) => Some(msg),
        Err((id, err)) => {
            eprintln!("Error when making request, propogating to client: {err}");

            if let Some(id) = id {
                Some(Message {
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
                })
            } else {
                eprintln!(
                    "Couldn't even parse message id from data, so can't send client the error :/"
                );
                None
            }
        }
    }
}

fn udp_server() -> Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:8080")?;

    loop {
        // https://www.rfc-editor.org/std/std75.txt
        // "The maximum allowable size of a DNS message over UDP not using the extensions described in this document is 512 bytes."
        let mut data = [0; UDP_MAX_SIZE];

        let (_, addr) = socket.recv_from(&mut data)?;

        if data.is_empty() {
            continue;
        }

        let mut buf = BytesMut::new();

        if let Some(msg) = recursive_resolve("UDP", BytesBuf::new(data.into())) {
            msg.serialize(&mut buf)?;

            let slice = if buf.len() > UDP_MAX_SIZE {
                // Set is_truncated to true
                buf[2] |= 0x02;
                println!("UDP request was truncated...");
                &buf[0..UDP_MAX_SIZE]
            } else {
                &buf
            };

            socket.send_to(slice, addr)?;
        }
    }
}

fn stream_handler(mut stream: TcpStream) -> Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(60)))?;
    // These 2 pesky bytes only mentioned once in RFC 1035
    let mut size = [0; 2];
    stream.read_exact(&mut size)?;

    let size = u16::from_be_bytes(size) as usize;

    let mut data = vec![0; size];
    stream.read_exact(&mut data)?;

    let mut buf = BytesMut::new();

    if let Some(msg) = recursive_resolve("TCP", BytesBuf::new(data)) {
        msg.serialize(&mut buf)?;

        #[allow(clippy::cast_possible_truncation)]
        stream.write_all(&u16::to_be_bytes(buf.len() as u16))?;

        stream.write_all(&buf)?;
    }

    Ok(())
}

fn tcp_server() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    for stream in listener.incoming() {
        let stream = stream?;

        thread::spawn(move || {
            stream_handler(stream).expect("TODO: deal with this");
        });
    }

    Ok(())
}

fn main() -> Result<()> {
    let tcp = thread::spawn(|| tcp_server().unwrap());

    // For now don't handle UDP
    thread::spawn(udp_server).join().unwrap()?;
    tcp.join().unwrap();

    Ok(())
}
