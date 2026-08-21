use local::Tag;

/// A single tag constraint. When `value` is `None`, any value for the tag key
/// satisfies the constraint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagFilter {
    pub key: String,
    pub value: Option<String>,
}

impl TagFilter {
    pub fn presence(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: None,
        }
    }

    pub fn exact(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: Some(value.into()),
        }
    }
}

/// The filters applied to the workflow-run list.
///
/// Every tag filter must match for a run to be included. Multiple filters with
/// the same key are therefore useful when a run has more than one value for a
/// tag.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FilterSet {
    pub tags: Vec<TagFilter>,
}

impl FilterSet {
    pub fn new(tags: Vec<TagFilter>) -> Self {
        Self { tags }
    }

    /// Creates a filter from the sidebar's tag and value inputs.
    ///
    /// An empty key means no filter. An empty value matches the presence of
    /// the key regardless of its value.
    pub fn from_inputs(key: &str, value: &str) -> Self {
        let key = key.trim();
        if key.is_empty() {
            return Self::default();
        }

        let value = value.trim();
        let tag_filter = if value.is_empty() {
            TagFilter::presence(key)
        } else {
            TagFilter::exact(key, value)
        };

        Self::new(vec![tag_filter])
    }

    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }

    pub fn matches(&self, tags: &[Tag]) -> bool {
        self.tags.iter().all(|filter| {
            tags.iter().any(|tag| {
                tag.key == filter.key
                    && filter
                        .value
                        .as_ref()
                        .is_none_or(|value| tag.value == *value)
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tag(key: &str, value: &str) -> Tag {
        Tag {
            workflow_run_id: "run-1".to_owned(),
            key: key.to_owned(),
            value: value.to_owned(),
            created_at: "now".to_owned(),
        }
    }

    #[test]
    fn inputs_support_presence_and_exact_value_filters() {
        assert_eq!(
            FilterSet::from_inputs(" env ", " "),
            FilterSet::new(vec![TagFilter::presence("env")])
        );
        assert_eq!(
            FilterSet::from_inputs(" env ", " prod "),
            FilterSet::new(vec![TagFilter::exact("env", "prod")])
        );
    }

    #[test]
    fn all_tag_filters_must_match() {
        let filters = FilterSet::new(vec![
            TagFilter::exact("env", "prod"),
            TagFilter::presence("team"),
        ]);

        assert!(filters.matches(&[tag("env", "prod"), tag("team", "desktop")]));
        assert!(!filters.matches(&[tag("env", "prod")]));
        assert!(
            FilterSet::new(vec![TagFilter::presence("env")]).matches(&[tag("env", "staging",)])
        );
    }
}
