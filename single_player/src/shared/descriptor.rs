use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::char;
use std::ops::Deref;

static SEPARATOR_CHAR: char = '.';

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Descriptor<'a> {
    inner: Cow<'a, str>,
}

impl<'a> Descriptor<'a> {
    pub fn from_name(name: impl Into<Cow<'a, str>>) -> Self {
        let name = name.into();

        Self { inner: name }
    }

    pub fn object_name(&self) -> &str {
        let str = self.inner.as_ref();
        let mut first_separator = str.len();
        for (idx, char) in self.inner.as_ref().char_indices() {
            if char == SEPARATOR_CHAR {
                first_separator = idx;
                break;
            }
        }
        &str[0..first_separator]
    }

    pub fn has_tag(&self, tag_name: &str) -> bool {
        if let Some(tag_start_idx) = self.inner.find(tag_name) {
            if tag_start_idx >= 1
                && self.inner.chars().nth(tag_start_idx - 1).unwrap() == SEPARATOR_CHAR
            {
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn add_tag(&mut self, tag_name: &str) {
        if self.has_tag(tag_name) {
            return;
        }

        let mut new_tag = String::with_capacity(1 + tag_name.len());
        new_tag.push(SEPARATOR_CHAR);
        new_tag.push_str(tag_name);

        let new_inner = self.inner.as_ref().to_owned() + &new_tag;
        self.inner = Cow::Owned(new_inner);
    }
}

impl<'a> AsRef<str> for Descriptor<'a> {
    fn as_ref(&self) -> &str {
        self.inner.as_ref()
    }
}

impl<'a> Deref for Descriptor<'a> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> From<String> for Descriptor<'a> {
    fn from(name_string: String) -> Self {
        Self::from_name(name_string)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_usage() {
        let mut d = Descriptor::from_name("Player.tall.is_fast.woken_up.space between");
        assert!(d.has_tag("tall"));
        assert!(d.has_tag("is_fast"));
        assert!(d.has_tag("woken_up"));
        assert!(d.has_tag("space between"));

        assert!(!d.has_tag("untagged"));
        d.add_tag("untagged");
        assert!(d.has_tag("untagged"));

        assert_eq!(d.object_name(), "Player");
    }
}
