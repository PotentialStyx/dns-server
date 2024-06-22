#![warn(clippy::pedantic, clippy::all)]
#![deny(clippy::unwrap_used)]

use std::{
    collections::{HashMap, HashSet},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
};

use bytes::{Buf, Bytes};
use clap::Parser;
use types::{
    parser::{BytesBuf, Parsable},
    Domain, Question, RecordClass, RecordType,
};
use utils::{make_request, Transport};

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

        impl std::fmt::Display for $name {
            fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                fmt.write_str(match self {
                    $($name::$field => stringify!($field),)*
                })
            }
        }
    };
}

record_type! {
    enum ArgRecordType {
        A,
        NS,
        MX,
        SOA,
        ANY,
        TXT,
        CAA,
        SVCB,
        AAAA,
        HTTPS,
        CNAME,
        HINFO,
    }
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

#[allow(clippy::too_many_lines)] // TODO: dont
fn format_data(
    rtype: RecordType,
    mut data: Bytes,
    domain: Option<Vec<Domain>>,
    after_ptr: Option<usize>,
) -> Option<String> {
    match rtype {
        RecordType::CNAME | RecordType::NS => Some(format!(
            "\"\x1b[0;95m{}\x1b[0m\"",
            domain
                .expect("This is garunteed to be Some(...) by the parser")
                .first()
                .expect("This is garunteed to be Some(...) by the parser")
        )),
        RecordType::MX => Some(format!(
            "{} \"\x1b[0;95m{}\x1b[0m\"",
            data.get_u16(),
            domain
                .expect("This is garunteed to be Some(...) by the parser")
                .first()
                .expect("This is garunteed to be Some(...) by the parser")
        )),
        RecordType::SOA => {
            let mut domains = domain.expect("This is garunteed to be Some(...) by the parser");

            let mname = domains.remove(0);

            let mut rname = domains.remove(0);
            let mut email = String::new();
            if !rname.0.is_empty() {
                let item = rname.0.remove(0);
                email += &item;
                email += "@";

                for item in rname.0 {
                    email += &item;
                    email += ".";
                }
            }

            data.advance(after_ptr.expect("This is garunteed to be Some(...) by the parser"));

            let email = &email[0..email.len() - 1];

            Some(format!(
                "\"\x1b[0;95m{}\x1b[0m\" {email} {} {} {} {} {}",
                mname.idna_to_string(),
                data.get_u32(),
                data.get_u32(),
                data.get_u32(),
                data.get_u32(),
                data.get_u32(),
            ))
        }
        RecordType::AAAA => Some(format!(
            "\x1b[0;35m{}\x1b[0m",
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
            "\x1b[0;35m{}\x1b[0m",
            Ipv4Addr::new(data[0], data[1], data[2], data[3])
        )),
        RecordType::TXT => {
            let mut res = "\"\x1b[0;32m".to_string();
            let mut length = data.get_u8() as usize;
            while length != 0 && data.len() >= length {
                match std::str::from_utf8(&data.slice(0..length)) {
                    Ok(data) => {
                        res += data;
                    }
                    Err(_err) => {
                        // err.
                        // // TODO: handle this
                        // eprintln!("uhoh - {err}");
                        // None
                        return Some("This record contained invalid utf-8".to_string());
                    }
                };

                data.advance(length);

                if data.is_empty() {
                    break;
                }

                length = data.get_u8() as usize;
            }

            Some(res + "\x1b[0m\"")
        }
        RecordType::SVCB | RecordType::HTTPS => {
            let priority = data.get_u16();

            let mut buf = BytesBuf::from_bytes(data);

            // OK because domain has to be UNCOMPRESSED according to rfc9460 section 2.2
            let name = Domain::parse(&mut buf).expect("TODO: deal with this");

            let mut data = buf.take();

            let mut attributes = HashMap::new();

            while !data.is_empty() {
                let key = data.get_u16();
                let value_len = data.get_u16() as usize;

                let value = data[0..value_len].to_vec();
                data.advance(value_len);

                attributes.insert(key, value);
            }

            let mut attributes_rendered = String::new();

            for (key, val) in attributes {
                attributes_rendered += " ";

                let mut data = Bytes::from(val);

                // See https://www.iana.org/assignments/dns-svcb/dns-svcb.xhtml
                match key {
                    1 => {
                        attributes_rendered += "alpn=\"";

                        while !data.is_empty() {
                            let len = data.get_u8() as usize;

                            match std::str::from_utf8(&data[0..len]) {
                                Ok(part) => attributes_rendered += part,
                                Err(_) => return None,
                            }

                            data.advance(len);
                            attributes_rendered += ",";
                        }

                        attributes_rendered =
                            attributes_rendered[0..attributes_rendered.len() - 1].to_string();

                        attributes_rendered += "\"";
                    }
                    // TODO: find a domain to test: port, ipv4hint, and ipv6hint rendering on
                    3 => {
                        attributes_rendered += "port=";
                        attributes_rendered += &data.get_u16().to_string();
                    }
                    4 => {
                        attributes_rendered += "ipv4hint=";

                        while !data.is_empty() {
                            attributes_rendered += &Ipv4Addr::new(
                                data.get_u8(),
                                data.get_u8(),
                                data.get_u8(),
                                data.get_u8(),
                            )
                            .to_string();
                            attributes_rendered += ",";
                        }

                        attributes_rendered = attributes_rendered[1..].to_string();
                    }
                    6 => {
                        attributes_rendered += "ipv6hint=";

                        while !data.is_empty() {
                            attributes_rendered += &Ipv6Addr::new(
                                data.get_u16(),
                                data.get_u16(),
                                data.get_u16(),
                                data.get_u16(),
                                data.get_u16(),
                                data.get_u16(),
                                data.get_u16(),
                                data.get_u16(),
                            )
                            .to_string();
                            attributes_rendered += ",";
                        }

                        attributes_rendered = attributes_rendered[1..].to_string();
                    }
                    _ => {}
                }
            }

            Some(format!(
                "{priority} {} {}",
                name.idna_to_string(),
                &attributes_rendered[1..]
            ))
        }
        RecordType::CAA => {
            let flags = data.get_u8();

            let critical = flags & 128 == 128;

            let tag_len = data.get_u8() as usize;
            let tag = std::str::from_utf8(&data[..tag_len]).expect("TODO: deal with this");

            let value = std::str::from_utf8(&data[tag_len..]).expect("TODO: deal with this");

            let critical_rendered = if critical {
                "\x1b[1;91mcritical\x1b[0m"
            } else {
                "\x1b[1;92mnormal\x1b[0m"
            };

            Some(format!("{critical_rendered} {tag} \"{value}\""))
        }
        RecordType::HINFO => {
            let mut res = "\"\x1b[0;32m".to_string();
            let length = data.get_u8() as usize;

            match std::str::from_utf8(&data.slice(0..length)) {
                Ok(data) => {
                    res += data;
                }
                Err(_err) => {
                    // err.
                    // // TODO: handle this
                    // eprintln!("uhoh - {err}");
                    // None
                    return Some("This record contained invalid utf-8".to_string());
                }
            };

            res += "\x1b[0m\" \"\x1b[0;32m";

            data.advance(length);

            let length = data.get_u8() as usize;

            match std::str::from_utf8(&data.slice(0..length)) {
                Ok(data) => {
                    res += data;
                }
                Err(_err) => {
                    // err.
                    // // TODO: handle this
                    // eprintln!("uhoh - {err}");
                    // None
                    return Some("This record contained invalid utf-8".to_string());
                }
            };

            Some(res + "\x1b[0m\"")
        }
        _ => {
            None //format!("{data:#?}")
        }
    }
}

fn ttl_to_string(ttl: u32) -> String {
    let seconds = ttl % 60;
    let minutes = ttl / 60;
    let hours = minutes / 60;
    let days = hours / 24;
    let years = days / 365; // Doesn't and won't account for leap years

    if hours == 0 {
        format!("{:0>2}m{seconds:0>2}s", minutes % 60)
    } else if days == 0 {
        format!("{:0>2}h{:0>2}m", hours % 24, minutes % 60)
    } else if years == 0 {
        format!("{:0>3}d{:0>2}h", days % 365, hours % 24)
    } else {
        format!("{years:0>3}y{:0>3}d", days % 365)
    }
}

fn main() {
    let cli = Cli::parse();
    let _no_color =
        (!atty::is(atty::Stream::Stdout) || std::env::var_os("NO_COLOR").is_some() || cli.no_color)
            && std::env::var_os("FORCE_COLOR").is_none();

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

    let qtype = if let Some(qtype) = cli.record_type {
        // println!(
        //     "Requesting all {} records for {} via {transport}",
        //     qtype.to_string(),
        //     cli.domain
        // );

        qtype.into()
    } else {
        // println!("Requesting all records for {} via {transport}", cli.domain);

        RecordType::ANY
    };

    let mut domain = vec![];
    for part in cli.domain.clone().split('.') {
        if !part.is_empty() {
            domain.push(part.to_owned());
        }
    }

    let default_port = match transport {
        Transport::Udp | Transport::Tcp | Transport::Unspecified => 53,
        Transport::Tls => 853,
        Transport::Https => 443,
        _ => unreachable!(),
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
        // println!("womp womp");
    }

    let mut unsupported = HashSet::new();
    // AUTHORITY
    // ANSWER
    // ADDITIONAL
    println!("SECTION    DOMAIN                  TTL      CLASS   TYPE    DATA");

    let mut records = vec![];

    for record in res.answers {
        records.push(("ANSWER    ", record));
    }

    for record in res.authorities {
        records.push(("AUTHORITY ", record));
    }

    for record in res.additional {
        records.push(("ADDITIONAL", record));
    }

    for (section, record) in records {
        if record.rclass != RecordClass::IN {
            // TODO: handle this
            println!("womp womp womp");
            continue;
        }

        //hackclub.com.           3789    IN      HINFO   "RFC8482" ""
        //google.com.             300     IN      A       74.125.142.139
        if let Some(display) = format_data(
            record.rtype,
            record.data,
            record.domain_data,
            record.after_ptr,
        ) {
            println!(
                "{section} \x1b[0;96m{:<23}\x1b[0;93m {:<8}\x1b[0m {:<7} {:<7} {}",
                record.name.idna_to_string(),
                ttl_to_string(record.ttl),
                format!("{:#?}", record.rclass),
                format!("{:#?}", record.rtype),
                display
            );
        } else {
            unsupported.insert(record.rtype);
        }
    }

    for rtype in unsupported {
        if let RecordType::Unknown(id) = rtype {
            println!("\x1b[0;91mUnsupported record type with id {id}");
        } else {
            println!("\x1b[0;91mUnsupported record type {rtype:#?}");
        }
    }
}
