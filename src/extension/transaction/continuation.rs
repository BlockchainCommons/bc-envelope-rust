use bc_components::{PrivateKeyBase, PublicKeyBase, ARID};
use dcbor::Date;
use anyhow::{bail, Error, Result};

use crate::{known_values, Envelope, EnvelopeEncodable};

#[derive(Clone, Debug)]
pub struct Continuation {
    state: Envelope,
    valid_id: Option<ARID>,
    valid_until: Option<Date>,
}

impl PartialEq for Continuation {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
            && self.valid_id == other.valid_id
            && self.valid_until == other.valid_until
    }
}

//
// Composition
//
impl Continuation {
    pub fn new(state: impl EnvelopeEncodable) -> Self {
        Self {
            state: state.into_envelope(),
            valid_id: None,
            valid_until: None,
        }
    }

    pub fn with_valid_id(mut self, valid_id: impl AsRef<ARID>) -> Self {
        self.valid_id = Some(valid_id.as_ref().clone());
        self
    }

    pub fn with_optional_valid_id(self, valid_id: Option<impl AsRef<ARID>>) -> Self {
        if let Some(valid_id) = valid_id {
            return self.with_valid_id(valid_id);
        }
        self
    }

    pub fn with_valid_until(mut self, valid_until: impl AsRef<Date>) -> Self {
        self.valid_until = Some(valid_until.as_ref().clone());
        self
    }

    pub fn with_optional_valid_until(self, valid_until: Option<impl AsRef<Date>>) -> Self {
        if let Some(valid_until) = valid_until {
            return self.with_valid_until(valid_until);
        }
        self
    }

    pub fn with_valid_duration(self, duration: std::time::Duration) -> Self {
        self.with_valid_until(Date::now() + duration)
    }
}

//
// Parsing
//
impl Continuation {
    pub fn state(&self) -> &Envelope {
        &self.state
    }

    pub fn id(&self) -> Option<&ARID> {
        self.valid_id.as_ref()
    }

    pub fn valid_until(&self) -> Option<&Date> {
        self.valid_until.as_ref()
    }

    pub fn is_valid_date(&self, now: Option<&Date>) -> bool {
        match now {
            Some(now) => self.valid_until().map_or(true, |valid_until| valid_until > now),
            None => true,
        }
    }

    pub fn is_valid_id(&self, id: Option<&ARID>) -> bool {
        match id {
            Some(expected_id) => self.valid_id.as_ref().map_or(true, |id| id == expected_id),
            None => true,
        }
    }

    pub fn is_valid(&self, now: Option<&Date>, id: Option<&ARID>) -> bool {
        self.is_valid_date(now) && self.is_valid_id(id)
    }
}

impl From<Continuation> for Envelope {
    fn from(continuation: Continuation) -> Self {
        continuation.state
            .wrap_envelope()
            .add_optional_assertion(known_values::ID, continuation.valid_id)
            .add_optional_assertion(known_values::VALID_UNTIL, continuation.valid_until)
    }
}

impl TryFrom<(Envelope, Option<&ARID>, Option<&Date>)> for Continuation {
    type Error = Error;

    fn try_from((envelope, id, now): (Envelope, Option<&ARID>, Option<&Date>)) -> Result<Self> {
        let continuation = Self {
            state: envelope.unwrap_envelope()?,
            valid_id: envelope.extract_optional_object_for_predicate(known_values::ID)?,
            valid_until: envelope.extract_optional_object_for_predicate(known_values::VALID_UNTIL)?,
        };
        if !continuation.is_valid_date(now) {
            bail!("Continuation expired");
        }
        if !continuation.is_valid_id(id) {
            bail!("Continuation ID invalid");
        }
        Ok(continuation)
    }
}

impl From<(Continuation, &PublicKeyBase)> for Envelope {
    fn from((continuation, sender): (Continuation, &PublicKeyBase)) -> Self {
        <Continuation as Into<Self>>::into(continuation)
            .encrypt_to_recipient(sender)
    }
}

impl TryFrom<(Envelope, Option<&ARID>, Option<&Date>, &PrivateKeyBase)> for Continuation {
    type Error = Error;

