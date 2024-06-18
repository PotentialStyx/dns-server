use bytes::Bytes;

use super::*;

#[test]
fn convert_enum_macro_recordclass() {
    // RecordClass::IN
    let in_rec: u16 = RecordClass::IN.into();
    assert_eq!(in_rec, 1);

    let in_rec: RecordClass = 1.into();
    assert_eq!(in_rec, RecordClass::IN);

    // RecordClass::CS
    let cs_rec: u16 = RecordClass::CS.into();
    assert_eq!(cs_rec, 2);

    let cs_rec: RecordClass = 2.into();
    assert_eq!(cs_rec, RecordClass::CS);

    // RecordClass::CH
    let ch_rec: u16 = RecordClass::CH.into();
    assert_eq!(ch_rec, 3);

    let ch_rec: RecordClass = 3.into();
    assert_eq!(ch_rec, RecordClass::CH);

    // RecordClass::HS
    let hs_rec: u16 = RecordClass::HS.into();
    assert_eq!(hs_rec, 4);

    let hs_rec: RecordClass = 4.into();
    assert_eq!(hs_rec, RecordClass::HS);

    // RecordClass::ANY
    let any_rec: u16 = RecordClass::ANY.into();
    assert_eq!(any_rec, 255);

    let any_rec: RecordClass = 255.into();
    assert_eq!(any_rec, RecordClass::ANY);

    // RecordClass::Unknown(155)
    let rnd_rec: u16 = RecordClass::Unknown(155).into();
    assert_eq!(rnd_rec, 155);

    let rnd_rec: RecordClass = 155.into();
    assert_eq!(rnd_rec, RecordClass::Unknown(155));
}

#[test]
fn display_single_part_domain() {
    assert_eq!(format!("{}", Domain(vec!["com".to_string()])), "com.")
}

#[test]
fn display_multi_part_domain() {
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
fn domain_no_data() {
    // An empty buf should error
    let domain_buf: Bytes = vec![].into();

    assert_eq!(
        Domain::parse(&domain_buf),
        Err(ParserError::NotEnoughBytes {
            expected: 1,
            recieved: 0
        })
    );
}

#[test]
fn domain_not_enough_data() {
    // A domain name claiming to have a longer section than it really has should error
    // This buf claims to have 6 chars but only have 5, which spell out "hello"
    let domain_buf: Bytes = vec![6, 104, 101, 108, 108, 111].into();

    assert_eq!(
        Domain::parse(&domain_buf),
        Err(ParserError::NotEnoughBytes {
            expected: 6,
            recieved: 5
        })
    );
}

#[test]
fn domain_non_ascii_error() {
    // 😀 is an invalid domain because it isn't ascii
    let domain_buf: Bytes = vec![4, 240, 159, 152, 128].into();

    assert_eq!(
        Domain::parse(&domain_buf),
        Err(ParserError::InvalidAscii(AsciiError::NotAscii))
    );
}

#[test]
fn domain_non_utf8_error() {
    // Invalid utf-8 sequence from https://stackoverflow.com/a/17199164/15264500
    let domain_buf: Bytes = vec![2, 0xc3, 0x28].into();

    assert!(matches!(
        Domain::parse(&domain_buf),
        Err(ParserError::InvalidAscii(..))
    ));
}

#[test]
fn domain_correct_decoding() {
    // 3 www, 8 hackclub, 3 com, 0
    let domain_buf: Bytes = vec![
        3, 119, 119, 119, 8, 104, 97, 99, 107, 99, 108, 117, 98, 3, 99, 111, 109, 0,
    ]
    .into();

    assert_eq!(
        Domain::parse(&domain_buf),
        Ok(Domain(vec!["www".into(), "hackclub".into(), "com".into()]))
    );
}
