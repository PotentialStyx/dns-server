use bytes::Bytes;

use crate::*;
use parser::*;

#[test]
fn no_data() {
    // Buf has 1 0x00 so we don't catch the domain NEB error, but the resource record NEB error
    let mut record_buf: BytesBuf = BytesBuf::new(vec![0x00]);

    assert_eq!(
        ResourceRecord::parse(&mut record_buf),
        Err(ParserError::NotEnoughBytes {
            expected: 8,
            recieved: 0
        })
    );
}

#[test]
fn not_enough_data() {
    // Missing one byte of ttl
    let mut record_buf: BytesBuf = BytesBuf::new(vec![
        0x00, // domain: `.`
        0x00, 0x00, // rtype: 0
        0x00, 0x00, // rclass: 0
        0x00, 0x00, // ttl: 0
        0x00,
    ]);

    assert_eq!(
        ResourceRecord::parse(&mut record_buf),
        Err(ParserError::NotEnoughBytes {
            expected: 8,
            recieved: 7
        })
    );
}

#[test]
fn not_enough_record_data() {
    // Claims to have 13 bytes of data but has nothing
    let mut record_buf: BytesBuf = BytesBuf::new(vec![
        0x00, // domain: `.`
        0x00, 0x00, // rtype: 0
        0x00, 0x00, // rclass: 0
        0x00, 0x00, // ttl: 0
        0x00, 0x00, //
        0x00, 0x0D, // data len: 13
    ]);

    assert_eq!(
        ResourceRecord::parse(&mut record_buf),
        Err(ParserError::NotEnoughBytes {
            expected: 13,
            recieved: 0
        })
    );
}

#[test]
fn no_record_data() {
    // Blank record
    let mut record_buf: BytesBuf = BytesBuf::new(vec![
        0x00, // domain: `.`
        0x00, 0x00, // rtype: 0
        0x00, 0x00, // rclass: 0
        0x00, 0x00, // ttl: 0
        0x00, 0x00, //
        0x00, 0x00, // data len: 0
    ]);

    assert_eq!(
        ResourceRecord::parse(&mut record_buf),
        Ok(ResourceRecord {
            name: Domain(vec![]),
            rtype: RecordType::Unknown(0),
            rclass: RecordClass::Unknown(0),
            ttl: 0,
            data: Bytes::new(),
            domain_data: None,
            after_ptr: None
        })
    );
}

#[test]
fn correct_record_data() {
    // Blank record
    let mut record_buf: BytesBuf = BytesBuf::new(vec![
        0x00, // domain: `.`
        0x00, 0x00, // rtype: 0
        0x00, 0x00, // rclass: 0
        0x00, 0x00, // ttl: 0
        0x00, 0x00, //
        0x00, 0x04, // data len: 4
        0x13, 0x37, 0x13, 0x37,
    ]);

    assert_eq!(
        ResourceRecord::parse(&mut record_buf),
        Ok(ResourceRecord {
            name: Domain(vec![]),
            rtype: RecordType::Unknown(0),
            rclass: RecordClass::Unknown(0),
            ttl: 0,
            data: Bytes::from_static(&[0x13, 0x37, 0x13, 0x37]),
            domain_data: None,
            after_ptr: None
        })
    );
}

#[test]
fn correct_total_decoding() {
    let mut record_buf: BytesBuf = BytesBuf::new(vec![
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
    ]);

    assert_eq!(
        ResourceRecord::parse(&mut record_buf),
        Ok(ResourceRecord {
            name: Domain(vec!["www".into(), "hackclub".into(), "com".into()]),
            rtype: RecordType::A,
            rclass: RecordClass::IN,
            ttl: 0xDEADBEEF,
            data: Bytes::from_static(&[0xBA, 0xAA, 0xAA, 0xAD]),
            domain_data: None,
            after_ptr: None
        })
    );
}