    fn try_from((envelope, id, now, recipient): (Envelope, Option<&ARID>, Option<&Date>, &PrivateKeyBase)) -> Result<Self> {
        Self::try_from((envelope.decrypt_to_recipient(recipient)?, id, now))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use hex_literal::hex;
    use indoc::indoc;
    use std::time::Duration;

    fn request_id() -> ARID {
        ARID::from_data(hex!("c66be27dbad7cd095ca77647406d07976dc0f35f0d4d654bb0e96dd227a1e9fc"))
    }

    fn request_date() -> Date {
        Date::try_from("2024-07-04T11:11:11Z").unwrap()
    }

    fn request_continuation() -> Continuation {
        let valid_duration = Duration::from_secs(60);
        let valid_until = request_date() + valid_duration;
        Continuation::new("The state of things.")
            .with_valid_id(request_id())
            .with_valid_until(valid_until)
    }

    fn response_continuation() -> Continuation {
        let valid_duration = Duration::from_secs(60 * 60);
        let valid_until = request_date() + valid_duration;
        Continuation::new("The state of things.")
            .with_valid_until(valid_until)
    }

    #[test]
    fn test_request_continuation() -> Result<()> {
        crate::register_tags();

        let continuation = request_continuation();
        let envelope: Envelope = continuation.clone().into();

        // println!("{}", envelope.format());
        assert_eq!(envelope.format(),
        indoc!{r#"
        {
            "The state of things."
        } [
            'id': ARID(c66be27d)
            'validUntil': 2024-07-04T11:12:11Z
        ]
        "#}.trim());

        let parsed_continuation = Continuation::try_from((envelope, Some(&request_id()), None))?;
        assert_eq!(continuation.state(), parsed_continuation.state());
        assert_eq!(continuation.id(), parsed_continuation.id());
        assert_eq!(continuation.valid_until(), parsed_continuation.valid_until());
        assert_eq!(continuation, parsed_continuation);

        Ok(())
    }

    #[test]
    fn test_response_continuation() -> Result<()> {
        crate::register_tags();

        let continuation = response_continuation();
        let envelope: Envelope = continuation.clone().into();

        // println!("{}", envelope.format());
        assert_eq!(envelope.format(),
        indoc!{r#"
        {
            "The state of things."
        } [
            'validUntil': 2024-07-04T12:11:11Z
        ]
        "#}.trim());

        let parsed_continuation = Continuation::try_from((envelope, None, None))?;
        assert_eq!(continuation.state(), parsed_continuation.state());
        assert_eq!(continuation.id(), parsed_continuation.id());
        assert_eq!(continuation.valid_until(), parsed_continuation.valid_until());
        assert_eq!(continuation, parsed_continuation);

        Ok(())
    }

    #[test]
    fn test_encrypted_continuation() -> Result<()> {
        crate::register_tags();
        
        let sender_private_key = PrivateKeyBase::new();
        let sender_public_key = sender_private_key.schnorr_public_key_base();

        let continuation = request_continuation();
        let envelope: Envelope = (continuation.clone(), &sender_public_key).into();

        // println!("{}", envelope.format());
        assert_eq!(envelope.format(),
        indoc!{r#"
        ENCRYPTED [
            'hasRecipient': SealedMessage
        ]
        "#}.trim());

        let valid_now = Some(request_date() + Duration::from_secs(30));
        let parsed_continuation = Continuation::try_from((envelope.clone(), Some(&request_id()), valid_now.as_ref(), &sender_private_key))?;
        assert_eq!(continuation.state(), parsed_continuation.state());
        assert_eq!(continuation.id(), parsed_continuation.id());
        assert_eq!(continuation.valid_until(), parsed_continuation.valid_until());
        assert_eq!(continuation, parsed_continuation);

        let invalid_now = Some(request_date() + Duration::from_secs(90));
        let invalid_continuation_error = Continuation::try_from((envelope.clone(), Some(&request_id()), invalid_now.as_ref(), &sender_private_key));
        assert!(invalid_continuation_error.is_err());

        let invalid_id = ARID::new();
        let invalid_continuation_error = Continuation::try_from((envelope, Some(&invalid_id), valid_now.as_ref(), &sender_private_key));
        assert!(invalid_continuation_error.is_err());

        Ok(())
    }
}
