use serde::{Deserialize, Serialize};

use crate::models::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NonEmptyString(String);

impl NonEmptyString {
    pub fn new(value: String, field_name: &str) -> Result<Self, DomainError> {
        if value.trim().is_empty() {
            Err(DomainError::empty(field_name))
        } else {
            Ok(Self(value))
        }
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for NonEmptyString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<NonEmptyString> for String {
    fn from(s: NonEmptyString) -> String {
        s.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_with_valid_string_succeeds() {
        let result = NonEmptyString::new("hello".to_string(), "test_field");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_ref(), "hello");
    }

    #[test]
    fn new_with_empty_string_fails() {
        let result = NonEmptyString::new("".to_string(), "test_field");
        assert!(result.is_err());
    }

    #[test]
    fn new_with_whitespace_only_fails() {
        let result = NonEmptyString::new("   ".to_string(), "test_field");
        assert!(result.is_err());
    }
}
