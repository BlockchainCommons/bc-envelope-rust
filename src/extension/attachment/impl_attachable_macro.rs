/// A macro for easily implementing the `Attachable` trait for types with an `attachments` field.
///
/// The `impl_attachable!` macro automatically generates an implementation of the
/// `Attachable` trait for any type that has an `attachments: Attachments` field.
/// This removes boilerplate code and ensures consistent implementations across
/// the codebase.
///
/// # Usage Requirements
/// The target type must have an `attachments: Attachments` field for this macro to work.
/// It will automatically implement the required `attachments()` and `attachments_mut()`
/// methods, giving the type access to all the default methods in the `Attachable` trait.
///
/// # Examples
/// ```no_run
/// # use zewif::{Attachments, impl_attachable, Attachable};
/// #
/// struct TransactionRecord {
///     // Core fields...
///     tx_id: String,
///     amount: i64,
///     // Required field for impl_attachable! to work
///     attachments: Attachments,
/// }
///
/// // Automatically implement the Attachable trait
/// impl_attachable!(TransactionRecord);
///
/// // Now the type has all Attachable methods available
/// let mut tx = TransactionRecord {
///     tx_id: "abcdef".to_string(),
///     amount: 500000,
///     attachments: Attachments::new(),
/// };
///
/// // Use methods provided by the Attachable trait
/// tx.add_attachment("Coffee purchase", "vendor.example", Some("memo.v1"));
/// ```
#[macro_export]
macro_rules! impl_attachable {
    ($type:ty) => {
        impl $crate::Attachable for $type {
            fn attachments(&self) -> &Attachments {
                &self.attachments
            }

            fn attachments_mut(&mut self) -> &mut Attachments {
                &mut self.attachments
            }
        }
    };
}
