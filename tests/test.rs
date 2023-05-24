use std::path::PathBuf;

use gw2_strs::*;

fn make_reader(file: &str) -> Reader {
    let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "ref", "tests", file]
        .iter()
        .collect();
    let buffer = std::fs::read(path).unwrap();
    Reader::from(buffer.into()).unwrap()
}

#[test]
fn test_language() {
    let r = make_reader("2440761");
    assert_eq!(r.language, Language::English);
    let r = make_reader("2444710");
    assert_eq!(r.language, Language::Chinese);
}

#[test]
fn test_raw() {
    let r = make_reader("2440761");
    assert_eq!(r.get_string(0).unwrap(), "Skale Toxin");
    assert_eq!(r.get_string(142).unwrap(), "Charge Shot");
    assert_eq!(r.get_string(512).unwrap(), "Cotton Shoe Upper[s]");
    assert_eq!(
        r.get_string(869).unwrap(),
        "Spin attack to knock back foes."
    );
    assert_eq!(r.get_string(1020).unwrap(), "Stun");
    let r = make_reader("2444710");
    assert_eq!(r.get_string(24).unwrap(), "未鉴定的红色染料");
    assert_eq!(
        r.get_string(376).unwrap(),
        "挡住掘洞人，保护投石修理喳喳车。"
    );
    assert_eq!(r.get_string(419).unwrap(), "%str1%：\n\n狮子拱门的重建工作已经全面启动。虽然我们的朋友和泰瑞亚的同胞都来帮忙，但是进展依然缓慢。\n\n珍娜女王给予了我们帮助，商贸伙伴风裔也即将前来支援，他们此行不仅是为了再次兑现贸易协定，还要亲眼目睹绯红的惨败。\n\n我希望为风裔献上最隆重的接待仪式。因此，我邀请诸位英雄与我一起迎接他们的归来。如果你或是任何与你一同冒险的朋友能来狮子拱门与我一起为风裔接风，那将是我的荣幸。\n\n在此先行谢过。期望你的到来。\n\n——爱伦·齐尔船长");
}

#[test]
fn test_missing_key() {
    let r = make_reader("2440761");
    assert_eq!(r.get_string(5).unwrap_err(), Error::NoEncryptionKeyProvided);
    assert_eq!(
        r.get_string(1021).unwrap_err(),
        Error::NoEncryptionKeyProvided
    );
}

#[test]
fn test_empty() {
    let r = make_reader("2472403");
    assert_eq!(r.get_string(0).unwrap(), "");
    assert_eq!(r.get_string(511).unwrap(), "");
    assert_eq!(r.get_string(1023).unwrap(), "");
}

#[test]
fn test_encrypted() {
    let r = make_reader("2441520");
    assert_eq!(
        r.get_encrypted_string(724, 14399848955341).unwrap(),
        "Outlaw Outfit"
    );
}
