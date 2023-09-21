use std::rc::Rc;

use anyhow::bail;
use bc_components::tags;
use bc_ur::preamble::*;

use bc_envelope::{IntoEnvelope, Envelope};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Seed {
    data: Vec<u8>,
    name: String,
    note: String,
    creation_date: Option<dcbor::Date>,
}

impl Seed {
    pub fn new<T>(data: T) -> Self
    where
        T: Into<Vec<u8>>,
    {
        Self::new_opt(data, String::new(), String::new(), None)
    }

    pub fn new_opt<T>(data: T, name: String, note: String, creation_date: Option<dcbor::Date>) -> Self
    where
        T: Into<Vec<u8>>,
    {
        Self {
            data: data.into(),
            name,
            note,
            creation_date,
        }
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    pub fn note(&self) -> &str {
        &self.note
    }

    pub fn set_note(&mut self, note: &str) {
        self.note = note.to_string();
    }

    pub fn creation_date(&self) -> &Option<dcbor::Date> {
        &self.creation_date
    }

    pub fn set_creation_date(&mut self, creation_date: Option<dcbor::Date>) {
        self.creation_date = creation_date;
    }
}

impl CBORTagged for Seed {
    const CBOR_TAG: Tag = tags::SEED;
}

impl CBOREncodable for Seed {
    fn cbor(&self) -> CBOR {
        self.tagged_cbor()
    }
}

impl CBORTaggedEncodable for Seed {
    fn untagged_cbor(&self) -> CBOR {
        let mut map = dcbor::Map::new();
        map.insert_into(1, CBOR::byte_string(self.data()));
        if let Some(creation_date) = self.creation_date() {
            map.insert_into(2, creation_date);
        }
        if !self.name().is_empty() {
            map.insert_into(3, self.name());
        }
        if !self.note().is_empty() {
            map.insert_into(4, self.note());
        }
        map.cbor()
    }
}

impl UREncodable for Seed { }

impl CBORDecodable for Seed {
    fn from_cbor(cbor: &CBOR) -> anyhow::Result<Self> {
        Self::from_tagged_cbor(cbor)
    }
}

impl CBORTaggedDecodable for Seed {
    fn from_untagged_cbor(cbor: &CBOR) -> anyhow::Result<Self> {
        let map = cbor.expect_map()?;
        let data = map.extract::<i32, CBOR>(1)?.expect_byte_string()?.to_vec();
        if data.is_empty() {
            bail!("invalid seed data");
        }
        let creation_date = map.get::<i32, dcbor::Date>(2);
        let name = map.get::<i32, String>(3).unwrap_or_default();
        let note = map.get::<i32, String>(4).unwrap_or_default();
        Ok(Self::new_opt(data, name, note, creation_date))
    }
}

impl URDecodable for Seed { }

impl URCodable for Seed { }

impl IntoEnvelope for &Seed {
    fn into_envelope(self) -> Rc<Envelope> {
        self.cbor().into_envelope()
    }
}

impl IntoEnvelope for Seed {
    fn into_envelope(self) -> Rc<Envelope> {
        self.cbor().into_envelope()
    }
}
