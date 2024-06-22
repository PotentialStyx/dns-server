use std::{
    collections::HashMap,
    net::{Ipv4Addr, Ipv6Addr},
};

use bytes::{Buf, Bytes};
use types::{
    parser::{BytesBuf, Parsable},
    Domain,
};

pub fn format_domain(domain: &Domain, idna: bool) -> String {
    let res = if idna {
        domain.idna_to_string()
    } else {
        domain.to_string()
    };

    format!("\x1b[0;95m{res}\x1b[0m")
}

pub fn format_ipv4(ip: Ipv4Addr) -> String {
    format!("\x1b[0;35m{ip}\x1b[0m")
}

pub fn format_ipv6(ip: Ipv6Addr) -> String {
    format!("\x1b[0;35m{ip}\x1b[0m")
}

pub fn format_svcb(mut data: Bytes) -> Option<String> {
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
                    attributes_rendered += &format_ipv4(Ipv4Addr::new(
                        data.get_u8(),
                        data.get_u8(),
                        data.get_u8(),
                        data.get_u8(),
                    ));
                    attributes_rendered += ",";
                }

                attributes_rendered =
                    attributes_rendered[1..attributes_rendered.len() - 1].to_string();
            }
            6 => {
                attributes_rendered += "ipv6hint=";

                while !data.is_empty() {
                    attributes_rendered += &format_ipv6(Ipv6Addr::new(
                        data.get_u16(),
                        data.get_u16(),
                        data.get_u16(),
                        data.get_u16(),
                        data.get_u16(),
                        data.get_u16(),
                        data.get_u16(),
                        data.get_u16(),
                    ));
                    attributes_rendered += ",";
                }

                attributes_rendered =
                    attributes_rendered[1..attributes_rendered.len() - 1].to_string();
            }
            _ => {}
        }
    }

    Some(format!(
        "{priority} {} {}",
        format_domain(&name, true),
        &attributes_rendered[1..]
    ))
}

pub fn format_character_string(data: &mut Bytes) -> Result<String, &'static str> {
    let length = data.get_u8() as usize;

    match std::str::from_utf8(&data.slice(0..length)) {
        Ok(ret) => {
            data.advance(length);
            Ok(ret.into())
        }
        Err(_err) => {
            // err.
            // // TODO: handle this
            // eprintln!("uhoh - {err}");
            // None
            Err("This record contained invalid utf-8")
        }
    }
}

pub fn format_hinfo(mut data: Bytes) -> String {
    let mut res = "\"\x1b[0;32m".to_string();

    match format_character_string(&mut data) {
        Ok(data) => {
            res += &data;
        }
        Err(err) => return err.to_string(),
    };

    res += "\x1b[0m\" \"\x1b[0;32m";

    match format_character_string(&mut data) {
        Ok(data) => {
            res += &data;
        }
        Err(err) => return err.to_string(),
    };

    res + "\x1b[0m\""
}

pub fn format_caa(mut data: Bytes) -> String {
    let flags = data.get_u8();

    let critical = flags & 128 == 128;

    let tag = match format_character_string(&mut data) {
        Ok(data) => data,
        Err(err) => return err.to_string(),
    };

    let value = std::str::from_utf8(&data).expect("TODO: deal with this");

    let critical_rendered = if critical {
        "\x1b[1;91mcritical\x1b[0m"
    } else {
        "\x1b[1;92mnormal\x1b[0m"
    };

    format!("{critical_rendered} {tag} \"{value}\"")
}

pub fn format_soa(mut data: Bytes, mut domains: Vec<Domain>, after_ptr: usize) -> String {
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

    data.advance(after_ptr);

    let email = &email[0..email.len() - 1];

    format!(
        "\"\x1b[0;95m{}\x1b[0m\" {email} {} {} {} {} {}",
        format_domain(&mname, true),
        data.get_u32(),
        data.get_u32(),
        data.get_u32(),
        data.get_u32(),
        data.get_u32(),
    )
}
