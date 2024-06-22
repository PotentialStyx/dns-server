use bytes::Bytes;

use crate::*;
use parser::*;

fn get_shared_test_case_data() -> Message {
    Message{
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
                ), domain_data: None, after_ptr: None
            }
        ],
        additional: vec![]
    }
}

#[test]
fn test_decode() {
    let bytes = std::fs::read("tests/test-pointers.bin").unwrap();
    let mut data = BytesBuf::from_bytes(Bytes::copy_from_slice(&bytes));
    let message = Message::parse(&mut data).unwrap();

    assert_eq!(message, get_shared_test_case_data())
}

#[test]
fn test_decode_nopointers() {
    let bytes = std::fs::read("tests/test-nopointers.bin").unwrap();
    let mut data = BytesBuf::from_bytes(Bytes::copy_from_slice(&bytes));
    let message = Message::parse(&mut data).unwrap();

    assert_eq!(message, get_shared_test_case_data())
}
