use bc_components::tags::*;
#[cfg(any(feature = "expression", feature = "known_value"))]
use std::sync::Arc;
use std::sync::{ Mutex, Once };
#[cfg(feature = "known_value")]
use known_values::{ KnownValuesStore, KnownValue, KNOWN_VALUES };
#[cfg(feature = "known_value")]
use bc_components::tags::TAG_KNOWN_VALUE;

#[cfg(feature = "expression")]
use crate::extension::expressions::{
    FunctionsStore,
    ParametersStore,
    GLOBAL_FUNCTIONS,
    GLOBAL_PARAMETERS,
};

#[cfg(feature = "expression")]
use crate::Envelope;

#[cfg(any(feature = "expression", feature = "known_value"))]
use crate::string_utils::StringUtils;

#[derive(Clone)]
pub enum FormatContextOpt {
    None,
    Global,
    Custom(&'static FormatContext),
}

impl Default for FormatContextOpt {
    fn default() -> Self {
        FormatContextOpt::Global
    }
}

/// Context object for formatting Gordian Envelopes with annotations.
///
/// The `FormatContext` provides information about CBOR tags, known values, functions,
/// and parameters that are used to annotate the output of envelope formatting functions.
/// This context enables human-readable output when converting envelopes to string
/// representations like diagnostic notation.
///
/// This type is central to the diagnostic capabilities of Gordian Envelope,
/// translating numeric CBOR tags into meaningful names and providing context-specific
/// formatting for special values.
///
/// # Format Context Content
///
/// A `FormatContext` contains:
/// - CBOR tag registry (always present)
/// - Known Values store (when `known_value` feature is enabled)
/// - Functions store (when `expression` feature is enabled)
/// - Parameters store (when `expression` feature is enabled)
/// - A flag indicating whether the format should be flat or structured
///
/// # Global Context
///
/// A global format context is available through the `with_format_context!` and
/// `with_format_context_mut!` macros. This global context is initialized with
/// standard tags and registries.
///
/// # Example
///
/// Using the global format context to produce annotated CBOR diagnostic notation:
///
/// ```
/// # use bc_envelope::prelude::*;
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
///
/// The annotations (comments after the `/` characters) provide human-readable context
/// for the CBOR tags and structure.
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
    /// Creates a new format context with the specified components.
    ///
    /// This constructor allows full customization of the format context by providing
    /// optional components. Any component not provided will be initialized with its default.
    ///
    /// # Parameters
    ///
    /// * `flat` - If true, formatting will be flattened without indentation and structure
    /// * `tags` - Optional CBOR tag registry for mapping tag numbers to names
    /// * `known_values` - Optional known values registry (requires `known_value` feature)
    /// * `functions` - Optional functions registry (requires `expression` feature)
    /// * `parameters` - Optional parameters registry (requires `expression` feature)
    ///
    /// # Returns
    ///
    /// A new `FormatContext` instance initialized with the provided components.
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

    /// Returns whether flat formatting is enabled.
    ///
    /// When flat formatting is enabled, envelope formatting functions produce
    /// more compact output without indentation and structural formatting.
    pub fn is_flat(&self) -> bool {
        self.flat
    }

    /// Sets whether flat formatting should be enabled and returns the modified context.
    ///
    /// This method allows fluent-style modification of the context's flat formatting setting.
    ///
    /// # Parameters
    ///
    /// * `flat` - If true, flat formatting will be enabled
    ///
    /// # Returns
    ///
    /// A new `FormatContext` with the updated flat setting
    pub fn set_flat(mut self, flat: bool) -> Self {
        self.flat = flat;
        self
    }

    /// Returns a reference to the CBOR tags registry.
    ///
    /// The tags registry maps CBOR tag numbers to human-readable names
    /// and provides summarizers for tag-specific formatting.
    pub fn tags(&self) -> &TagsStore {
        &self.tags
    }

    /// Returns a mutable reference to the CBOR tags registry.
    ///
    /// This allows modifying the tags registry to add or change tag mappings.
    pub fn tags_mut(&mut self) -> &mut TagsStore {
        &mut self.tags
    }

    /// Returns a reference to the known values registry.
    ///
    /// The known values registry maps symbolic values (like "true", "false", etc.)
    /// to their canonical string representations.
    ///
    /// This method is only available when the `known_value` feature is enabled.
    #[cfg(feature = "known_value")]
    pub fn known_values(&self) -> &KnownValuesStore {
        &self.known_values
    }

    /// Returns a reference to the functions registry.
    ///
    /// The functions registry maps function identifiers to their human-readable names
    /// for use in expression formatting.
    ///
    /// This method is only available when the `expression` feature is enabled.
    #[cfg(feature = "expression")]
    pub fn functions(&self) -> &FunctionsStore {
        &self.functions
    }

    /// Returns a reference to the parameters registry.
    ///
    /// The parameters registry maps parameter identifiers to their human-readable names
    /// for use in expression formatting.
    ///
    /// This method is only available when the `expression` feature is enabled.
    #[cfg(feature = "expression")]
    pub fn parameters(&self) -> &ParametersStore {
        &self.parameters
    }
}

/// Implementation of `TagsStoreTrait` for `FormatContext`, delegating to the internal `TagsStore`.
///
/// This implementation allows a `FormatContext` to be used anywhere a `TagsStoreTrait`
/// is required, providing the tag resolution functionality directly.
impl TagsStoreTrait for FormatContext {
    /// Returns the assigned name for a tag if one exists.
    fn assigned_name_for_tag(&self, tag: &Tag) -> Option<String> {
        self.tags.assigned_name_for_tag(tag)
    }

