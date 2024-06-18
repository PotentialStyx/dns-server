use super::*;

mod enums {
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
}

mod domain {
    use super::*;

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
        let domain_buf: BytesBuf = BytesBuf::new(vec![]);

        assert_eq!(
            Domain::parse(&domain_buf),
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
        let domain_buf: BytesBuf = BytesBuf::new(vec![6, 104, 101, 108, 108, 111]);

        assert_eq!(
            Domain::parse(&domain_buf),
            Err(ParserError::NotEnoughBytes {
                expected: 6,
                recieved: 5
            })
        );
    }

    #[test]
    fn non_ascii_error() {
        // 😀 is an invalid domain because it isn't ascii
        let domain_buf: BytesBuf = BytesBuf::new(vec![4, 240, 159, 152, 128]);

        assert_eq!(
            Domain::parse(&domain_buf),
            Err(ParserError::InvalidAscii(AsciiError::NotAscii))
        );
    }

    #[test]
    fn non_utf8_error() {
        // Invalid utf-8 sequence from https://stackoverflow.com/a/17199164/15264500
        let domain_buf: BytesBuf = BytesBuf::new(vec![2, 0xc3, 0x28]);

        assert!(matches!(
            Domain::parse(&domain_buf),
            Err(ParserError::InvalidAscii(..))
        ));
    }

    #[test]
    fn correct_hackclub_decoding() {
        // 3 www, 8 hackclub, 3 com, 0
        let domain_buf: BytesBuf = BytesBuf::new(vec![
            3, 119, 119, 119, 8, 104, 97, 99, 107, 99, 108, 117, 98, 3, 99, 111, 109, 0,
        ]);

        assert_eq!(
            Domain::parse(&domain_buf),
            Ok(Domain(vec!["www".into(), "hackclub".into(), "com".into()]))
        );
    }

    #[test]
    fn invalid_hackclub_decoding() {
        // 3 www, 10! hackclub, 3 com, 0
        let domain_buf: BytesBuf = BytesBuf::new(vec![
            3, 119, 119, 119, 10, 104, 97, 99, 107, 99, 108, 117, 98, 3, 99, 111, 109, 0,
        ]);

        assert_eq!(
            Domain::parse(&domain_buf),
            Err(ParserError::NotEnoughBytes {
                expected: 111,
                recieved: 2
            })
        );
    }

    #[test]
    fn correct_google_decoding() {
        // 3 www, 6 google, 3 com, 0
        let domain_buf: BytesBuf = BytesBuf::new(vec![
            3, 119, 119, 119, 6, 103, 111, 111, 103, 108, 101, 3, 99, 111, 109, 0,
        ]);

        assert_eq!(
            Domain::parse(&domain_buf),
            Ok(Domain(vec!["www".into(), "google".into(), "com".into()]))
        );
    }

    #[test]
    fn invalid_google_decoding() {
        // 3 www, 7! google, 3 com, 0
        let domain_buf: BytesBuf = BytesBuf::new(vec![
            3, 119, 119, 119, 7, 103, 111, 111, 103, 108, 101, 3, 99, 111, 109, 0,
        ]);

        assert_eq!(
            Domain::parse(&domain_buf),
            Err(ParserError::NotEnoughBytes {
                expected: 99,
                recieved: 3
            })
        );
    }
}

mod header {
    use super::*;

    #[test]
    fn not_enough_data() {
        let header_buf: BytesBuf = BytesBuf::new(vec![]);

        assert_eq!(
            Header::parse(&header_buf),
            Err(ParserError::NotEnoughBytes {
                expected: 12,
                recieved: 0
            })
        );
    }

