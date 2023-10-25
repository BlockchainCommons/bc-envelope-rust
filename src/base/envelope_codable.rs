use crate::{EnvelopeEncodable, EnvelopeDecodable};

pub trait EnvelopeCodable: EnvelopeEncodable + EnvelopeDecodable { }
