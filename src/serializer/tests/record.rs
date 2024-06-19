use bytes::{Bytes, BytesMut};

use super::super::*;

#[test]
fn correct_record_data() {
    let mut buf = BytesMut::new();
    assert_eq!(
        ResourceRecord {
            name: Domain(vec![]),
            rtype: RecordType::Unknown(0),
            rclass: RecordClass::Unknown(0),
            ttl: 0,
            data: Bytes::from_static(&[0x13, 0x37, 0x13, 0x37])
        }
        .serialize(&mut buf),
        Ok(())
    );

    // Blank record
    let result_buf: &[u8] = &[
        0x00, // domain: `.`
        0x00, 0x00, // rtype: 0
        0x00, 0x00, // rclass: 0
        0x00, 0x00, // ttl: 0
        0x00, 0x00, //
        0x00, 0x04, // data len: 4
        0x13, 0x37, 0x13, 0x37,
    ];

    assert_eq!(buf, result_buf);
}

#[test]
fn correct_total_encoding() {
    let mut buf = BytesMut::new();
    assert_eq!(
        ResourceRecord {
            name: Domain(vec!["www".into(), "hackclub".into(), "com".into()]),
            rtype: RecordType::A,
            rclass: RecordClass::IN,
            ttl: 0xDEADBEEF,
            data: Bytes::from_static(&[0xBA, 0xAA, 0xAA, 0xAD])
        }
        .serialize(&mut buf),
        Ok(())
    );

    let result_buf: &[u8] = &[
        0x03, // length: 3
        119, 119, 119,  // www
        0x08, // length 8
        104, 97, 99, 107, 99, 108, 117, 98,   // hackclub
        0x03, // length: 3
        99, 111, 109,  // com
        0x00, // length 0 - end of domain
        0x00, 0x01, // rtype: 1 (A)
        0x00, 0x01, // rclass: 1 (IN)
        0xDE, 0xAD, // ttl: 0xDEADBEEF
        0xBE, 0xEF, //
        0x00, 0x04, // data len: 4
        0xBA, 0xAA, 0xAA, 0xAD, // data: 0xBAAAAAAD
    ];

    assert_eq!(buf, result_buf);
}

#[test]
fn buf_too_long_error() {
    let mut buf = BytesMut::new();
    assert_eq!(
        ResourceRecord {
            name: Domain(vec!["www".into(), "hackclub".into(), "com".into()]),
            rtype: RecordType::A,
            rclass: RecordClass::IN,
            ttl: 0xDEADBEEF,
            data: Bytes::from_static(&[0x00; (u16::MAX as usize) + 1])
        }
        .serialize(&mut buf),
        Err(SerializerError::TooManyBytes {
            expected_max: u16::MAX as usize,
            recieved: (u16::MAX as usize) + 1
        })
    );
}
