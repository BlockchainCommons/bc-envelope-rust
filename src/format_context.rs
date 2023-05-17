use dcbor::{KnownTagsDict, Tag, KnownTags};

use crate::{known_values::KnownValues, KnownFunctions, KnownParameters};

#[derive(Clone, Debug)]
pub struct FormatContext {
    tags: KnownTagsDict,
    known_values: KnownValues,
    functions: KnownFunctions,
    parameters: KnownParameters,
}

impl FormatContext {
    pub fn new(
        tags: Option<&KnownTagsDict>,
        known_values: Option<&KnownValues>,
        functions: Option<&KnownFunctions>,
        parameters: Option<&KnownParameters>,
    ) -> Self {
        Self {
            tags: tags.cloned().unwrap_or_else(KnownTagsDict::default),
            known_values: known_values.cloned().unwrap_or_else(KnownValues::default),
            functions: functions.cloned().unwrap_or_else(KnownFunctions::default),
            parameters: parameters.cloned().unwrap_or_else(KnownParameters::default),
        }
    }

    pub fn tags(&self) -> &KnownTagsDict {
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

    pub fn known_values(&self) -> &KnownValues {
        &self.known_values
    }

    pub fn functions(&self) -> &KnownFunctions {
        &self.functions
    }

    pub fn parameters(&self) -> &KnownParameters {
        &self.parameters
    }
}

impl Default for FormatContext {
    fn default() -> Self {
        Self::new(None, None, None, None)
    }
}
