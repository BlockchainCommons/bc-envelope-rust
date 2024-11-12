use bc_components::tags::*;
use dcbor::prelude::*;
use std::sync::{ Arc, Mutex, Once };
#[cfg(feature = "known_value")]
use crate::extension::known_values::{ KnownValuesStore, KNOWN_VALUES };

#[cfg(feature = "expression")]
use crate::extension::expressions::{
    FunctionsStore,
    ParametersStore,
    GLOBAL_FUNCTIONS,
    GLOBAL_PARAMETERS,
};
use crate::{ string_utils::StringUtils, Envelope, KnownValue };

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
/// # bc_envelope::register_tags();
/// assert_eq!(e.diagnostic_annotated(),
/// indoc! {r#"
/// 200(   / envelope /
///     201("Hello.")   / leaf /
/// )
/// "#}.trim()
/// );
/// ```
#[derive(Clone)]
pub struct FormatContext {
    flat: bool,
    tags: TagsStore,
    #[cfg(feature = "known_value")]
    known_values: KnownValuesStore,
    #[cfg(feature = "expression")]
    functions: FunctionsStore,
    #[cfg(feature = "expression")]
    parameters: ParametersStore,
}

impl FormatContext {
    pub fn new(
        flat: bool,
        tags: Option<&TagsStore>,
        #[cfg(feature = "known_value")] known_values: Option<&KnownValuesStore>,
        #[cfg(feature = "expression")] functions: Option<&FunctionsStore>,
        #[cfg(feature = "expression")] parameters: Option<&ParametersStore>
    ) -> Self {
        Self {
            flat,
            tags: tags.cloned().unwrap_or_default(),
            #[cfg(feature = "known_value")]
            known_values: known_values.cloned().unwrap_or_default(),
            #[cfg(feature = "expression")]
            functions: functions.cloned().unwrap_or_default(),
            #[cfg(feature = "expression")]
            parameters: parameters.cloned().unwrap_or_default(),
        }
    }

    pub fn is_flat(&self) -> bool {
        self.flat
    }

    pub fn set_flat(mut self, flat: bool) -> Self {
        self.flat = flat;
        self
    }

    pub fn tags(&self) -> &TagsStore {
        &self.tags
    }

    pub fn tags_mut(&mut self) -> &mut TagsStore {
        &mut self.tags
    }

    #[cfg(feature = "known_value")]
    pub fn known_values(&self) -> &KnownValuesStore {
        &self.known_values
    }

    #[cfg(feature = "expression")]
    pub fn functions(&self) -> &FunctionsStore {
        &self.functions
    }

    #[cfg(feature = "expression")]
    pub fn parameters(&self) -> &ParametersStore {
        &self.parameters
    }
}

impl TagsStoreTrait for FormatContext {
    fn assigned_name_for_tag(&self, tag: &Tag) -> Option<String> {
        self.tags.assigned_name_for_tag(tag)
    }

    fn name_for_tag(&self, tag: &Tag) -> String {
        self.tags.name_for_tag(tag)
    }

    fn tag_for_name(&self, name: &str) -> Option<Tag> {
        self.tags.tag_for_name(name)
    }

    fn tag_for_value(&self, value: u64) -> Option<Tag> {
        self.tags.tag_for_value(value)
    }

    fn summarizer(&self, tag: TagValue) -> Option<&CBORSummarizer> {
        self.tags.summarizer(tag)
    }
}

impl Default for FormatContext {
    fn default() -> Self {
        Self::new(
            false,
            None,
            #[cfg(feature = "known_value")] None,
            #[cfg(feature = "expression")] None,
            #[cfg(feature = "expression")] None
        )
    }
}

pub struct LazyFormatContext {
    init: Once,
    data: Mutex<Option<FormatContext>>,
}

impl LazyFormatContext {
    pub fn get(&self) -> std::sync::MutexGuard<'_, Option<FormatContext>> {
        self.init.call_once(|| {
            bc_components::register_tags();
            let tags_binding = dcbor::GLOBAL_TAGS.get();
            let tags = tags_binding.as_ref().unwrap();

            #[cfg(feature = "known_value")]
            let known_values_binding = KNOWN_VALUES.get();
            #[cfg(feature = "known_value")]
            let known_values = known_values_binding.as_ref().unwrap();

            #[cfg(feature = "expression")]
            let functions_binding = GLOBAL_FUNCTIONS.get();
            #[cfg(feature = "expression")]
            let functions = functions_binding.as_ref().unwrap();
            #[cfg(feature = "expression")]
            let parameters_binding = GLOBAL_PARAMETERS.get();
            #[cfg(feature = "expression")]
            let parameters = parameters_binding.as_ref().unwrap();

            let context = FormatContext::new(
                false,
                Some(tags),
                #[cfg(feature = "known_value")] Some(known_values),
                #[cfg(feature = "expression")] Some(functions),
                #[cfg(feature = "expression")] Some(parameters)
            );
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
    ($action:expr) => {
        {
        let binding = $crate::GLOBAL_FORMAT_CONTEXT.get();
        let context = &*binding.as_ref().unwrap();
        #[allow(clippy::redundant_closure_call)]
        $action(context)
        }
    };
}

#[macro_export]
macro_rules! with_format_context_mut {
    ($action:expr) => {
        {
        let mut binding = $crate::GLOBAL_FORMAT_CONTEXT.get();
        let context = binding.as_mut().unwrap();
        #[allow(clippy::redundant_closure_call)]
        $action(context)
        }
    };
}

pub fn register_tags_in(context: &mut FormatContext) {
    bc_components::register_tags_in(context.tags_mut());

    #[cfg(feature = "expression")]
    {
        use crate::extension::expressions::{ Function, FunctionsStore, Parameter, ParametersStore };

        let functions = context.functions().clone();
        context.tags_mut().set_summarizer(
            TAG_FUNCTION,
            Arc::new(move |untagged_cbor: CBOR| {
                let f = Function::from_untagged_cbor(untagged_cbor)?;
                Ok(FunctionsStore::name_for_function(&f, Some(&functions)).flanked_by("«", "»"))
            })
        );

        let parameters = context.parameters().clone();
        context.tags_mut().set_summarizer(
            TAG_PARAMETER,
            Arc::new(move |untagged_cbor: CBOR| {
                let p = Parameter::from_untagged_cbor(untagged_cbor)?;
                Ok(ParametersStore::name_for_parameter(&p, Some(&parameters)).flanked_by("❰", "❱"))
            })
        );

        let known_values = context.known_values().clone();
        context.tags_mut().set_summarizer(
            TAG_KNOWN_VALUE,
            Arc::new(move |untagged_cbor: CBOR| {
                Ok(
                    known_values
                        .name(KnownValue::from_untagged_cbor(untagged_cbor)?)
                        .flanked_by("'", "'")
                )
            })
        );

        let cloned_context = context.clone();
        context.tags_mut().set_summarizer(
            TAG_REQUEST,
            Arc::new(move |untagged_cbor: CBOR| {
                Ok(
                    Envelope::new(untagged_cbor)
                        .format_opt(Some(&cloned_context))
                        .flanked_by("request(", ")")
                )
            })
        );

        let cloned_context = context.clone();
        context.tags_mut().set_summarizer(
            TAG_RESPONSE,
            Arc::new(move |untagged_cbor: CBOR| {
                Ok(
                    Envelope::new(untagged_cbor)
                        .format_opt(Some(&cloned_context))
                        .flanked_by("response(", ")")
                )
            })
        );
    }
}

pub fn register_tags() {
    with_format_context_mut!(|context: &mut FormatContext| {
        register_tags_in(context);
    });
}
