use crate::*;
use parser::*;

#[test]
fn not_enough_data() {
    // Buf has 1 0x00 so we don't catch the domain NEB error, but the question NEB error
    let mut question_buf: BytesBuf = BytesBuf::new(vec![0x00]);

    assert_eq!(
        Question::parse(&mut question_buf),
        Err(ParserError::NotEnoughBytes {
            expected: 4,
            recieved: 0
        })
    );
}

#[test]
fn correct_total_decoding() {
    // 3 www, 8 hackclub, 3 com, 0, qtype: 255 (ALL), qclass: 1 (IN)
    let mut question_buf: BytesBuf = BytesBuf::new(vec![
        3, 119, 119, 119, 8, 104, 97, 99, 107, 99, 108, 117, 98, 3, 99, 111, 109, 0, 0x00, 0xFF,
        0x00, 0x01,
    ]);

    assert_eq!(
        Question::parse(&mut question_buf),
        Ok(Question {
            name: Domain(vec!["www".into(), "hackclub".into(), "com".into()]),
            qtype: RecordType::ALL,
            qclass: RecordClass::IN,
        })
    );
}