    #[test]
    fn correct_id() {
        let header_buf: BytesBuf = BytesBuf::new(vec![
            0x13, 0x37, // ID: 0x1337
            0x00, 0x00, // Everything is 0
            0x00, 0x00, // Questions: 0
            0x00, 0x00, // Answers: 0
            0x00, 0x00, // Authorities: 0
            0x00, 0x00, // Additional: 0
        ]);

        assert_eq!(
            Header::parse(&header_buf),
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
        let header_buf: BytesBuf = BytesBuf::new(vec![
            0x00, 0x00, // ID: 0
            0x80, 0x00, // Only is_response: true
            0x00, 0x00, // Questions: 0
            0x00, 0x00, // Answers: 0
            0x00, 0x00, // Authorities: 0
            0x00, 0x00, // Additional: 0
        ]);

        assert_eq!(
            Header::parse(&header_buf),
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
        let header_buf: BytesBuf = BytesBuf::new(vec![
            0x00, 0x00, // ID: 0
            0x70, 0x00, // Only opcode: 0xE
            0x00, 0x00, // Questions: 0
            0x00, 0x00, // Answers: 0
            0x00, 0x00, // Authorities: 0
            0x00, 0x00, // Additional: 0
        ]);

        assert_eq!(
            Header::parse(&header_buf),
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
        let header_buf: BytesBuf = BytesBuf::new(vec![
            0x00, 0x00, // ID: 0
            0x04, 0x00, // Only is_authoritative: true
            0x00, 0x00, // Questions: 0
            0x00, 0x00, // Answers: 0
            0x00, 0x00, // Authorities: 0
            0x00, 0x00, // Additional: 0
        ]);

        assert_eq!(
            Header::parse(&header_buf),
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
        let header_buf: BytesBuf = BytesBuf::new(vec![
            0x00, 0x00, // ID: 0
            0x02, 0x00, // Only is_truncated: true
            0x00, 0x00, // Questions: 0
            0x00, 0x00, // Answers: 0
            0x00, 0x00, // Authorities: 0
            0x00, 0x00, // Additional: 0
        ]);

        assert_eq!(
            Header::parse(&header_buf),
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
        let header_buf: BytesBuf = BytesBuf::new(vec![
            0x00, 0x00, // ID: 0
            0x01, 0x00, // Only should_recurse: true
            0x00, 0x00, // Questions: 0
            0x00, 0x00, // Answers: 0
            0x00, 0x00, // Authorities: 0
            0x00, 0x00, // Additional: 0
        ]);

        assert_eq!(
            Header::parse(&header_buf),
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
        let header_buf: BytesBuf = BytesBuf::new(vec![
            0x00, 0x00, // ID: 0
            0x00, 0x80, // Only recursion_available: true
            0x00, 0x00, // Questions: 0
            0x00, 0x00, // Answers: 0
            0x00, 0x00, // Authorities: 0
            0x00, 0x00, // Additional: 0
        ]);

        assert_eq!(
            Header::parse(&header_buf),
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
        let header_buf: BytesBuf = BytesBuf::new(vec![
            0x00, 0x00, // ID: 0
            0x00, 0x70, // Only _z: 0x7
            0x00, 0x00, // Questions: 0
            0x00, 0x00, // Answers: 0
            0x00, 0x00, // Authorities: 0
            0x00, 0x00, // Additional: 0
        ]);

        assert_eq!(
            Header::parse(&header_buf),
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
        let header_buf: BytesBuf = BytesBuf::new(vec![
            0x00, 0x00, // ID: 0
            0x00, 0x0E, // Only rescode: 0xE
            0x00, 0x00, // Questions: 0
            0x00, 0x00, // Answers: 0
            0x00, 0x00, // Authorities: 0
            0x00, 0x00, // Additional: 0
        ]);

        assert_eq!(
            Header::parse(&header_buf),
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
        let header_buf: BytesBuf = BytesBuf::new(vec![
            0x00, 0x00, // ID: 0
            0x00, 0x00, // Everything is 0
            0x13, 0x37, // Questions: 0x1337
            0x00, 0x00, // Answers: 0
            0x00, 0x00, // Authorities: 0
            0x00, 0x00, // Additional: 0
        ]);

        assert_eq!(
            Header::parse(&header_buf),
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
        let header_buf: BytesBuf = BytesBuf::new(vec![
            0x00, 0x00, // ID: 0
            0x00, 0x00, // Everything is 0
            0x00, 0x00, // Questions: 0
            0x13, 0x37, // Answers: 0x1337
            0x00, 0x00, // Authorities: 0
            0x00, 0x00, // Additional: 0
        ]);

        assert_eq!(
            Header::parse(&header_buf),
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
        let header_buf: BytesBuf = BytesBuf::new(vec![
            0x00, 0x00, // ID: 0
            0x00, 0x00, // Everything is 0
            0x00, 0x00, // Questions: 0
            0x00, 0x00, // Answers: 0
            0x13, 0x37, // Authorities: 0x1337
            0x00, 0x00, // Additional: 0
        ]);

        assert_eq!(
            Header::parse(&header_buf),
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
        let header_buf: BytesBuf = BytesBuf::new(vec![
            0x00, 0x00, // ID: 0
            0x00, 0x00, // Everything is 0
            0x00, 0x00, // Questions: 0
            0x00, 0x00, // Answers: 0
            0x00, 0x00, // Authorities: 0
            0x13, 0x37, // Additional: 0x1337
        ]);

        assert_eq!(
            Header::parse(&header_buf),
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
}

// 0b 10000000 00000000
