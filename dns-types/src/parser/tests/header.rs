use crate::*;
use parser::*;

#[test]
fn no_data() {
    let mut header_buf: BytesBuf = BytesBuf::new(vec![]);

    assert_eq!(
        Header::parse(&mut header_buf),
        Err(ParserError::NotEnoughBytes {
            expected: 12,
            recieved: 0
        })
    );
}

#[test]
fn correct_id() {
    let mut header_buf: BytesBuf = BytesBuf::new(vec![
        0x13, 0x37, // ID: 0x1337
        0x00, 0x00, // Everything is 0
        0x00, 0x00, // Questions: 0
        0x00, 0x00, // Answers: 0
        0x00, 0x00, // Authorities: 0
        0x00, 0x00, // Additional: 0
    ]);

    assert_eq!(
        Header::parse(&mut header_buf),
        Ok(Header {
            id: 0x1337,
            opcode: OpCode::Query,
            is_authoritative: false,
            is_response: false,
            is_truncated: false,
            should_recurse: false,
            _z: 0,
            recursion_available: false,
            rescode: ResCode::NoError,
            questions: 0,
            answer_records: 0,
            authority_records: 0,
            additional_records: 0
        })
    );
}

#[test]
fn correct_is_response() {
    let mut header_buf: BytesBuf = BytesBuf::new(vec![
        0x00, 0x00, // ID: 0
        0x80, 0x00, // Only is_response: true
        0x00, 0x00, // Questions: 0
        0x00, 0x00, // Answers: 0
        0x00, 0x00, // Authorities: 0
        0x00, 0x00, // Additional: 0
    ]);

    assert_eq!(
        Header::parse(&mut header_buf),
        Ok(Header {
            id: 0,
            opcode: OpCode::Query,
            is_authoritative: false,
            is_response: true,
            is_truncated: false,
            should_recurse: false,
            _z: 0,
            recursion_available: false,
            rescode: ResCode::NoError,
            questions: 0,
            answer_records: 0,
            authority_records: 0,
            additional_records: 0
        })
    );
}

#[test]
fn correct_opcode() {
    let mut header_buf: BytesBuf = BytesBuf::new(vec![
        0x00, 0x00, // ID: 0
        0x70, 0x00, // Only opcode: 0xE
        0x00, 0x00, // Questions: 0
        0x00, 0x00, // Answers: 0
        0x00, 0x00, // Authorities: 0
        0x00, 0x00, // Additional: 0
    ]);

    assert_eq!(
        Header::parse(&mut header_buf),
        Ok(Header {
            id: 0,
            opcode: OpCode::Reserved(0xE),
            is_authoritative: false,
            is_response: false,
            is_truncated: false,
            should_recurse: false,
            _z: 0,
            recursion_available: false,
            rescode: ResCode::NoError,
            questions: 0,
            answer_records: 0,
            authority_records: 0,
            additional_records: 0
        })
    );
}

#[test]
fn correct_is_authoritative() {
    let mut header_buf: BytesBuf = BytesBuf::new(vec![
        0x00, 0x00, // ID: 0
        0x04, 0x00, // Only is_authoritative: true
        0x00, 0x00, // Questions: 0
        0x00, 0x00, // Answers: 0
        0x00, 0x00, // Authorities: 0
        0x00, 0x00, // Additional: 0
    ]);

    assert_eq!(
        Header::parse(&mut header_buf),
        Ok(Header {
            id: 0,
            opcode: OpCode::Query,
            is_authoritative: true,
            is_response: false,
            is_truncated: false,
            should_recurse: false,
            _z: 0,
            recursion_available: false,
            rescode: ResCode::NoError,
            questions: 0,
            answer_records: 0,
            authority_records: 0,
            additional_records: 0
        })
    );
}

#[test]
fn correct_is_truncated() {
    let mut header_buf: BytesBuf = BytesBuf::new(vec![
        0x00, 0x00, // ID: 0
        0x02, 0x00, // Only is_truncated: true
        0x00, 0x00, // Questions: 0
        0x00, 0x00, // Answers: 0
        0x00, 0x00, // Authorities: 0
        0x00, 0x00, // Additional: 0
    ]);

    assert_eq!(
        Header::parse(&mut header_buf),
        Ok(Header {
            id: 0,
            opcode: OpCode::Query,
            is_authoritative: false,
            is_response: false,
            is_truncated: true,
            should_recurse: false,
            _z: 0,
            recursion_available: false,
            rescode: ResCode::NoError,
            questions: 0,
            answer_records: 0,
            authority_records: 0,
            additional_records: 0
        })
    );
}

#[test]
fn correct_should_recurse() {
    let mut header_buf: BytesBuf = BytesBuf::new(vec![
        0x00, 0x00, // ID: 0
        0x01, 0x00, // Only should_recurse: true
        0x00, 0x00, // Questions: 0
        0x00, 0x00, // Answers: 0
        0x00, 0x00, // Authorities: 0
        0x00, 0x00, // Additional: 0
    ]);

    assert_eq!(
        Header::parse(&mut header_buf),
        Ok(Header {
            id: 0,
            opcode: OpCode::Query,
            is_authoritative: false,
            is_response: false,
            is_truncated: false,
            should_recurse: true,
            _z: 0,
            recursion_available: false,
            rescode: ResCode::NoError,
            questions: 0,
            answer_records: 0,
            authority_records: 0,
            additional_records: 0
        })
    );
}

