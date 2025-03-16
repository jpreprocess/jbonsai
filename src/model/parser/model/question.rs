use std::marker::PhantomData;

use nom::{
    IResult, Parser,
    bytes::complete::tag,
    combinator::cut,
    error::{ContextError, FromExternalError, ParseError, context},
    multi::separated_list0,
    sequence::{delimited, pair, preceded, separated_pair, terminated},
};

use crate::model::{parser::base::ParseTarget, voice::question};

type QuestionKVPair = (String, question::Question);

pub struct QuestionParser<T>(PhantomData<T>);

impl<S: ParseTarget> QuestionParser<S>
where
    S::Item: nom::AsChar + Clone + Copy,
    for<'a> &'a str: nom::FindToken<S::Item>,
{
    fn parse_question_ident<E: ParseError<S> + ContextError<S>>(i: S) -> IResult<S, S, E> {
        i.parse_template1(|c| c.is_ascii() && !" \n".contains(c))
    }
    fn parse_pattern_list_section<
        E: ParseError<S>
            + ContextError<S>
            + FromExternalError<S, nom::Err<jlabel_question::ParseError>>,
    >(
        i: S,
    ) -> IResult<S, question::Question, E> {
        use nom::character::complete::char;
        context(
            "pattern",
            cut(delimited(
                pair(char('{'), S::sp),
                S::parse_question,
                pair(S::sp, char('}')),
            )),
        )
        .parse(i)
    }
    fn parse_question<
        E: ParseError<S>
            + ContextError<S>
            + FromExternalError<S, nom::Err<jlabel_question::ParseError>>,
    >(
        i: S,
    ) -> IResult<S, QuestionKVPair, E> {
        context(
            "question",
            preceded(
                terminated(tag("QS"), S::sp1),
                separated_pair(
                    Self::parse_question_ident,
                    S::sp1,
                    Self::parse_pattern_list_section,
                ),
            ),
        )
        .parse(i)
        .and_then(|(rest, (name, question))| {
            Ok((rest, (name.parse_ascii_to_string()?.1, question)))
        })
    }
    pub fn parse_questions<
        E: ParseError<S>
            + ContextError<S>
            + FromExternalError<S, nom::Err<jlabel_question::ParseError>>,
    >(
        i: S,
    ) -> IResult<S, Vec<QuestionKVPair>, E> {
        use nom::character::complete::{char, space0};
        context(
            "questions",
            cut(separated_list0(
                delimited(space0, char('\n'), S::sp),
                QuestionParser::parse_question,
            )),
        )
        .parse(i)
    }
}

#[cfg(test)]
mod tests {
    use jlabel_question::{AllQuestion, position::SignedRangePosition};

    use crate::model::parser::question;

    use super::QuestionParser;

    #[test]
    fn parse_question() {
        assert_eq!(
            QuestionParser::parse_question::<nom::error::Error<&str>>(
                r#"QS C-Mora_diff_Acc-Type<=0 { "*/A:-??+*","*/A:-?+*","*/A:0+*" }"#
            ),
            Ok((
                "",
                (
                    "C-Mora_diff_Acc-Type<=0".to_string(),
                    question::Question::AllQustion(AllQuestion::SignedRange(
                        jlabel_question::Question {
                            position: SignedRangePosition::A1,
                            range: Some(-99..1),
                        }
                    ))
                )
            ))
        );
    }
}
