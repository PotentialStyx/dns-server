use super::super::*;

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