#[test]
fn correct_recursion_available() {
    let mut header_buf: BytesBuf = BytesBuf::new(vec![
        0x00, 0x00, // ID: 0
        0x00, 0x80, // Only recursion_available: true
        0x00, 0x00, // Questions: 0
        0x00, 0x00, // Answers: 0
        0x00, 0x00, // Authorities: 0
        0x00, 0x00, // Additional: 0
    ]);

    assert_eq!(
        Header::parse(&mut header_buf),
        Ok(Header {
            id: 0,
            opcode: OpCode::Query,
            is_authoritative: false,
            is_response: false,
            is_truncated: false,
            should_recurse: false,
            _z: 0,
            recursion_available: true,
            rescode: ResCode::NoError,
            questions: 0,
            answer_records: 0,
            authority_records: 0,
            additional_records: 0
        })
    );
}

#[test]
fn correct_z() {
    let mut header_buf: BytesBuf = BytesBuf::new(vec![
        0x00, 0x00, // ID: 0
        0x00, 0x70, // Only _z: 0x7
        0x00, 0x00, // Questions: 0
        0x00, 0x00, // Answers: 0
        0x00, 0x00, // Authorities: 0
        0x00, 0x00, // Additional: 0
    ]);

    assert_eq!(
        Header::parse(&mut header_buf),
        Ok(Header {
            id: 0,
            opcode: OpCode::Query,
            is_authoritative: false,
            is_response: false,
            is_truncated: false,
            should_recurse: false,
            _z: 0x7,
            recursion_available: false,
            rescode: ResCode::NoError,
            questions: 0,
            answer_records: 0,
            authority_records: 0,
            additional_records: 0
        })
    );
}

#[test]
fn correct_rescode() {
    let mut header_buf: BytesBuf = BytesBuf::new(vec![
        0x00, 0x00, // ID: 0
        0x00, 0x0E, // Only rescode: 0xE
        0x00, 0x00, // Questions: 0
        0x00, 0x00, // Answers: 0
        0x00, 0x00, // Authorities: 0
        0x00, 0x00, // Additional: 0
    ]);

    assert_eq!(
        Header::parse(&mut header_buf),
        Ok(Header {
            id: 0,
            opcode: OpCode::Query,
            is_authoritative: false,
            is_response: false,
            is_truncated: false,
            should_recurse: false,
            _z: 0,
            recursion_available: false,
            rescode: ResCode::Reserved(0xE),
            questions: 0,
            answer_records: 0,
            authority_records: 0,
            additional_records: 0
        })
    );
}

#[test]
fn correct_questions() {
    let mut header_buf: BytesBuf = BytesBuf::new(vec![
        0x00, 0x00, // ID: 0
        0x00, 0x00, // Everything is 0
        0x13, 0x37, // Questions: 0x1337
        0x00, 0x00, // Answers: 0
        0x00, 0x00, // Authorities: 0
        0x00, 0x00, // Additional: 0
    ]);

    assert_eq!(
        Header::parse(&mut header_buf),
        Ok(Header {
            id: 0,
            opcode: OpCode::Query,
            is_authoritative: false,
            is_response: false,
            is_truncated: false,
            should_recurse: false,
            _z: 0,
            recursion_available: false,
            rescode: ResCode::NoError,
            questions: 0x1337,
            answer_records: 0,
            authority_records: 0,
            additional_records: 0
        })
    );
}

#[test]
fn correct_answers() {
    let mut header_buf: BytesBuf = BytesBuf::new(vec![
        0x00, 0x00, // ID: 0
        0x00, 0x00, // Everything is 0
        0x00, 0x00, // Questions: 0
        0x13, 0x37, // Answers: 0x1337
        0x00, 0x00, // Authorities: 0
        0x00, 0x00, // Additional: 0
    ]);

    assert_eq!(
        Header::parse(&mut header_buf),
        Ok(Header {
            id: 0,
            opcode: OpCode::Query,
            is_authoritative: false,
            is_response: false,
            is_truncated: false,
            should_recurse: false,
            _z: 0,
            recursion_available: false,
            rescode: ResCode::NoError,
            questions: 0,
            answer_records: 0x1337,
            authority_records: 0,
            additional_records: 0
        })
    );
}

#[test]
fn correct_authorities() {
    let mut header_buf: BytesBuf = BytesBuf::new(vec![
        0x00, 0x00, // ID: 0
        0x00, 0x00, // Everything is 0
        0x00, 0x00, // Questions: 0
        0x00, 0x00, // Answers: 0
        0x13, 0x37, // Authorities: 0x1337
        0x00, 0x00, // Additional: 0
    ]);

    assert_eq!(
        Header::parse(&mut header_buf),
        Ok(Header {
            id: 0,
            opcode: OpCode::Query,
            is_authoritative: false,
            is_response: false,
            is_truncated: false,
            should_recurse: false,
            _z: 0,
            recursion_available: false,
            rescode: ResCode::NoError,
            questions: 0,
            answer_records: 0,
            authority_records: 0x1337,
            additional_records: 0
        })
    );
}

#[test]
fn correct_additional() {
    let mut header_buf: BytesBuf = BytesBuf::new(vec![
        0x00, 0x00, // ID: 0
        0x00, 0x00, // Everything is 0
        0x00, 0x00, // Questions: 0
        0x00, 0x00, // Answers: 0
        0x00, 0x00, // Authorities: 0
        0x13, 0x37, // Additional: 0x1337
    ]);

    assert_eq!(
        Header::parse(&mut header_buf),
        Ok(Header {
            id: 0,
            opcode: OpCode::Query,
            is_authoritative: false,
            is_response: false,
            is_truncated: false,
            should_recurse: false,
            _z: 0,
            recursion_available: false,
            rescode: ResCode::NoError,
            questions: 0,
            answer_records: 0,
            authority_records: 0,
            additional_records: 0x1337
        })
    );
}
