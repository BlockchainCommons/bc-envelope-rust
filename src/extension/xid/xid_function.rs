use anyhow::{ bail, Error, Result };

use crate::{extension::{ALL, ALL_RAW, VERIFY, VERIFY_RAW}, KnownValue};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum XIDFunction {
    All,
    Verify,
}

impl TryFrom<&KnownValue> for XIDFunction {
    type Error = Error;

    fn try_from(known_value: &KnownValue) -> Result<Self> {
        match known_value.value() {
            ALL_RAW => Ok(Self::All),
            VERIFY_RAW => Ok(Self::Verify),
            _ => bail!("Unknown XID function"),
        }
    }
}

impl From<&XIDFunction> for KnownValue {
    fn from(xid_function: &XIDFunction) -> Self {
        match xid_function {
            XIDFunction::All => ALL,
            XIDFunction::Verify => VERIFY,
        }
    }
}
