use std::rc::Rc;

use crate::Envelope;

pub trait FromEnvelope {
    fn from_envelope(envelope: Rc<Envelope>) -> anyhow::Result<Self>
    where
        Self: Sized;
}
