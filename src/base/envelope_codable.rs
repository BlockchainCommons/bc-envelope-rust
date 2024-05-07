use crate::{Envelope, EnvelopeEncodable};

pub trait EnvelopeCodable { }

impl<T> EnvelopeCodable for T where T: TryFrom<Envelope> + EnvelopeEncodable { }
