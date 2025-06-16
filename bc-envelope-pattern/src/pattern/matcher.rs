use bc_envelope::Envelope;

pub type Path = Vec<Envelope>;

pub trait Matcher: std::fmt::Debug + Clone {
    fn paths(&self, envelope: &Envelope) -> Vec<Path>;

    fn matches(&self, envelope: &Envelope) -> bool {
        !self.paths(envelope).is_empty()
    }
}
