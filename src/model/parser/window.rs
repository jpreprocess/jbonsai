use std::marker::PhantomData;

use nom::{
    character::complete::{digit1, space1},
    error::{ContextError, ErrorKind, ParseError},
    multi::many_m_n,
    number::complete::double,
    sequence::preceded,
    IResult,
};

use super::base::ParseTarget;

pub struct WindowParser<T>(PhantomData<T>);

impl<S: ParseTarget> WindowParser<S>
where
    <S as nom::InputIter>::Item: nom::AsChar + Clone + Copy,
    <S as nom::InputIter>::IterElem: Clone,
    <S as nom::InputTakeAtPosition>::Item: nom::AsChar + Clone,
{
    pub fn parse_window_row<E: ParseError<S> + ContextError<S>>(i: S) -> IResult<S, Vec<f64>, E> {
        let (i, n) = digit1(i)?;
        let Some(n) = n.parse_to() else {
            return Err(nom::Err::Error(E::from_error_kind(n, ErrorKind::Float)));
        };
        many_m_n(n, n, preceded(space1, double))(i)
    }
}

#[cfg(test)]
mod tests {
    use nom::error::VerboseError;

    use super::WindowParser;

    #[test]
    fn parse_window_row() {
        assert_eq!(
            WindowParser::parse_window_row::<VerboseError<&str>>("3 -0.5 0.0 0.5"),
            Ok(("", vec![-0.5, 0.0, 0.5]))
        );
    }
}
