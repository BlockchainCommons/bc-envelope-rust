#[cfg(feature = "known_value")]
use bc_components::DigestProvider;
use bc_envelope::prelude::*;
use bc_rand::{
    fake_random_data, make_fake_random_number_generator,
    rng_next_in_closed_range,
};

mod common;
use crate::common::check_encoding::*;

#[cfg(feature = "known_value")]
#[test]
fn test_known_value() {
    let envelope = Envelope::new(known_values::SIGNED)
        .check_encoding()
        .unwrap();
    assert_eq!(format!("{}", envelope), ".knownValue(signed)");
    assert_eq!(
        format!("{:?}", envelope.digest()),
        "Digest(d0e39e788c0d8f0343af4588db21d3d51381db454bdf710a9a1891aaa537693c)"
    );
    assert_eq!(envelope.format(), "'signed'");
    assert_eq!(
        format!("{}", envelope.ur_string()),
        "ur:envelope/axgrbdrnem"
    );
}

#[test]
fn test_date() {
    let date = dcbor::Date::from_string("2018-01-07").unwrap();
    let envelope = Envelope::new(date).check_encoding().unwrap();
    assert_actual_expected!(envelope.format(), "2018-01-07");
}

#[test]
fn test_fake_random_data() {
    assert_eq!(
        fake_random_data(100),
        hex_literal::hex!(
            "7eb559bbbf6cce2632cf9f194aeb50943de7e1cbad54dcfab27a42759f5e2fed518684c556472008a67932f7c682125b50cb72e8216f6906358fdaf28d3545532daee0c5bb5023f50cd8e71ec14901ac746c576c481b893be6656b80622b3a564e59b4e2"
        )
    );
}

#[test]
fn test_fake_numbers() {
    let mut rng = make_fake_random_number_generator();
    let array = (0..100)
        .map(|_| rng_next_in_closed_range(&mut rng, &(-50..=50)))
        .collect::<Vec<_>>();
    assert_eq!(
        format!("{:?}", array),
        "[-43, -6, 43, -34, -34, 17, -9, 24, 17, -29, -32, -44, 12, -15, -46, 20, 50, -31, -50, 36, -28, -23, 6, -27, -31, -45, -27, 26, 31, -23, 24, 19, -32, 43, -18, -17, 6, -13, -1, -27, 4, -48, -4, -44, -6, 17, -15, 22, 15, 20, -25, -35, -33, -27, -17, -44, -27, 15, -14, -38, -29, -12, 8, 43, 49, -42, -11, -1, -42, -26, -25, 22, -13, 14, 42, -29, -38, 17, 2, 5, 5, -31, 27, -3, 39, -12, 42, 46, -17, -25, -46, -19, 16, 2, -45, 41, 12, -22, 43, -11]"
    );
}
