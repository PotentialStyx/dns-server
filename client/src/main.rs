use clap::Parser;

macro_rules! from_str_enum {
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
    };
}

from_str_enum! {
    enum ArgRecordType[Unknown] {
        A,
        NS,
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

fn main() {
    let _cli = dbg!(Cli::parse());
}
