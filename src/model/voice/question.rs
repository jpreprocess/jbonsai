use jlabel_question::{AllQuestion, QuestionMatcher, regex::RegexQuestion};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Question {
    AllQustion(AllQuestion),
    Regex(RegexWrap),
}

impl Question {
    pub fn parse(patterns: &[&str]) -> Result<Self, jlabel_question::ParseError> {
        match AllQuestion::parse(patterns) {
            Ok(question) => Ok(Self::AllQustion(question)),
            Err(_) => Ok(Self::Regex(RegexWrap::parse(patterns)?)),
        }
    }

    pub fn test(&self, label: &jlabel::Label) -> bool {
        match self {
            Self::AllQustion(q) => q.test(label),
            Self::Regex(q) => q.test(label),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RegexWrap {
    orig: Vec<String>,
    q: RegexQuestion,
}

impl RegexWrap {
    pub fn parse(patterns: &[&str]) -> Result<Self, jlabel_question::ParseError> {
        Ok(Self {
            orig: patterns.iter().map(|s| s.to_string()).collect(),
            q: RegexQuestion::parse(patterns)?,
        })
    }
    pub fn test(&self, label: &jlabel::Label) -> bool {
        self.q.test(label)
    }
}

impl PartialEq for RegexWrap {
    fn eq(&self, other: &Self) -> bool {
        self.orig == other.orig
    }
}

impl Serialize for RegexWrap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.orig.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RegexWrap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let orig = Vec::deserialize(deserializer)?;
        Ok(Self {
            q: RegexQuestion::parse(&orig).map_err(serde::de::Error::custom)?,
            orig,
        })
    }
}
