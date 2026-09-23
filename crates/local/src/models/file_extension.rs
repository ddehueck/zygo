use serde::{Deserialize, Deserializer, Serialize, Serializer};

// todo: this is more of a helper type than it is a core model. Maybe move it to a different dir.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileExtension(String);

impl FileExtension {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into().trim_start_matches('.').to_owned())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for FileExtension {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for FileExtension {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl AsRef<str> for FileExtension {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for FileExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for FileExtension {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for FileExtension {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::new(String::deserialize(deserializer)?))
    }
}

#[cfg(test)]
mod tests {
    use super::FileExtension;

    #[test]
    fn removes_leading_dots() {
        assert_eq!(FileExtension::new("..json").as_str(), "json");
    }

    #[test]
    fn deserialization_removes_leading_dots() {
        let extension: FileExtension = serde_json::from_str(r#"".json""#).unwrap();

        assert_eq!(extension.as_str(), "json");
    }
}
