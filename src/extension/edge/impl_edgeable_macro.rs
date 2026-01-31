/// A macro for easily implementing the `Edgeable` trait for types with an
/// `edges` field.
///
/// The target type must have an `edges: Edges` field for this macro to work.
#[macro_export]
macro_rules! impl_edgeable {
    ($type:ty) => {
        impl $crate::Edgeable for $type {
            fn edges(&self) -> &Edges { &self.edges }

            fn edges_mut(&mut self) -> &mut Edges { &mut self.edges }
        }
    };
}
