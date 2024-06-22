use std::fs;

use crate::*;
use bytes::{Bytes, BytesMut};
use serializer::*;

#[test]
fn test_encode() {
    let mut buf = BytesMut::new();
    assert_eq!(Message{
        header: Header {
            id: 0,
            is_response: false,
            opcode: OpCode::Reserved(5),
            is_authoritative: false,
            is_truncated: false,
            should_recurse: false,
            recursion_available: false,
            _z: 0,
            rescode: ResCode::NoError,
            questions: 1,
            answer_records: 0,
            authority_records: 1,
            additional_records: 0
        },
        questions: vec![
            Question {
                name: Domain(vec!["se".into()]),
                qtype: RecordType::SOA,
                qclass: RecordClass::IN
            }
        ],
        answers: vec![],
        authorities: vec![
            ResourceRecord {
                name: Domain (vec!["se".into()]),
                rtype: RecordType::TXT,
                rclass: RecordClass::IN,
                ttl: 1337,
                data: Bytes::copy_from_slice(
                    b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                ),
                domain_data: None, after_ptr: None
            }
        ],
        additional: vec![]
    }.serialize(&mut buf), Ok(()));

    assert_eq!(buf, fs::read("tests/test-nopointers.bin").unwrap());
}

#[test]
fn too_many_questions_error() {
    let mut msg = Box::new(Message {
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
            authority_records: 1,
            additional_records: 0,
        },
        questions: vec![],
        answers: vec![],
        authorities: vec![],
        additional: vec![],
    });

    for _ in 0..(u16::MAX as usize + 1) {
        msg.questions.push(Question {
            name: Domain(vec![]),
            qtype: RecordType::Unknown(0),
            qclass: RecordClass::Unknown(0),
        });
    }

    let mut buf = BytesMut::new();
    assert_eq!(
        msg.serialize(&mut buf),
        Err(SerializerError::TooManyRecords {
            expected_max: 65535,
            recieved: 65536
        })
    );
}

#[test]
fn too_many_recordss_error() {
    let mut msg = Box::new(Message {
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
            authority_records: 1,
            additional_records: 0,
        },
        questions: vec![],
        answers: vec![],
        authorities: vec![],
        additional: vec![],
    });

    let data = Bytes::new();

    for _ in 0..(u16::MAX as usize + 1) {
        msg.additional.push(ResourceRecord {
            name: Domain(vec![]),
            rtype: RecordType::Unknown(0),
            rclass: RecordClass::Unknown(0),
            ttl: 0,
            data: data.clone(),
            domain_data: None,
            after_ptr: None,
        });
    }

    let mut buf = BytesMut::new();
    assert_eq!(
        msg.serialize(&mut buf),
        Err(SerializerError::TooManyRecords {
            expected_max: 65535,
            recieved: 65536
        })
    );
}
