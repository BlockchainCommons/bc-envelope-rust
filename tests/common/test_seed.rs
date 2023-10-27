use anyhow::bail;
use bc_components::tags;
use bc_ur::prelude::*;

use bc_envelope::prelude::*;

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
        T: AsRef<[u8]>,
    {
        Self::new_opt(data, String::new(), String::new(), None)
    }

    pub fn new_opt<T, S, U>(data: T, name: S, note: U, creation_date: Option<dcbor::Date>) -> Self
    where
        T: AsRef<[u8]>,
        S: AsRef<str>,
        U: AsRef<str>,
    {
        Self {
            data: data.as_ref().to_vec(),
            name: name.as_ref().to_string(),
            note: note.as_ref().to_string(),
            creation_date,
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: impl AsRef<str>) {
        self.name = name.as_ref().to_string();
    }

    pub fn note(&self) -> &str {
        &self.note
    }

    pub fn set_note(&mut self, note: impl AsRef<str>) {
        self.note = note.as_ref().to_string();
    }

    pub fn creation_date(&self) -> Option<&dcbor::Date> {
        self.creation_date.as_ref()
    }

    pub fn set_creation_date(&mut self, creation_date: Option<impl AsRef<dcbor::Date>>) {
        self.creation_date = creation_date.map(|x| x.as_ref().clone());
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

impl EnvelopeEncodable for &Seed {
    fn envelope(self) -> Envelope {
        let mut e = Envelope::new(CBOR::byte_string(self.data()))
            .add_type(known_values::SEED_TYPE)
            .add_optional_assertion(known_values::DATE, self.creation_date());

        if !self.name().is_empty() {
            e = e.add_assertion(known_values::HAS_NAME, self.name());
        }

        if !self.note().is_empty() {
            e = e.add_assertion(known_values::NOTE, self.note());
        }

        e
    }
}

impl Seed {
    pub fn from_envelope(envelope: &Envelope) -> anyhow::Result<Self> {
        envelope.clone().check_type(&known_values::SEED_TYPE)?;
        let data = envelope.clone().subject().expect_leaf()?.expect_byte_string()?.to_vec();
        let name = envelope.clone().extract_optional_object_for_predicate::<String>(known_values::HAS_NAME)?.unwrap_or_default().to_string();
        let note = envelope.clone().extract_optional_object_for_predicate::<String>(known_values::NOTE)?.unwrap_or_default().to_string();
        let creation_date: Option<dcbor::Date> = envelope.clone().extract_optional_object_for_predicate::<dcbor::Date>(known_values::DATE)?.map(|s| s.as_ref().clone());
        Ok(Self::new_opt(data, name, note, creation_date))
    }
}
