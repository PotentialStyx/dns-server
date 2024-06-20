use bytes::BytesMut;

use crate::*;
use serializer::*;
// TODO: write more tests(?)

#[test]
fn correct_total_encoding() {
    let mut buf = BytesMut::new();
    assert_eq!(
        Question {
            name: Domain(vec!["www".into(), "hackclub".into(), "com".into()]),
            qtype: RecordType::ALL,
            qclass: RecordClass::IN,
        }
        .serialize(&mut buf),
        Ok(())
    );

    // 3 www, 8 hackclub, 3 com, 0, qtype: 255 (ALL), qclass: 1 (IN)
    let result_buf: &[u8] = &[
        3, 119, 119, 119, 8, 104, 97, 99, 107, 99, 108, 117, 98, 3, 99, 111, 109, 0, 0x00, 0xFF,
        0x00, 0x01,
    ];

    assert_eq!(buf, result_buf);
}
