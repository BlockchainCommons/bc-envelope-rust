// use bc_components::{PrivateKeyBase, PublicKeyBase, ARID};
// use anyhow::Result;
// use dcbor::Date;

// use crate::{known_values, Continuation, Envelope, Function};

// /// Transaction Request Parsing
// impl Envelope {
//     /// Parses the transaction request envelope and returns the id, sender,
//     /// body, note, and date.
//     #[allow(clippy::type_complexity)]
//     pub fn parse_transaction_request_with_metadata(&self, expected_function: Option<&Function>) -> Result<(ARID, PublicKeyBase, Envelope, Function, String, Option<Date>, Option<Envelope>)> {
//         let (id, body, function, note, date) = self.parse_request_with_metadata(expected_function)?;
//         let sender: PublicKeyBase = self.extract_object_for_predicate(known_values::SENDER_PUBLIC_KEY)?;
//         let encrypted_continuation: Option<Envelope> = self.extract_optional_object_for_predicate(known_values::CONTINUATION)?;
//         Ok((id, sender, body, function, note, date, encrypted_continuation))
//     }

//     /// Parses the signed transaction request envelope and verifies the
//     /// signature from the sender, then returns the id, sender, body, note, and
//     /// date.
//     #[allow(clippy::type_complexity)]
//     pub fn parse_signed_transaction_request_with_metadata(&self, expected_function: Option<&Function>) -> Result<(ARID, PublicKeyBase, Envelope, Function, String, Option<Date>, Option<Envelope>)> {
//         let inner = self.unwrap_envelope()?;
//         let (id, sender, body, function, note, date, encrypted_continuation) = inner.parse_transaction_request_with_metadata(expected_function)?;
//         self.verify_signature_from(&sender)?;
//         Ok((id, sender, body, function, note, date, encrypted_continuation))
//     }

//     /// Decrypts the sealed transaction request envelope to the recipient,
//     /// verifies the signature from the sender, and returns the id, sender,
//     /// body, note, and date.
//     #[allow(clippy::type_complexity)]
//     pub fn parse_sealed_transaction_request_with_metadata(&self, expected_function: Option<&Function>, recipient: &PrivateKeyBase) -> Result<(ARID, PublicKeyBase, Envelope, Function, String, Option<Date>, Option<Continuation>)> {
//         let signed = self.decrypt_to_recipient(recipient)?;
//         let (id, sender, body, function, note, date, encrypted_continuation) = signed.parse_transaction_request_with_metadata(expected_function)?;
//         let mut continuation: Option<Continuation> = None;
//         if let Some(encrypted_continuation) = encrypted_continuation {
//             continuation = Some(encrypted_continuation.decrypt_to_recipient(recipient)?.try_into()?);
//         }
//         Ok((id, sender, body, function, note, date, continuation))
//     }

//     /// Parses the transaction request envelope and returns the id, sender, and
//     /// body.
//     pub fn parse_transaction_request(&self, expected_function: Option<&Function>) -> Result<(ARID, PublicKeyBase, Envelope, Function, Option<Envelope>)> {
//         let (id, body, function) = self.from_request(expected_function)?;
//         let sender: PublicKeyBase = self.extract_object_for_predicate(known_values::SENDER_PUBLIC_KEY)?;
//         let encrypted_continuation: Option<Envelope> = self.extract_optional_object_for_predicate(known_values::CONTINUATION)?;
//         Ok((id, sender, body, function, encrypted_continuation))
//     }

//     /// Parses the signed transaction request envelope and verifies the
//     /// signature from the sender, then returns the id, sender, and body.
//     pub fn parse_signed_transaction_request(&self, expected_function: Option<&Function>) -> Result<(ARID, PublicKeyBase, Envelope, Function, Option<Envelope>)> {
//         let inner = self.unwrap_envelope()?;
//         let (id, sender, body, function, encrypted_continuation) = inner.parse_transaction_request(expected_function)?;
//         self.verify_signature_from(&sender)?;
//         Ok((id, sender, body, function, encrypted_continuation))
//     }

//     /// Decrypts the sealed transaction request envelope to the recipient,
//     /// verifies the signature from the sender, and returns the id, sender, and
//     /// body.
//     pub fn parse_sealed_transaction_request(&self, expected_function: Option<&Function>, recipient: &PrivateKeyBase) -> Result<(ARID, PublicKeyBase, Envelope, Function, Option<Continuation>)> {
//         let signed = self.decrypt_to_recipient(recipient)?;
//         let (id, sender, body, function, encrypted_continuation) = signed.parse_signed_transaction_request(expected_function)?;
//         let mut continuation: Option<Continuation> = None;
//         if let Some(encrypted_continuation) = encrypted_continuation {
//             continuation = Some(encrypted_continuation.decrypt_to_recipient(recipient)?.try_into()?);
//         }
//         Ok((id, sender, body, function, continuation))
//     }
// }
