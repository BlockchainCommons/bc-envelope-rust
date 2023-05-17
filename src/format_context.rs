use dcbor::{KnownTagsDict, Tag, KnownTags};

use crate::{known_values::KnownValues, KnownFunctions, KnownParameters};

#[derive(Clone, Debug)]
pub struct FormatContext<'a> {
    tags: Option<&'a KnownTagsDict>,
    known_values: Option<&'a KnownValues>,
    functions: Option<&'a KnownFunctions>,
    parameters: Option<&'a KnownParameters>,
}

impl<'a> FormatContext<'a> {
    pub fn new(
        tags: Option<&'a KnownTagsDict>,
        known_values: Option<&'a KnownValues>,
        functions: Option<&'a KnownFunctions>,
        parameters: Option<&'a KnownParameters>,
    ) -> Self {
        Self {
            tags,
            known_values,
            functions,
            parameters,
        }
    }

    pub fn assigned_name_for_tag(&self, tag: &Tag) -> Option<String> {
        self.tags.as_ref().and_then(|tags| tags.assigned_name_for_tag(tag))
    }

    pub fn name_for_tag(&self, tag: &Tag) -> String {
        self.tags.as_ref().map_or_else(|| tag.value().to_string(), |tags| tags.name_for_tag(tag))
    }

    pub fn tag_for_value(&self, value: u64) -> Option<Tag> {
        self.tags.as_ref().and_then(|tags| tags.tag_for_value(value))
    }

    pub fn tag_for_name(&self, name: &str) -> Option<Tag> {
        self.tags.as_ref().and_then(|tags| tags.tag_for_name(name))
    }

    pub fn known_values(&self) -> Option<&'a KnownValues> {
        self.known_values
    }

    pub fn functions(&self) -> Option<&'a KnownFunctions> {
        self.functions
    }

    pub fn parameters(&self) -> Option<&'a KnownParameters> {
        self.parameters
    }
}
