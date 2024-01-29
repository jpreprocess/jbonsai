use std::collections::BTreeMap;

use nom::{
    error::{ContextError, ParseError, VerboseError},
    IResult, Parser,
};

use crate::model::parser::{base::ParseTarget, header::parse_header};

use self::{
    convert::convert_tree,
    header::{error::DeserializeError, Global, Position, Stream, StreamData},
    tree::{Question, TreeParser},
    window::WindowParser,
};

use super::{
    stream::{Model, Pattern, StreamModels},
    GlobalModelMetadata, Voice,
};

mod base;
mod header;
mod tree;
mod window;

mod convert;

#[derive(Debug, thiserror::Error)]
pub enum ModelParseError {
    #[error("Nom parser returned error:{0}")]
    NomError(String),
    #[error("Failed to parse Header as UTF-8")]
    HeaderUtf8Error,
    #[error("Failed to parse header:{0}")]
    DeserializeError(#[from] DeserializeError),
    #[error("Failed to parse pattern")]
    PatternParseError,

    #[error("Stream was not found")]
    StreamNotFound,
    #[error("Position was not found")]
    PositionNotFound,

    #[error("USE_GV is true, but positions for GV is not set")]
    UseGvError,
}

impl<'a> From<nom::Err<VerboseError<&'a [u8]>>> for ModelParseError {
    fn from(value: nom::Err<VerboseError<&'a [u8]>>) -> Self {
        match value {
            nom::Err::Error(e) | nom::Err::Failure(e) => {
                let message = e
                    .errors
                    .iter()
                    .fold(String::new(), |acc: String, (src, kind)| {
                        let input = std::string::String::from_utf8_lossy(&src[..src.len().min(20)]);
                        match kind {
                            nom::error::VerboseErrorKind::Nom(e) => {
                                format!("{}\n{:?} at: {}", acc, e, input)
                            }
                            nom::error::VerboseErrorKind::Char(c) => {
                                format!("{}\nexpected '{}' at: {}", acc, c, input)
                            }
                            nom::error::VerboseErrorKind::Context(s) => {
                                format!("{}\nin section '{}', at: {}", acc, s, input)
                            }
                        }
                    });
                Self::NomError(message)
            }
            nom::Err::Incomplete(_) => Self::NomError("Not enough data".to_string()),
        }
    }
}

pub fn parse_htsvoice(
    input: &[u8],
) -> Result<(GlobalModelMetadata, Voice), ModelParseError> {
    let (_, (in_global, in_stream, in_position, in_data)) = split_sections(input)?;

    let global: Global = parse_header(&in_global)?;
    let stream: Stream = parse_header(&in_stream)?;
    let position: Position = parse_header(&in_position)?;

    let (duration_model, stream_models) = parse_data_section(in_data, &global, &stream, &position)?;

    // TODO: verify

    let voice = Voice {
        duration_model,
        stream_models,
    };

    Ok((global.try_into()?, voice))
}

pub fn split_sections<'a, S, E>(input: S) -> IResult<S, (S, S, S, S), E>
where
    S: ParseTarget,
    E: ParseError<S> + ContextError<S>,
    <S as nom::InputIter>::Item: nom::AsChar + Clone + Copy,
    <S as nom::InputTakeAtPosition>::Item: nom::AsChar + Clone,
    for<'b> &'b str: nom::FindToken<<S as nom::InputIter>::Item>,
{
    use nom::{
        bytes::complete::{tag, take_until},
        character::complete::newline,
        combinator::{all_consuming, rest},
        error::context,
        multi::{many0, many1},
        sequence::{pair, preceded, tuple},
    };

    context(
        "htsvoice_split",
        all_consuming(tuple((
            preceded(pair(many0(newline), tag("[GLOBAL]\n")), take_until("\n[")),
            preceded(pair(many1(newline), tag("[STREAM]\n")), take_until("\n[")),
            preceded(pair(many1(newline), tag("[POSITION]\n")), take_until("\n[")),
            preceded(pair(many1(newline), tag("[DATA]\n")), rest),
        ))),
    )(input)
}

fn parse_data_section(
    input: &[u8],
    global: &Global,
    stream: &Stream,
    position: &Position,
) -> Result<(Model, Vec<StreamModels>), ModelParseError> {
    use nom::{combinator::all_consuming, sequence::terminated};

    let duration_model = parse_model(
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

    let stream_models: Vec<StreamModels> = global
        .stream_type
        .iter()
        .map(|key| {
            let pos = position
                .position
                .get(key)
                .ok_or(ModelParseError::PositionNotFound)?;
            let stream_data = stream
                .stream
                .get(key)
                .ok_or(ModelParseError::StreamNotFound)?;

            let stream_model = parse_model(input, pos.stream_tree, pos.stream_pdf, stream_data)?;

            let gv_model = if stream_data.use_gv {
                let gv_model = parse_model(
                    input,
                    pos.gv_tree.ok_or(ModelParseError::UseGvError)?,
                    pos.gv_pdf.ok_or(ModelParseError::UseGvError)?,
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
                        ))(&input[win.0..=win.1])?
                        .1)
                    })
                    .collect::<Result<_, ModelParseError>>()?;

            Ok(StreamModels::new(
                stream_data.clone().into(),
                stream_model,
                gv_model,
                windows,
            ))
        })
        .collect::<Result<_, ModelParseError>>()?;

    Ok((duration_model, stream_models))
}

