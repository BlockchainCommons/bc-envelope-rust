use std::error::Error;

use crate::{known_value, Envelope};
use bc_components::DigestProvider;
use bc_ur::UREncodable;

#[test]
fn test_known_value() -> Result<(), Box<dyn Error>> {
    let envelope = Envelope::new_with_known_value(known_value::VERIFIED_BY).check_encoding()?;
    assert_eq!(format!("{}", envelope), ".knownValue(verifiedBy)");
    assert_eq!(format!("{:?}", envelope.digest()), "Digest(9d7ba9eb8986332bf3e6f3f96b36d937176d95b556441b18612b9c06edc9b7e1)");
    assert_eq!(envelope.format(), "verifiedBy");
    assert_eq!(format!("{}", envelope.ur_string()), "ur:envelope/tpsgaxtystteve");
    Ok(())
}
