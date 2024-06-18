use core::fmt::{Display, Error, Formatter};

macro_rules! useful_enum {
    (
        $vis:vis enum $name:ident($unknown:ident, $type:ty) {
            $($field:ident = $value:expr,)*
        }
    ) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        #[repr($type)]
        #[allow(clippy::upper_case_acronyms)]
        $vis enum $name {
            $($field = $value,)*
            $unknown($type)
        }

        impl From<$type> for $name {
            fn from(s: $type) -> Self {
                match s {
                    $($value => Self::$field,)*
                    _ => Self::$unknown(s),
                }
            }
        }

        impl From<$name> for $type {
            fn from(s: $name) -> Self {
                match s {
                    $($name::$field => $value,)*
                    $name::$unknown(s) => s,
                }
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
                match self {
                    $($name::$field => f.write_str(concat!(stringify!($name), "::", stringify!($field),"(",stringify!($value),")")),)*
                    $name::$unknown(s) => f.write_fmt(format_args!("{}::{}({s})", stringify!($name), stringify!($unknown))),
                }
            }
        }
    };
}

useful_enum! {
    pub enum OpCode(Reserved, u16) {
        Query = 0,
        IQuery = 1,
        Status = 2,
    }
}

useful_enum! {
    pub enum ResCode(Reserved, u16) {
        NoError = 0,
        FormatError = 1,
        ServerFailure = 2,
        NameError = 3,
        NotImplemented = 4,
        Refused = 5,
    }
}

// From: https://datatracker.ietf.org/doc/html/rfc1035#autoid-14
useful_enum! {
    pub enum RecordType(Unknown, u16) {
        // TYPE
        A = 1,      // a host address
        NS = 2,     // an authoritative name server
        MD = 3,     // a mail destination (Obsolete - use MX)
        MF = 4,     // a mail forwarder (Obsolete - use MX)
        CNAME = 5,  // the canonical name for an alias
        SOA = 6,    // marks the start of a zone of authority
        MB = 7,     // a mailbox domain name (EXPERIMENTAL)
        MG = 8,     // a mail group member (EXPERIMENTAL)
        MR = 9,     // a mail rename domain name (EXPERIMENTAL)
        NULL = 10,  // a null RR (EXPERIMENTAL)
        WKS = 11,   // a well known service description
        PTR = 12,   // a domain name pointer
        HINFO = 13, // host information
        MINFO = 14, // mailbox or mail list information
        MX = 15,    // mail exchange
        TXT = 16,   // text strings

        // QTYPE
        AXFR = 252,  // A request for a transfer of an entire zone
        MAILB = 253, // A request for mailbox-related records (MB, MG or MR)
        MAILA = 254, // A request for mail agent RRs (Obsolete - see MX)
        ALL = 255,   // A request for all records
    }
}

useful_enum! {
    pub enum RecordClass(Unknown, u16) {
        IN = 1, // the Internet
        CS = 2, // the CSNET class (Obsolete - used only for examples in some obsolete RFCs)
        CH = 3, // the CHAOS class
        HS = 4, // Hesiod [Dyer 87]

        // QCLASS
        ANY = 255,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enum_macro_recordclass() {
        // RecordClass::IN
        let in_rec: u16 = RecordClass::IN.into();
        assert_eq!(in_rec, 1);

        let in_rec: RecordClass = 1.into();
        assert_eq!(in_rec, RecordClass::IN);

        // RecordClass::CS
        let cs_rec: u16 = RecordClass::CS.into();
        assert_eq!(cs_rec, 2);

        let cs_rec: RecordClass = 2.into();
        assert_eq!(cs_rec, RecordClass::CS);

        // RecordClass::CH
        let ch_rec: u16 = RecordClass::CH.into();
        assert_eq!(ch_rec, 3);

        let ch_rec: RecordClass = 3.into();
        assert_eq!(ch_rec, RecordClass::CH);

        // RecordClass::HS
        let hs_rec: u16 = RecordClass::HS.into();
        assert_eq!(hs_rec, 4);

        let hs_rec: RecordClass = 4.into();
        assert_eq!(hs_rec, RecordClass::HS);

        // RecordClass::ANY
        let any_rec: u16 = RecordClass::ANY.into();
        assert_eq!(any_rec, 255);

        let any_rec: RecordClass = 255.into();
        assert_eq!(any_rec, RecordClass::ANY);

        // RecordClass::Unknown(155)
        let rnd_rec: u16 = RecordClass::Unknown(155).into();
        assert_eq!(rnd_rec, 155);

        let rnd_rec: RecordClass = 155.into();
        assert_eq!(rnd_rec, RecordClass::Unknown(155));
    }
}
