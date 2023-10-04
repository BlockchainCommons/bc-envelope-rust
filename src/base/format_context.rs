use bc_components::tags::GLOBAL_TAGS;
use dcbor::prelude::*;
use std::sync::{Once, Mutex};
use crate::{expressions::{FunctionsStore, ParametersStore, GLOBAL_FUNCTIONS, GLOBAL_PARAMETERS}, known_values::{KnownValuesStore, KNOWN_VALUES}};

/// The envelope formatting functions take a `FormatContext` as an argument. This type
/// defines information about CBOR tags, known values, functions and parameters that
/// are used to annotate the output of the formatting functions.
///
/// The `with_format_context!` macro can be used to access the global format context:
///
/// ```
/// # use bc_envelope::{Envelope, with_format_context};
/// # use indoc::indoc;
/// # let e = Envelope::new("Hello.");
/// with_format_context!(|context| {
///     assert_eq!(e.diagnostic_opt(true, Some(context)),
///     indoc! {r#"
///     200(   / envelope /
///        24("Hello.")   / leaf /
///     )
///     "#}.trim()
///     );
/// });
/// ```
#[derive(Clone, Debug)]
pub struct FormatContext {
    tags: TagsStore,
    known_values: KnownValuesStore,
    functions: FunctionsStore,
    parameters: ParametersStore,
}

impl FormatContext {
    pub fn new(
        tags: Option<&TagsStore>,
        known_values: Option<&KnownValuesStore>,
        functions: Option<&FunctionsStore>,
        parameters: Option<&ParametersStore>,
    ) -> Self {
        Self {
            tags: tags.cloned().unwrap_or_default(),
            known_values: known_values.cloned().unwrap_or_default(),
            functions: functions.cloned().unwrap_or_default(),
            parameters: parameters.cloned().unwrap_or_default(),
        }
    }

    pub fn tags(&self) -> &TagsStore {
        &self.tags
    }

    pub fn assigned_name_for_tag(&self, tag: &Tag) -> Option<String> {
        self.tags.assigned_name_for_tag(tag)
    }

    pub fn name_for_tag(&self, tag: &Tag) -> String {
        self.tags.name_for_tag(tag)
    }

    pub fn tag_for_value(&self, value: u64) -> Option<Tag> {
        self.tags.tag_for_value(value)
    }

    pub fn tag_for_name(&self, name: &str) -> Option<Tag> {
        self.tags.tag_for_name(name)
    }

    pub fn known_values(&self) -> &KnownValuesStore {
        &self.known_values
    }

    pub fn functions(&self) -> &FunctionsStore {
        &self.functions
    }

    pub fn parameters(&self) -> &ParametersStore {
        &self.parameters
    }
}

impl Default for FormatContext {
    fn default() -> Self {
        Self::new(None, None, None, None)
    }
}

pub struct LazyFormatContext {
    init: Once,
    data: Mutex<Option<FormatContext>>,
}

impl LazyFormatContext {
    pub fn get(&self) -> std::sync::MutexGuard<'_, Option<FormatContext>> {
        self.init.call_once(|| {
            let tags_binding = GLOBAL_TAGS.get();
            let tags = tags_binding.as_ref().unwrap();
            let known_values_binding = KNOWN_VALUES.get();
            let known_values = known_values_binding.as_ref().unwrap();
            let functions_binding = GLOBAL_FUNCTIONS.get();
            let functions = functions_binding.as_ref().unwrap();
            let parameters_binding = GLOBAL_PARAMETERS.get();
            let parameters = parameters_binding.as_ref().unwrap();

            let context = FormatContext::new(Some(tags), Some(known_values), Some(functions), Some(parameters));
            *self.data.lock().unwrap() = Some(context);
        });
        self.data.lock().unwrap()
    }
}

/// Access using the `with_format_context!` macro.
pub static GLOBAL_FORMAT_CONTEXT: LazyFormatContext = LazyFormatContext {
    init: Once::new(),
    data: Mutex::new(None),
};

/// A macro to access the global format context.
#[macro_export]
macro_rules! with_format_context {
    ($action:expr) => {{
        let binding = $crate::GLOBAL_FORMAT_CONTEXT.get();
        let context = &*binding.as_ref().unwrap();
        #[allow(clippy::redundant_closure_call)]
        $action(context)
    }};
}