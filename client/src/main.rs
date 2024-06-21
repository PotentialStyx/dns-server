#![warn(clippy::pedantic, clippy::all)]
#![deny(clippy::unwrap_used)]

use std::{
    collections::HashSet,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
};

use bytes::{Buf, Bytes};
use clap::Parser;
use types::{Domain, Question, RecordClass, RecordType};
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
        ANY,
        TXT,
        AAAA,
        CNAME,
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

fn format_data(rtype: RecordType, mut data: Bytes, domain: Option<Domain>) -> Option<String> {
    match rtype {
        RecordType::CNAME | RecordType::NS => Some(format!(
            "\"\x1b[0;95m{}\x1b[0m\"",
            domain.expect("This is garunteed to be Some(...) by the parser")
        )),
        RecordType::MX => Some(format!(
            "{} \"\x1b[0;95m{}\x1b[0m\"",
            data.get_u16(),
            domain.expect("This is garunteed to be Some(...) by the parser")
        )),
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
            match std::str::from_utf8(&data) {
                Ok(data) => Some(format!("\"\x1b[0;32m{data}\x1b[0m\"")),
                Err(err) => {
                    // err.
                    // // TODO: handle this
                    // eprintln!("uhoh - {err}");
                    // None
                    Some(format!("This record contained invalid utf-8"))
                }
            }
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
        domain.push(part.to_owned());
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

    for record in res.answers {
        if record.rclass != RecordClass::IN {
            // TODO: handle this
            println!("womp womp womp");
            continue;
        }

        //hackclub.com.           3789    IN      HINFO   "RFC8482" ""
        //google.com.             300     IN      A       74.125.142.139
        if let Some(display) = format_data(record.rtype, record.data, record.domain_data) {
            println!(
                "\x1b[0;96m{:<23}\x1b[0;93m {:<8}\x1b[0m {:<7} {:<7} {}",
                record.name.to_string(),
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
