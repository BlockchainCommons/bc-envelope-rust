use std::error::Error;

use crate::{known_value, Envelope};
use bc_components::DigestProvider;
use bc_crypto::{fake_random_data, make_fake_random_number_generator, RandomNumberGenerator};
use bc_ur::UREncodable;
use dcbor::Date;

#[test]
fn test_known_value() -> Result<(), Box<dyn Error>> {
    let envelope = Envelope::new(known_value::VERIFIED_BY).check_encoding()?;
    assert_eq!(format!("{}", envelope), ".knownValue(verifiedBy)");
    assert_eq!(format!("{:?}", envelope.digest()), "Digest(9d7ba9eb8986332bf3e6f3f96b36d937176d95b556441b18612b9c06edc9b7e1)");
    assert_eq!(envelope.format(), "verifiedBy");
    assert_eq!(format!("{}", envelope.ur_string()), "ur:envelope/tpsgaxtystteve");
    Ok(())
}

#[test]
fn test_date() -> Result<(), Box<dyn Error>> {
    let envelope = Envelope::new_leaf(Date::from_str("2018-01-07").unwrap()).check_encoding()?;
    assert_eq!(envelope.format(), "2018-01-07");
    Ok(())
}

#[test]
fn test_fake_random_data() {
    assert_eq!(fake_random_data(100), hex_literal::hex!("7eb559bbbf6cce2632cf9f194aeb50943de7e1cbad54dcfab27a42759f5e2fed518684c556472008a67932f7c682125b50cb72e8216f6906358fdaf28d3545532daee0c5bb5023f50cd8e71ec14901ac746c576c481b893be6656b80622b3a564e59b4e2"));
}

#[test]
fn test_fake_numbers() {
    let mut rng = make_fake_random_number_generator();
    let array = (0..100).map(|_| rng.next_in_closed_range(&(-50..=50))).collect::<Vec<_>>();
    assert_eq!(format!("{:?}", array), "[-43, -6, 43, -34, -34, 17, -9, 24, 17, -29, -32, -44, 12, -15, -46, 20, 50, -31, -50, 36, -28, -23, 6, -27, -31, -45, -27, 26, 31, -23, 24, 19, -32, 43, -18, -17, 6, -13, -1, -27, 4, -48, -4, -44, -6, 17, -15, 22, 15, 20, -25, -35, -33, -27, -17, -44, -27, 15, -14, -38, -29, -12, 8, 43, 49, -42, -11, -1, -42, -26, -25, 22, -13, 14, 42, -29, -38, 17, 2, 5, 5, -31, 27, -3, 39, -12, 42, 46, -17, -25, -46, -19, 16, 2, -45, 41, 12, -22, 43, -11]");
}
