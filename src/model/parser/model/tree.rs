use std::marker::PhantomData;

use nom::{
    AsChar, IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{cut, map, opt, peek, recognize},
    error::{ContextError, ErrorKind, FromExternalError, ParseError, context},
    multi::{many_m_n, separated_list0},
    sequence::{delimited, pair, preceded},
};

use crate::model::parser::base::ParseTarget;

#[derive(Debug, Clone, PartialEq)]
pub struct Tree {
    pub state: usize,
    pub nodes: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub id: isize,
    pub question_name: String,
    pub yes: TreeIndex,
    pub no: TreeIndex,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeIndex {
    Node(isize),
    Pdf(isize),
}

pub struct TreeParser<T>(PhantomData<T>);

impl<S: ParseTarget> TreeParser<S>
where
    S::Item: nom::AsChar + Clone + Copy,
    for<'a> &'a str: nom::FindToken<S::Item>,
{
    fn parse_signed_digits<E: ParseError<S> + ContextError<S>>(i: S) -> IResult<S, isize, E> {
        use nom::character::complete::char;
        recognize(pair(opt(char('-')), digit1))
            .parse(i)
            .and_then(|(rest, number)| match number.parse_to() {
                Some(n) => Ok((rest, n)),
                None => Err(nom::Err::Error(E::from_error_kind(
                    number,
                    ErrorKind::Float,
                ))),
            })
    }
    fn parse_question_ident<E: ParseError<S> + ContextError<S>>(i: S) -> IResult<S, S, E> {
        i.parse_template1(|c| c.is_ascii() && !" \n".contains(c))
    }
    fn parse_tree_index<E: ParseError<S> + ContextError<S>>(i: S) -> IResult<S, TreeIndex, E> {
        let pdf_index = move |i| {
            S::parse_identifier(i).and_then(|(rest, input)| {
                let mut id_str = String::new();
                for c in input.iter_elements() {
                    let c = c.as_char();
                    if c.is_ascii_digit() {
                        id_str.push(c)
                    } else {
                        id_str.clear();
                    }
                }
                match id_str.parse() {
                    Ok(i) => Ok((rest, TreeIndex::Pdf(i))),
                    Err(_) => Err(nom::Err::Error(E::from_error_kind(input, ErrorKind::Digit))),
                }
            })
        };
        let tree_index =
            move |i| alt((map(Self::parse_signed_digits, TreeIndex::Node), pdf_index)).parse(i);
        use nom::character::complete::char;
        context(
            "tree_index",
            alt((tree_index, delimited(char('\"'), tree_index, char('\"')))),
        )
        .parse(i)
    }
    fn parse_node<E: ParseError<S> + ContextError<S>>(i: S) -> IResult<S, Node, E> {
        context(
            "node",
            (
                preceded(S::sp, Self::parse_signed_digits),
                preceded(S::sp1, Self::parse_question_ident),
                preceded(S::sp1, Self::parse_tree_index),
                preceded(S::sp1, Self::parse_tree_index),
            ),
        )
        .parse(i)
        .and_then(|(rest, (id, question_name, no, yes))| {
            Ok((
                rest,
                Node {
                    id,
                    question_name: question_name.parse_ascii_to_string()?.1,
                    yes,
                    no,
                },
            ))
        })
    }
    fn parse_tree<
        E: ParseError<S>
            + ContextError<S>
            + FromExternalError<S, nom::Err<jlabel_question::ParseError>>,
    >(
        i: S,
    ) -> IResult<S, Tree, E> {
        use nom::character::complete::{char, space0};
        context(
            "tree",
            cut((
                preceded(S::sp, tag("{*}")),
                preceded(
                    S::sp,
                    cut(delimited(char('['), Self::parse_signed_digits, char(']'))),
                ),
                preceded(
                    S::sp,
                    cut(alt((
                        many_m_n(
                            1,
                            1,
                            map(Self::parse_tree_index, |index| Node {
                                id: 0,
                                question_name: "".to_string(),
                                yes: index.clone(),
                                no: index.clone(),
                            }),
                        ),
                        delimited(
                            pair(char('{'), S::sp),
                            separated_list0(
                                delimited(space0, char('\n'), space0),
                                Self::parse_node,
                            ),
                            pair(S::sp, char('}')),
                        ),
                    ))),
                ),
            )),
        )
        .parse(i)
        .map(|(rest, (_, state, nodes))| {
            (
                rest,
                Tree {
                    state: state as usize,
                    nodes,
                },
            )
        })
    }
    pub fn parse_trees<
        E: ParseError<S>
            + ContextError<S>
            + FromExternalError<S, nom::Err<jlabel_question::ParseError>>,
    >(
        i: S,
    ) -> IResult<S, Vec<Tree>, E> {
        use nom::character::complete::{char, none_of, space0};
        context(
            "trees",
            cut(separated_list0(
                delimited(space0, char('\n'), pair(S::sp, peek(none_of(" \n")))),
                Self::parse_tree,
            )),
        )
        .parse(i)
    }
}

#[cfg(test)]
mod tests {
    use super::{Node, Tree, TreeIndex, TreeParser};

    #[test]
    fn parse_node() {
        assert_eq!(
            TreeParser::parse_node::<nom::error::Error<&str>>(concat!(
                r#" -235 R-Phone_Boin_E                                       -236          "dur_s2_230" "#,
                "\n}",
            )),
            Ok((
                " \n}",
                Node {
                    id: -235,
                    question_name: "R-Phone_Boin_E".to_string(),
                    yes: TreeIndex::Pdf(230),
                    no: TreeIndex::Node(-236),
                }
            ))
        );
    }

    #[test]
    fn parse_tree() {
        assert_eq!(
            TreeParser::parse_tree::<nom::error::Error<&str>>(
                r#"{*}[2]
{
    0 Utt_Len_Mora<=28                                    "gv_lf0_1"          -1      
    -1 Utt_Len_Mora=18                                     "gv_lf0_3"       "gv_lf0_2" 
}"#
            ),
            Ok((
                "",
                Tree {
                    state: 2,
                    nodes: vec![
                        Node {
                            id: 0,
                            question_name: "Utt_Len_Mora<=28".to_string(),
                            yes: TreeIndex::Node(-1),
                            no: TreeIndex::Pdf(1),
                        },
                        Node {
                            id: -1,
                            question_name: "Utt_Len_Mora=18".to_string(),
                            yes: TreeIndex::Pdf(2),
                            no: TreeIndex::Pdf(3),
                        }
                    ]
                }
            ))
        );
        assert_eq!(
            TreeParser::parse_tree::<nom::error::Error<&str>>(r#"{*}[2] "gv_lf0_3""#),
            Ok((
                "",
                Tree {
                    state: 2,
                    nodes: vec![Node {
                        id: 0,
                        question_name: "".to_string(),
                        yes: TreeIndex::Pdf(3),
                        no: TreeIndex::Pdf(3),
                    },]
                }
            ))
        );
    }

    #[test]
    fn parse_trees() {
        let tree = r#"
{*}[2]
{
0 Utt_Len_Mora<=28                                    "gv_lf0_1"          -1      
-1 Utt_Len_Mora=18                                     "gv_lf0_3"       "gv_lf0_2" 
}"#;
        TreeParser::parse_trees::<nom::error::Error<&str>>(tree).unwrap();
        TreeParser::parse_trees::<nom::error::Error<&str>>(&format!("{tree}  \n")).unwrap();
        TreeParser::parse_trees::<nom::error::Error<&str>>(&format!("{tree}  \n{tree}")).unwrap();
    }
}
