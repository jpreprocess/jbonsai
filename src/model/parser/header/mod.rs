mod de;
mod deserialize_hashmap;
pub mod error;

pub use de::from_str;

use std::collections::HashMap;

use serde::Deserialize;

use nom::{
    bytes::complete::{tag, take_until},
    character::complete::newline,
    error::{context, ContextError, ParseError},
    multi::{many0, many1},
    sequence::{pair, preceded, tuple},
    IResult,
};

use super::base::ParseTarget;

pub fn parse_header<'a, S, E>(input: S) -> IResult<S, (Global, Stream, Position), E>
where
    S: ParseTarget,
    E: ParseError<S> + ContextError<S>,
    <S as nom::InputIter>::Item: nom::AsChar + Clone + Copy,
    <S as nom::InputTakeAtPosition>::Item: nom::AsChar + Clone,
    for<'b> &'b str: nom::FindToken<<S as nom::InputIter>::Item>,
{
    let (in_data, (in_global, in_stream, in_position)) = context(
        "header",
        tuple((
            preceded(pair(many0(newline), tag("[GLOBAL]\n")), take_until("\n[")),
            preceded(pair(many1(newline), tag("[STREAM]\n")), take_until("\n[")),
            preceded(pair(many1(newline), tag("[POSITION]\n")), take_until("\n[")),
        )),
    )(input)?;

    let global: Global = from_str(in_global.parse_utf8().unwrap()).unwrap();
    let stream: Stream = from_str(in_stream.parse_utf8().unwrap()).unwrap();
    let position: Position = from_str(in_position.parse_utf8().unwrap()).unwrap();

    Ok((in_data, (global, stream, position)))
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Global {
    pub hts_voice_version: String,
    pub sampling_frequency: usize,
    pub frame_period: usize,
    pub num_states: usize,
    pub num_streams: usize,
    pub stream_type: Vec<String>,
    pub fullcontext_format: String,
    pub fullcontext_version: String,
    pub gv_off_context: Vec<String>,
    pub comment: String,
}

impl TryFrom<Global> for crate::model::GlobalModelMetadata {
    type Error = regex::Error;
    fn try_from(value: Global) -> Result<Self, Self::Error> {
        use crate::model::stream::Pattern;
        Ok(Self {
            hts_voice_version: value.hts_voice_version,
            sampling_frequency: value.sampling_frequency,
            frame_period: value.frame_period,
            num_voices: 1,
            num_states: value.num_states,
            num_streams: value.num_streams,
            stream_type: value.stream_type,
            fullcontext_format: value.fullcontext_format,
            fullcontext_version: value.fullcontext_version,
            gv_off_context: value
                .gv_off_context
                .into_iter()
                .map(Pattern::from_pattern_string)
                .collect::<Result<_, _>>()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Stream {
    #[serde(flatten, with = "deserialize_hashmap")]
    pub stream: HashMap<String, StreamData>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct StreamData {
    pub vector_length: usize,
    pub num_windows: usize,
    pub is_msd: bool,
    pub use_gv: bool,
    pub option: Vec<String>,
}

impl From<StreamData> for crate::model::stream::StreamModelMetadata {
    fn from(value: StreamData) -> Self {
        Self {
            vector_length: value.vector_length,
            num_windows: value.num_windows,
            is_msd: value.is_msd,
            use_gv: value.use_gv,
            option: value.option,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Position {
    pub duration_pdf: (usize, usize),
    pub duration_tree: (usize, usize),
    #[serde(flatten, with = "deserialize_hashmap")]
    pub position: HashMap<String, PositionData>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct PositionData {
    pub stream_win: Vec<(usize, usize)>,
    pub stream_pdf: (usize, usize),
    pub stream_tree: (usize, usize),
    pub gv_pdf: Option<(usize, usize)>,
    pub gv_tree: Option<(usize, usize)>,
}

#[cfg(test)]
mod tests {
    use super::{
        de::from_str, deserialize_hashmap, parse_header, Global, Position, PositionData, Stream,
        StreamData,
    };

    use std::collections::HashMap;

    #[test]
    fn serde_parser() {
        use serde::Deserialize;
        #[derive(Deserialize, PartialEq, Debug)]
        #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
        struct Test {
            fullcontext_version: String,
            gv_off_context: Vec<String>,
            sampling_frequency: usize,
            stream_win: Vec<(usize, usize)>,
            #[serde(flatten, with = "deserialize_hashmap")]
            test: HashMap<String, TestInner>,
        }
        #[derive(Deserialize, PartialEq, Debug, Clone)]
        #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
        struct TestInner {
            stream_pdf: (usize, usize),
        }

        let j = r#"
FULLCONTEXT_VERSION:1.0
GV_OFF_CONTEXT:"*-sil+*","*-pau+*"
SAMPLING_FREQUENCY:48000
STREAM_WIN:40880-40885,40886-40900
STREAM_PDF[LF0]:788578-848853
"#;
        let expected = Test {
            fullcontext_version: "1.0".to_string(),
            gv_off_context: vec!["*-sil+*".to_owned(), "*-pau+*".to_owned()],
            sampling_frequency: 48000,
            stream_win: vec![(40880, 40885), (40886, 40900)],
            test: HashMap::from([(
                "LF0".to_string(),
                TestInner {
                    stream_pdf: (788578, 848853),
                },
            )]),
        };
        assert_eq!(expected, from_str(j).unwrap());
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
        parse_header::<&str, nom::error::VerboseError<&str>>(CONTENT).unwrap();
    }

    #[test]
    fn global() {
        const GLOBAL: &str = r#"
HTS_VOICE_VERSION:1.0
SAMPLING_FREQUENCY:48000
FRAME_PERIOD:240
NUM_STATES:5
NUM_STREAMS:3
STREAM_TYPE:MCP,LF0,LPF
FULLCONTEXT_FORMAT:HTS_TTS_JPN
FULLCONTEXT_VERSION:1.0
GV_OFF_CONTEXT:"*-sil+*","*-pau+*"
COMMENT:
"#;
        assert_eq!(
            from_str::<Global>(GLOBAL).unwrap(),
            Global {
                hts_voice_version: "1.0".to_string(),
                sampling_frequency: 48000,
                frame_period: 240,
                num_states: 5,
                num_streams: 3,
                stream_type: vec!["MCP".to_string(), "LF0".to_string(), "LPF".to_string()],
                fullcontext_format: "HTS_TTS_JPN".to_string(),
                fullcontext_version: "1.0".to_string(),
                gv_off_context: vec!["*-sil+*".to_string(), "*-pau+*".to_string()],
                comment: "".to_string(),
            }
        );
    }

    #[test]
    fn stream() {
        const STREAM: &str = r#"
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
"#;
        assert_eq!(
            from_str::<Stream>(STREAM).unwrap(),
            Stream {
                stream: HashMap::from([
                    (
                        "MCP".to_string(),
                        StreamData {
                            vector_length: 35,
                            is_msd: false,
                            num_windows: 3,
                            use_gv: true,
                            option: vec!["ALPHA=0.55".to_string()],
                        },
                    ),
                    (
                        "LF0".to_string(),
                        StreamData {
                            vector_length: 1,
                            is_msd: true,
                            num_windows: 3,
                            use_gv: true,
                            option: vec![],
                        },
                    ),
                    (
                        "LPF".to_string(),
                        StreamData {
                            vector_length: 31,
                            is_msd: false,
                            num_windows: 1,
                            use_gv: false,
                            option: vec![],
                        },
                    )
                ])
            }
        );
    }

    #[test]
    fn position() {
        const POSITION: &str = r#"
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
"#;
        assert_eq!(
            from_str::<Position>(POSITION).unwrap(),
            Position {
                duration_pdf: (0, 9803),
                duration_tree: (9804, 40879),
                position: HashMap::from([
                    (
                        "MCP".to_string(),
                        PositionData {
                            stream_win: vec![(40880, 40885), (40886, 40900), (40901, 40915)],
                            stream_pdf: (40958, 788577),
                            stream_tree: (850114, 940979),
                            gv_pdf: Some((1167198, 1167761)),
                            gv_tree: Some((1167790, 1167967)),
                        },
                    ),
                    (
                        "LF0".to_string(),
                        PositionData {
                            stream_win: vec![(40916, 40921), (40922, 40936), (40937, 40951)],
                            stream_pdf: (788578, 848853),
                            stream_tree: (940980, 1167092),
                            gv_pdf: Some((1167762, 1167789)),
                            gv_tree: Some((1167968, 1168282)),
                        },
                    ),
                    (
                        "LPF".to_string(),
                        PositionData {
                            stream_win: vec![(40952, 40957)],
                            stream_pdf: (848854, 850113),
                            stream_tree: (1167093, 1167197),
                            gv_pdf: None,
                            gv_tree: None,
                        },
                    )
                ])
            }
        );
    }
}
