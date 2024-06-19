use bytes::BytesMut;

use super::super::*;

#[test]
fn non_ascii_error() {
    let domain = Domain(vec!["😀".into()]);
    let mut buf = BytesMut::new();

    assert_eq!(
        domain.serialize(&mut buf),
        Err(SerializerError::InvalidAscii("😀".into()))
    );
}

#[test]
fn invalid_length_error() {
    // Domain section is 1 longer than the max of 63
    let domain = Domain(vec![
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
    ]);
    let mut buf = BytesMut::new();

    assert_eq!(
        domain.serialize(&mut buf),
        Err(SerializerError::TooManyBytes {
            expected_max: 63,
            recieved: 64
        })
    );
}

#[test]
fn correct_long_domain_encoding() {
    let domain = Domain(vec![
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
    ]);
    let mut buf = BytesMut::new();

    assert_eq!(domain.serialize(&mut buf), Ok(()));

    // 63 a * 63
    let result_buf: &[u8] = &[
        0x3F, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61,
        0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61,
        0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61,
        0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61,
        0x61, 0x61, 0x61, 0x61, 0x00,
    ];

    assert_eq!(buf, result_buf);
}

#[test]
fn correct_hackclub_encoding() {
    let domain = Domain(vec!["www".into(), "hackclub".into(), "com".into()]);
    let mut buf = BytesMut::new();

    assert_eq!(domain.serialize(&mut buf), Ok(()));

    // 3 www, 8 hackclub, 3 com, 0
    let result_buf: &[u8] = &[
        3, 119, 119, 119, 8, 104, 97, 99, 107, 99, 108, 117, 98, 3, 99, 111, 109, 0,
    ];

    assert_eq!(buf, result_buf);
}

#[test]
fn correct_google_encoding() {
    let domain = Domain(vec!["www".into(), "google".into(), "com".into()]);
    let mut buf = BytesMut::new();

    assert_eq!(domain.serialize(&mut buf), Ok(()));

    // 3 www, 6 google, 3 com, 0
    let result_buf: &[u8] = &[
        3, 119, 119, 119, 6, 103, 111, 111, 103, 108, 101, 3, 99, 111, 109, 0,
    ];

    assert_eq!(buf, result_buf);
}
