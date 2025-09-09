use bc_components::{ARID, tags};
use dcbor::{Date, prelude::*};

use crate::{Envelope, EnvelopeEncodable, known_values, Error, Result};

/// An `Event` represents a notification or message that doesn't expect a
/// response.
///
/// Unlike `Request` and `Response` which form a pair, an `Event` is a
/// standalone message that can be used for broadcasting information, logging,
/// or publishing notifications. Events are used when the sender does not expect
/// or require a response from the recipients.
///
/// Each event contains:
/// - Content of a generic type `T` that holds the event payload
/// - A unique identifier (ARID) for tracking and correlation
/// - Optional metadata like a note and timestamp
///
/// When serialized to an envelope, events are tagged with `#6.40012`
/// (TAG_EVENT).
///
/// # Type Parameters
///
/// * `T` - The type of content this event carries. Must implement
///   `EnvelopeEncodable` and be convertible from an `Envelope`.
///
/// # Examples
///
/// ```
/// use bc_components::ARID;
/// use bc_envelope::prelude::*;
///
/// // Create a status update event
/// let event_id = ARID::new();
/// let timestamp = Date::try_from("2024-08-15T13:45:30Z").unwrap();
///
/// let status_event = Event::<String>::new("System online", event_id)
///     .with_note("Regular status update")
///     .with_date(&timestamp);
///
/// // Convert to an envelope for transmission
/// let envelope = status_event.into_envelope();
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Event<T>
where
    T: EnvelopeEncodable
        + TryFrom<Envelope>
        + std::fmt::Debug
        + Clone
        + PartialEq,
{
    content: T,
    id: ARID,
    note: String,
    date: Option<Date>,
}

impl<T> std::fmt::Display for Event<T>
where
    T: EnvelopeEncodable
        + TryFrom<Envelope>
        + std::fmt::Debug
        + Clone
        + PartialEq,
{
    /// Formats the event for display, showing its ID and content.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Event({})", self.summary())
    }
}

impl<T> Event<T>
where
    T: EnvelopeEncodable
        + TryFrom<Envelope>
        + std::fmt::Debug
        + Clone
        + PartialEq,
{
    /// Returns a human-readable summary of the event.
    pub fn summary(&self) -> String {
        format!(
            "id: {}, content: {}",
            self.id.short_description(),
            self.content.to_envelope().format_flat()
        )
    }
}

impl<T> Event<T>
where
    T: EnvelopeEncodable
        + TryFrom<Envelope>
        + std::fmt::Debug
        + Clone
        + PartialEq,
{
    /// Creates a new event with the specified content and ID.
    ///
    /// # Arguments
    ///
    /// * `content` - The payload for this event
    /// * `id` - Unique identifier for the event
    ///
    /// # Examples
    ///
    /// ```
    /// use bc_components::ARID;
    /// use bc_envelope::prelude::*;
    ///
    /// let event_id = ARID::new();
    /// let event = Event::<String>::new("Temperature alert: 90°F", event_id);
    /// ```
    pub fn new(content: impl Into<T>, id: ARID) -> Self {
        Self {
            content: content.into(),
            id,
            note: String::new(),
            date: None,
        }
    }
}

/// Trait that defines the behavior of an event.
///
/// This trait provides methods for composing events with metadata
/// and for extracting information from events.
pub trait EventBehavior<T>
where
    T: EnvelopeEncodable + TryFrom<Envelope>,
{
    //
    // Composition
    //

    /// Adds a note to the event.
    ///
    /// This provides human-readable context about the event's purpose.
    fn with_note(self, note: impl Into<String>) -> Self;

    /// Adds a date to the event.
    ///
    /// This timestamp typically represents when the event occurred.
    fn with_date(self, date: impl AsRef<Date>) -> Self;

    //
    // Parsing
    //

    /// Returns the content of the event.
    fn content(&self) -> &T;

    /// Returns the unique identifier (ARID) of the event.
    fn id(&self) -> ARID;

    /// Returns the note attached to the event, or an empty string if none
    /// exists.
    fn note(&self) -> &str;

    /// Returns the date attached to the event, if any.
    fn date(&self) -> Option<&Date>;
}

