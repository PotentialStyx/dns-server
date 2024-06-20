use std::fmt::Display;

use bytes::Bytes;

use super::{OpCode, RecordClass, RecordType, ResCode};

/// DNS Domain
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Domain(pub Vec<String>);

/// TODO: add tests for this maybe(?)
impl Display for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for part in &self.0 {
            f.write_str(part)?;
            f.write_str(".")?;
        }

        Ok(())
    }
}

// TODO: Maybe give this a display implementation
/// DNS Record Header
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    /// Message ID
    pub id: u16,
    /// If its a response message
    pub is_response: bool,
    /// Message OpCode
    pub opcode: OpCode,
    /// Is the message authoritative
    pub is_authoritative: bool,
    /// Was the message truncated due to UDP (and should be resent over TCP)
    pub is_truncated: bool,
    /// Should the server recursively look up the domain
    pub should_recurse: bool,
    /// Does the server support recursion
    pub recursion_available: bool,
    /// Should be zeros, but in the past my implementations haven't worked unless I tracked this
    pub(crate) _z: u8,
    /// Result code from request
    pub rescode: ResCode,

    /// Number of questions
    pub questions: u16,
    /// Number of answer records
    pub answer_records: u16,
    /// Number of authority records
    pub authority_records: u16,
    /// Number of additional records
    pub additional_records: u16,
}

/// A singular question
#[derive(Debug, PartialEq, Eq)]
pub struct Question {
    /// Domain to lookup
    pub name: Domain,
    /// What record types to check
    pub qtype: RecordType,
    /// What class of records to check
    pub qclass: RecordClass,
}

/// One singular Resource Record
#[derive(Debug, PartialEq, Eq)]
pub struct ResourceRecord {
    /// Domain this record refers to
    pub name: Domain,
    /// The record type
    pub rtype: RecordType,
    /// The record class
    pub rclass: RecordClass,
    /// Suggested record TTL
    pub ttl: u32,
    /// Actual record data
    pub data: Bytes,
    pub domain_data: Option<Domain>,
}

/// A full DNS Message
#[derive(Debug, PartialEq, Eq)]
pub struct Message {
    pub header: Header,
    pub questions: Vec<Question>,
    pub answers: Vec<ResourceRecord>,
    pub authorities: Vec<ResourceRecord>,
    pub additional: Vec<ResourceRecord>,
}