pub fn parse_model(
    input: &[u8],
    tree_range: (usize, usize),
    pdf_range: (usize, usize),
    stream_data: &StreamData,
) -> Result<Model, ModelParseError> {
    use nom::{
        combinator::map,
        multi::many_m_n,
        number::complete::{le_f32, le_u32},
        sequence::{pair, terminated},
    };

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
                        crate::model::stream::ModelParameter::from_linear,
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

    Ok(Model::new(new_trees, pdf))
}

fn parse_all<'a, T, F, E>(
    f: F,
    range: (usize, usize),
) -> impl FnOnce(&'a [u8]) -> IResult<&'a [u8], T, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
    F: Parser<&'a [u8], T, E>,
{
    use nom::combinator::all_consuming;

    move |input: &'a [u8]| all_consuming(f)(&input[range.0..range.1 + 1])
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::{model::parser::split_sections, tests::MODEL_NITECH_ATR503};

    use super::parse_htsvoice;

    #[test]
    fn load() {
        let model = fs::read(MODEL_NITECH_ATR503).unwrap();
        parse_htsvoice(&model).unwrap();
    }

    #[test]
    fn split() {
        const CONTENT: &str = "
[GLOBAL]
HTS_VOICE_VERSION:1.0
SAMPLING_FREQUENCY:48000
FRAME_PERIOD:240
NUM_STATES:5
NUM_STREAMS:3
STREAM_TYPE:MCP,LF0,LPF
FULLCONTEXT_FORMAT:HTS_TTS_JPN
FULLCONTEXT_VERSION:1.0
GV_OFF_CONTEXT:\"*-sil+*\",\"*-pau+*\"
COMMENT:
[STREAM]
VECTOR_LENGTH[MCP]:35
VECTOR_LENGTH[LF0]:1
VECTOR_LENGTH[LPF]:31
IS_MSD[MCP]:0
IS_MSD[LF0]:1
IS_MSD[LPF]:0
NUM_WINDOWS[MCP]:3
NUM_WINDOWS[LF0]:3
NUM_WINDOWS[LPF]:1
USE_GV[MCP]:1
USE_GV[LF0]:1
USE_GV[LPF]:0
OPTION[MCP]:ALPHA=0.55
OPTION[LF0]:
OPTION[LPF]:
[POSITION]
DURATION_PDF:0-9803
DURATION_TREE:9804-40879
STREAM_WIN[MCP]:40880-40885,40886-40900,40901-40915
STREAM_WIN[LF0]:40916-40921,40922-40936,40937-40951
STREAM_WIN[LPF]:40952-40957
STREAM_PDF[MCP]:40958-788577
STREAM_PDF[LF0]:788578-848853
STREAM_PDF[LPF]:848854-850113
STREAM_TREE[MCP]:850114-940979
STREAM_TREE[LF0]:940980-1167092
STREAM_TREE[LPF]:1167093-1167197
GV_PDF[MCP]:1167198-1167761
GV_PDF[LF0]:1167762-1167789
GV_TREE[MCP]:1167790-1167967
GV_TREE[LF0]:1167968-1168282
[DATA]
";
        split_sections::<&str, nom::error::VerboseError<&str>>(CONTENT).unwrap();
    }
}