    /// Returns a name for a tag, either the assigned name or a generic representation.
    fn name_for_tag(&self, tag: &Tag) -> String {
        self.tags.name_for_tag(tag)
    }

    /// Looks up a tag by its name.
    fn tag_for_name(&self, name: &str) -> Option<Tag> {
        self.tags.tag_for_name(name)
    }

    /// Looks up a tag by its numeric value.
    fn tag_for_value(&self, value: TagValue) -> Option<Tag> {
        self.tags.tag_for_value(value)
    }

    /// Returns a CBOR summarizer for a tag value if one exists.
    fn summarizer(&self, tag: TagValue) -> Option<&CBORSummarizer> {
        self.tags.summarizer(tag)
    }

    /// Returns a name for a tag value, either the assigned name or a generic representation.
    fn name_for_value(&self, value: TagValue) -> String {
        self.tags.name_for_value(value)
    }
}

/// Default implementation for `FormatContext`, creating an instance with default components.
impl Default for FormatContext {
    /// Creates a default `FormatContext` with:
    /// - Flat formatting disabled (structured formatting)
    /// - Default tag registry
    /// - Default known values store (when `known_value` feature is enabled)
    /// - Default functions store (when `expression` feature is enabled)
    /// - Default parameters store (when `expression` feature is enabled)
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

/// A container for lazily initializing the global format context.
///
/// This type ensures the global format context is only initialized once,
/// even in multithreaded contexts, and provides thread-safe access to the context.
pub struct LazyFormatContext {
    /// Initialization flag to ensure one-time initialization
    init: Once,
    /// Thread-safe storage for the format context
    data: Mutex<Option<FormatContext>>,
}

impl LazyFormatContext {
    /// Gets a thread-safe reference to the format context, initializing it if necessary.
    ///
    /// On first access, this method initializes the format context with standard
    /// registrations for tags, known values, functions, and parameters.
    ///
    /// # Returns
    ///
    /// A mutex guard containing a reference to the global format context.
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

/// Global singleton instance of `FormatContext` for application-wide use.
///
/// Access this using the `with_format_context!` macro.
pub static GLOBAL_FORMAT_CONTEXT: LazyFormatContext = LazyFormatContext {
    init: Once::new(),
    data: Mutex::new(None),
};

/// A macro to access the global format context for read-only operations.
///
/// This macro provides a convenient way to use the global format context without
/// dealing with mutex locking and unlocking directly.
///
/// # Example
///
/// ```
/// # use bc_envelope::prelude::*;
/// # let e = Envelope::new("Hello.");
/// let formatted = with_format_context!(|ctx| e.format_opt(Some(ctx)));
/// ```
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

/// A macro to access the global format context for read-write operations.
///
/// This macro provides a convenient way to use and modify the global format context
/// without dealing with mutex locking and unlocking directly.
///
/// # Example
///
/// ```
/// # use bc_envelope::prelude::*;
/// # use bc_envelope::with_format_context_mut;
/// with_format_context_mut!(|ctx: &mut FormatContext| {
///     // Use a mutable reference method instead of set_flat which consumes self
///     let flat_setting = ctx.is_flat();
///     // Do something with the flat setting
///     assert!(!flat_setting);
/// });
/// ```
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

/// Registers standard tags and summarizers in a format context.
///
/// This function populates a format context with standard tag definitions
/// and summarizers for various envelope-related types, enabling proper
/// annotation of formatted output.
///
/// # Parameters
///
/// * `context` - The format context to register tags in
pub fn register_tags_in(context: &mut FormatContext) {
    // Register standard component tags
    bc_components::register_tags_in(context.tags_mut());

    #[cfg(feature = "known_value")]
    {
        // Known value summarizer - formats known values with single quotes
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
    }

    // Register expression-related summarizers when the expression feature is enabled
    #[cfg(feature = "expression")]
    {
        use crate::extension::expressions::{ Function, FunctionsStore, Parameter, ParametersStore };

        // Function summarizer - formats functions with special delimiter characters
        let functions = context.functions().clone();
        context.tags_mut().set_summarizer(
            TAG_FUNCTION,
            Arc::new(move |untagged_cbor: CBOR| {
                let f = Function::from_untagged_cbor(untagged_cbor)?;
                Ok(FunctionsStore::name_for_function(&f, Some(&functions)).flanked_by("«", "»"))
            })
        );

        // Parameter summarizer - formats parameters with special delimiter characters
        let parameters = context.parameters().clone();
        context.tags_mut().set_summarizer(
            TAG_PARAMETER,
            Arc::new(move |untagged_cbor: CBOR| {
                let p = Parameter::from_untagged_cbor(untagged_cbor)?;
                Ok(ParametersStore::name_for_parameter(&p, Some(&parameters)).flanked_by("❰", "❱"))
            })
        );

        // Request summarizer - formats requests with request() notation
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

        // Response summarizer - formats responses with response() notation
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

        // Event summarizer - formats events with event() notation
        let cloned_context = context.clone();
        context.tags_mut().set_summarizer(
            TAG_EVENT,
            Arc::new(move |untagged_cbor: CBOR| {
                Ok(
                    Envelope::new(untagged_cbor)
                        .format_opt(Some(&cloned_context))
                        .flanked_by("event(", ")")
                )
            })
        );
    }
}

/// Registers standard tags in the global format context.
///
/// This function uses the global format context and registers standard tags
/// using the `register_tags_in` function. It's a convenience wrapper for
/// registering tags in the global context.
pub fn register_tags() {
    with_format_context_mut!(|context: &mut FormatContext| {
        register_tags_in(context);
    });
}
