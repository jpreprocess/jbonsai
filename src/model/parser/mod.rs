use std::collections::BTreeMap;

use nom::{
    bytes::complete::tag,
    combinator::{all_consuming, map},
    error::{ContextError, ParseError},
    multi::many_m_n,
    number::complete::{le_f32, le_u32},
    sequence::{pair, preceded, terminated},
    IResult, Parser,
};

use crate::model::parser::base::ParseTarget;

use self::{
    convert::convert_tree,
    header::{HeaderParser, Position, Stream},
    tree::{Question, TreeParser},
    window::WindowParser,
};

use super::{
    model::{Model, Pattern, StreamModelMetadata, StreamModels},
    GlobalModelMetadata, Voice,
};

mod base;
mod header;
mod tree;
mod window;

mod convert;

pub fn parse_htsvoice<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], (GlobalModelMetadata, Voice), E> {
    let (input, global) = HeaderParser::parse_global(input)?;
    let (input, stream) = HeaderParser::parse_stream(input)?;
    let (input, position) = HeaderParser::parse_position(input)?;

    // TODO: verify

    let (input, (duration_model, stream_models)) = preceded(tag("[DATA]\n"), |i| {
        parse_data_section(i, &global, &stream, &position)
    })(input)?;

    let voice = Voice {
        duration_model,
        stream_models,
    };

    Ok((input, (global, voice)))
}

fn parse_data_section<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
    input: &'a [u8],
    global: &GlobalModelMetadata,
    stream: &Stream,
    position: &Position,
) -> IResult<&'a [u8], (Model, Vec<StreamModels>), E> {
    let (_, duration_model) = parse_model(
        input,
        position.duration_tree,
        position.duration_pdf,
        &StreamModelMetadata {
            vector_length: global.num_states,
            num_windows: 1,
            is_msd: false,
            use_gv: false,
            option: vec![],
        },
    )?;
    let stream_models: Vec<StreamModels> = global
        .stream_type
        .iter()
        .map(|key| {
            let pos = position.position.get(key).unwrap();
            let stream_data = stream.stream.get(key).unwrap();

            let (_, stream_model) =
                parse_model(input, pos.stream_tree, pos.stream_pdf, stream_data)?;

            let gv_model = if stream_data.use_gv {
                let (_, gv_model) = parse_model(
                    input,
                    pos.gv_tree,
                    pos.gv_pdf,
                    &StreamModelMetadata {
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

            Ok(StreamModels::new(
                stream_data.clone(),
                stream_model,
                gv_model,
                windows,
            ))
        })
        .collect::<Result<_, _>>()?;

    Ok((b"", (duration_model, stream_models)))
}

pub fn parse_model<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
    input: &'a [u8],
    tree_range: (usize, usize),
    pdf_range: (usize, usize),
    stream_data: &StreamModelMetadata,
) -> IResult<&'a [u8], Model, E> {
    let pdf_len =
        stream_data.vector_length * stream_data.num_windows * 2 + (stream_data.is_msd as usize);

    let (_, (questions, trees)) = parse_all(
        terminated(
            pair(TreeParser::parse_questions, TreeParser::parse_trees),
            ParseTarget::sp,
        ),
        tree_range,
    )(input)?;

    let question_lut: BTreeMap<&String, &Vec<Pattern>> = BTreeMap::from_iter(
        questions
            .iter()
            .map(|Question { name, patterns }| (name, patterns)),
    );

    let (_, pdf) = parse_all(
        |i| {
            let ntree = trees.len();
            let (mut i, npdf) = many_m_n(ntree, ntree, le_u32)(i)?;
            let mut pdf = Vec::with_capacity(ntree);
            for n in npdf {
                let n = n as usize;
                let (ni, r) = many_m_n(
                    n,
                    n,
                    map(
                        many_m_n(pdf_len, pdf_len, map(le_f32, |v| v as f64)),
                        crate::model::model::ModelParameter::from_linear,
                    ),
                )(i)?;
                pdf.push(r);
                i = ni;
            }
            Ok((i, pdf))
        },
        pdf_range,
    )(input)?;

    let new_trees: Vec<_> = trees
        .into_iter()
        .map(|t| convert_tree(t, &question_lut))
        .collect();

    Ok((b"", Model::new(new_trees, pdf)))
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

    use crate::tests::MODEL_NITECH_ATR503;

    use super::parse_htsvoice;

    #[test]
    fn load() {
        let model = fs::read(MODEL_NITECH_ATR503).unwrap();
        parse_htsvoice::<VerboseError<&[u8]>>(&model).unwrap();
    }
}
