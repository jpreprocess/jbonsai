use jlabel_question::{regex::RegexQuestion, AllQuestion, QuestionMatcher};

#[derive(Debug, Clone)]
pub enum Question {
    AllQustion(AllQuestion),
    Regex(RegexQuestion),
}

impl Question {
    pub fn parse(patterns: &[&str]) -> Result<Self, jlabel_question::ParseError> {
        match AllQuestion::parse(patterns) {
            Ok(question) => Ok(Self::AllQustion(question)),
            Err(_) => Ok(Self::Regex(RegexQuestion::parse(patterns)?)),
        }
    }

    pub fn test(&self, label: &jlabel::Label) -> bool {
        match self {
            Self::AllQustion(q) => q.test(label),
            Self::Regex(q) => q.test(label),
        }
    }
}

impl PartialEq for Question {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::AllQustion(a), Self::AllQustion(b)) => a == b,
            // TODO: :exploding_head:
            (Self::Regex(_), Self::Regex(_)) => true,
            _ => false,
        }
    }
}
