use crate::*;
use parser::*;

#[test]
fn display_single_part() {
    assert_eq!(format!("{}", Domain(vec!["com".to_string()])), "com.")
}

#[test]
fn display_multi_part() {
    assert_eq!(
        format!(
            "{}",
            Domain(vec![
                "www".to_string(),
                "google".to_string(),
                "com".to_string()
            ])
        ),
        "www.google.com."
    )
}

#[test]
fn no_data() {
    // An empty buf should error
    let mut domain_buf: BytesBuf = BytesBuf::new(vec![]);

    assert_eq!(
        Domain::parse(&mut domain_buf),
        Err(ParserError::NotEnoughBytes {
            expected: 1,
            recieved: 0
        })
    );
}

#[test]
fn not_enough_data() {
    // A domain name claiming to have a longer section than it really has should error
    // This buf claims to have 6 chars but only have 5, which spell out "hello"
    let mut domain_buf: BytesBuf = BytesBuf::new(vec![6, 104, 101, 108, 108, 111]);

    assert_eq!(
        Domain::parse(&mut domain_buf),
        Err(ParserError::NotEnoughBytes {
            expected: 6,
            recieved: 5
        })
    );
}

#[test]
fn non_ascii_error() {
    // 😀 is an invalid domain because it isn't ascii
    let mut domain_buf: BytesBuf = BytesBuf::new(vec![4, 240, 159, 152, 128]);

    assert_eq!(
        Domain::parse(&mut domain_buf),
        Err(ParserError::InvalidAscii(AsciiError::NotAscii))
    );
}

#[test]
fn non_utf8_error() {
    // Invalid utf-8 sequence from https://stackoverflow.com/a/17199164/15264500
    let mut domain_buf: BytesBuf = BytesBuf::new(vec![2, 0xc3, 0x28]);

    assert!(matches!(
        Domain::parse(&mut domain_buf),
        Err(ParserError::InvalidAscii(..))
    ));
}

#[test]
fn correct_hackclub_decoding() {
    // 3 www, 8 hackclub, 3 com, 0
    let mut domain_buf: BytesBuf = BytesBuf::new(vec![
        3, 119, 119, 119, 8, 104, 97, 99, 107, 99, 108, 117, 98, 3, 99, 111, 109, 0,
    ]);

    assert_eq!(
        Domain::parse(&mut domain_buf),
        Ok(Domain(vec!["www".into(), "hackclub".into(), "com".into()]))
    );
}

#[test]
fn invalid_hackclub_decoding() {
    // 3 www, 10! hackclub, 3 com, 0
    let mut domain_buf: BytesBuf = BytesBuf::new(vec![
        3, 119, 119, 119, 10, 104, 97, 99, 107, 99, 108, 117, 98, 3, 99, 111, 109, 0,
    ]);

    assert_eq!(
        Domain::parse(&mut domain_buf),
        Err(ParserError::NotEnoughBytes {
            expected: 111,
            recieved: 2
        })
    );
}

#[test]
fn correct_google_decoding() {
    // 3 www, 6 google, 3 com, 0
    let mut domain_buf: BytesBuf = BytesBuf::new(vec![
        3, 119, 119, 119, 6, 103, 111, 111, 103, 108, 101, 3, 99, 111, 109, 0,
    ]);

    assert_eq!(
        Domain::parse(&mut domain_buf),
        Ok(Domain(vec!["www".into(), "google".into(), "com".into()]))
    );
}

#[test]
fn invalid_google_decoding() {
    // 3 www, 7! google, 3 com, 0
    let mut domain_buf: BytesBuf = BytesBuf::new(vec![
        3, 119, 119, 119, 7, 103, 111, 111, 103, 108, 101, 3, 99, 111, 109, 0,
    ]);

    assert_eq!(
        Domain::parse(&mut domain_buf),
        Err(ParserError::NotEnoughBytes {
            expected: 99,
            recieved: 3
        })
    );
}