impl<T> EventBehavior<T> for Event<T>
where
    T: EnvelopeEncodable
        + TryFrom<Envelope>
        + std::fmt::Debug
        + Clone
        + PartialEq,
{
    /// Adds a note to the event.
    fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = note.into();
        self
    }

    /// Adds a date to the event.
    fn with_date(mut self, date: impl AsRef<Date>) -> Self {
        self.date = Some(date.as_ref().clone());
        self
    }

    /// Returns the content of the event.
    fn content(&self) -> &T { &self.content }

    /// Returns the ID of the event.
    fn id(&self) -> ARID { self.id }

    /// Returns the note of the event.
    fn note(&self) -> &str { &self.note }

    /// Returns the date of the event.
    fn date(&self) -> Option<&Date> { self.date.as_ref() }
}

/// Converts an `Event<T>` to an `Envelope`.
///
/// The envelope's subject is the event's ID tagged with TAG_EVENT,
/// and assertions include the event's content, note (if not empty), and date
/// (if present).
impl<T> From<Event<T>> for Envelope
where
    T: EnvelopeEncodable
        + TryFrom<Envelope>
        + std::fmt::Debug
        + Clone
        + PartialEq,
{
    fn from(event: Event<T>) -> Self {
        Envelope::new(CBOR::to_tagged_value(tags::TAG_EVENT, event.id))
            .add_assertion(known_values::CONTENT, event.content.to_envelope())
            .add_assertion_if(
                !event.note.is_empty(),
                known_values::NOTE,
                event.note,
            )
            .add_optional_assertion(known_values::DATE, event.date)
    }
}

/// Converts an `Envelope` to an `Event<T>`.
///
/// This constructor extracts the event ID, content, note, and date from an
/// envelope. The content is converted to the generic type `T`.
impl<T> TryFrom<Envelope> for Event<T>
where
    T: EnvelopeEncodable
        + TryFrom<Envelope>
        + std::fmt::Debug
        + Clone
        + PartialEq,
{
    type Error = Error;

    fn try_from(envelope: Envelope) -> Result<Self> {
        let content_envelope =
            envelope.object_for_predicate(known_values::CONTENT)?;
        let content = T::try_from(content_envelope)
            .map_err(|_| Error::msg("Failed to parse content"))?;
        Ok(Self {
            content,
            id: envelope
                .subject()
                .try_leaf()?
                .try_into_expected_tagged_value(tags::TAG_EVENT)?
                .try_into()?,
            note: envelope
                .extract_optional_object_for_predicate(known_values::NOTE)?
                .unwrap_or_default(),
            date: envelope
                .extract_optional_object_for_predicate(known_values::DATE)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use indoc::indoc;

    use super::*;

    fn request_id() -> ARID {
        ARID::from_data(hex!(
            "c66be27dbad7cd095ca77647406d07976dc0f35f0d4d654bb0e96dd227a1e9fc"
        ))
    }

    #[test]
    fn test_event() {
        crate::register_tags();

        let event_date = Date::try_from("2024-07-04T11:11:11Z").unwrap();
        let event = Event::<String>::new("test", request_id())
            .with_note("This is a test")
            .with_date(&event_date);

        let envelope: Envelope = event.clone().into();
        #[rustfmt::skip]
        let expected = indoc!{r#"
            event(ARID(c66be27d)) [
                'content': "test"
                'date': 2024-07-04T11:11:11Z
                'note': "This is a test"
            ]
        "#}.trim();
        assert_eq!(envelope.format(), expected);

        let parsed_request = Event::<String>::try_from(envelope).unwrap();
        assert_eq!(parsed_request.content(), "test");
        assert_eq!(parsed_request.note(), "This is a test");
        assert_eq!(parsed_request.date(), Some(&event_date));
    }
}
