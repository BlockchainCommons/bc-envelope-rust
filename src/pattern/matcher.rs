use crate::Envelope;

pub type Path = Vec<Envelope>;

pub trait Matcher: std::fmt::Debug + Clone {
    fn paths(&self, envelope: &Envelope) -> impl Iterator<Item = Path>;

    fn is_match(&self, envelope: &Envelope) -> bool {
        self.paths(envelope).next().is_some()
    }
}
