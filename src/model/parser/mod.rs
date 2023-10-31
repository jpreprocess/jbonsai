use nom::{
    bytes::complete::tag,
    combinator::all_consuming,
    error::{ContextError, ParseError},
    multi::many_m_n,
    number::complete::{le_f32, le_u32},
    sequence::{pair, preceded, terminated},
    IResult, Parser,
};

use crate::model::parser::base::ParseTarget;

use self::{
    header::{Global, HeaderParser, Position, Stream, StreamData},
    tree::{Question, Tree, TreeParser},
    window::WindowParser,
};

mod base;
mod header;
mod tree;
mod window;

pub fn parse_htsvoice<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], (), E> {
    let (input, global) = HeaderParser::parse_global(input)?;
    let (input, stream) = HeaderParser::parse_stream(input)?;
    let (input, position) = HeaderParser::parse_position(input)?;

    // TODO: verify

    let (input, _) = preceded(tag("[DATA]\n"), |i| {
        parse_data_section(i, &global, &stream, &position)
    })(input)?;

    Ok((input, ()))
}

fn parse_data_section<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
    input: &'a [u8],
    global: &Global,
    stream: &Stream,
    position: &Position,
) -> IResult<&'a [u8], (), E> {
    let (_, duration_model) = Model::parse(
        input,
        position.duration_tree,
        position.duration_pdf,
        &StreamData {
            vector_length: global.num_states,
            num_windows: 1,
            is_msd: false,
            use_gv: false,
            option: vec![],
        },
    )?;
    let stream_models: Vec<(Model, Option<Model>, Vec<Vec<f32>>)> = global
        .stream_type
        .iter()
        .map(|key| {
            let pos = position.position.get(key).unwrap();
            let stream_data = stream.stream.get(key).unwrap();

            let (_, stream_model) =
                Model::parse(input, pos.stream_tree, pos.stream_pdf, stream_data)?;

            let gv_model = if stream_data.use_gv {
                let (_, gv_model) = Model::parse(
                    input,
                    pos.gv_tree,
                    pos.gv_pdf,
                    &StreamData {
                        vector_length: stream_data.vector_length,
                        num_windows: 1,
                        is_msd: false,
                        use_gv: true,
                        option: vec![],
                    },
                )?;
                Some(gv_model)
            } else {
                None
            };

            let windows =
                pos.stream_win
                    .iter()
                    .map(|win| {
                        Ok(all_consuming(terminated(
                            WindowParser::parse_window_row,
                            ParseTarget::sp,
                        ))(&input[win.0..win.1 + 1])?
                        .1)
                    })
                    .collect::<Result<_, _>>()?;
            dbg!(&windows);

            Ok((stream_model, gv_model, windows))
        })
        .collect::<Result<_, _>>()?;

    Ok((b"", ()))
}

#[derive(Debug, Clone)]
pub struct Model {
    questions: Vec<Question>,
    trees: Vec<Tree>,
    pdf: Vec<Vec<Vec<f32>>>,
}

impl Model {
    pub fn parse<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
        tree_range: (usize, usize),
        pdf_range: (usize, usize),
        stream_data: &StreamData,
    ) -> IResult<&'a [u8], Self, E> {
        let pdf_len =
            stream_data.vector_length * stream_data.num_windows * 2 + (stream_data.is_msd as usize);

        let (_, (questions, trees)) = parse_all(
            terminated(
                pair(TreeParser::parse_questions, TreeParser::parse_trees),
                ParseTarget::sp,
            ),
            tree_range,
        )(input)?;

        let (_, pdf) = parse_all(
            |i| {
                let ntree = trees.len();
                let (mut i, npdf) = many_m_n(ntree, ntree, le_u32)(i)?;
                let mut pdf = Vec::with_capacity(ntree);
                for n in npdf {
                    let n = n as usize;
                    let (ni, r) = many_m_n(n, n, many_m_n(pdf_len, pdf_len, le_f32))(i)?;
                    pdf.push(r);
                    i = ni;
                }
                Ok((i, pdf))
            },
            pdf_range,
        )(input)?;

        Ok((
            b"",
            Self {
                questions,
                trees,
                pdf,
            },
        ))
    }
}

fn parse_all<'a, T, F, E>(
    f: F,
    range: (usize, usize),
) -> impl FnOnce(&'a [u8]) -> IResult<&'a [u8], T, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
    F: Parser<&'a [u8], T, E>,
{
    move |input: &'a [u8]| all_consuming(f)(&input[range.0..range.1 + 1])
}

#[cfg(test)]
mod tests {
    use std::fs;

    use nom::error::VerboseError;

    use super::parse_htsvoice;

    #[test]
    fn load() {
        let model = fs::read("models/nitech_jp_atr503_m001.htsvoice").unwrap();
        parse_htsvoice::<VerboseError<&[u8]>>(&model).unwrap();
    }
}
