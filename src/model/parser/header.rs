use std::{collections::HashMap, marker::PhantomData, str::FromStr};

use nom::{
    bytes::complete::tag,
    character::complete::line_ending,
    combinator::{cut, map, opt},
    error::{context, ContextError, ErrorKind, ParseError},
    multi::{fold_many0, many0, many1},
    sequence::{delimited, pair, preceded, separated_pair, terminated},
    IResult,
};

use crate::model::{
    parser::header_entry::HeaderEntry, stream::StreamModelMetadata, GlobalModelMetadata,
};

use super::base::ParseTarget;

#[derive(Debug)]
enum ParseApplyError {
    MainKey,
    SubKey,
    Value,
}

trait ApplyParsed {
    fn apply(&mut self, entry: HeaderEntry<String>) -> Result<(), ParseApplyError>;
    fn parse_value<T: FromStr>(v: &str) -> Result<T, ParseApplyError> {
        v.parse().map_err(|_| ParseApplyError::Value)
    }
}

impl ApplyParsed for GlobalModelMetadata {
    fn apply<'a>(&mut self, entry: HeaderEntry<String>) -> Result<(), ParseApplyError> {
        if entry.key_sub().is_some() {
            return Err(ParseApplyError::SubKey);
        }
        match entry.key_main() {
            "HTS_VOICE_VERSION" => self.hts_voice_version = entry.value().to_string(),
            "SAMPLING_FREQUENCY" => self.sampling_frequency = Self::parse_value(entry.value())?,
            "FRAME_PERIOD" => self.frame_period = Self::parse_value(entry.value())?,
            "NUM_VOICES" => self.num_voices = Self::parse_value(entry.value())?,
            "NUM_STATES" => self.num_states = Self::parse_value(entry.value())?,
            "NUM_STREAMS" => self.num_streams = Self::parse_value(entry.value())?,
            "STREAM_TYPE" => {
                self.stream_type = entry.value().split(',').map(|s| s.to_string()).collect()
            }
            "FULLCONTEXT_FORMAT" => self.fullcontext_format = entry.value().to_string(),
            "FULLCONTEXT_VERSION" => self.fullcontext_version = entry.value().to_string(),
            "GV_OFF_CONTEXT" => {
                self.gv_off_context = ParseTarget::parse_pattern_list::<()>(entry.value())
                    .or(Err(ParseApplyError::Value))?
                    .1
            }
            "COMMENT" => (),
            _ => Err(ParseApplyError::MainKey)?,
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct Stream {
    pub stream: HashMap<String, StreamModelMetadata>,
}

impl ApplyParsed for Stream {
    fn apply<'a>(&mut self, header_entry: HeaderEntry<String>) -> Result<(), ParseApplyError> {
        let Some(subkey) = header_entry.key_sub() else {
            return Err(ParseApplyError::SubKey);
        };
        let entry = self.stream.entry(subkey.to_string()).or_default();
        match header_entry.key_main() {
            "VECTOR_LENGTH" => entry.vector_length = Self::parse_value(header_entry.value())?,
            "NUM_WINDOWS" => entry.num_windows = Self::parse_value(header_entry.value())?,
            "IS_MSD" => entry.is_msd = header_entry.value() == "1",
            "USE_GV" => entry.use_gv = header_entry.value() == "1",
            "OPTION" => {
                entry.option = header_entry
                    .value()
                    .split(',')
                    .map(|s| s.to_string())
                    .collect()
            }
            _ => Err(ParseApplyError::MainKey)?,
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct Position {
    pub duration_pdf: (usize, usize),
    pub duration_tree: (usize, usize),
    pub position: HashMap<String, PositionData>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PositionData {
    pub stream_win: Vec<(usize, usize)>,
    pub stream_pdf: (usize, usize),
    pub stream_tree: (usize, usize),
    pub gv_pdf: (usize, usize),
    pub gv_tree: (usize, usize),
}

impl ApplyParsed for Position {
    fn apply<'a>(&mut self, header_entry: HeaderEntry<String>) -> Result<(), ParseApplyError> {
        fn parse_duration(v: &str) -> Result<(usize, usize), ParseApplyError> {
            match v.split_once('-') {
                Some((s, e)) => Ok((Position::parse_value(s)?, Position::parse_value(e)?)),
                _ => Err(ParseApplyError::Value),
            }
        }
        if let Some(subkey) = header_entry.key_sub() {
            let entry = self.position.entry(subkey.to_string()).or_default();
            match header_entry.key_main() {
                "STREAM_WIN" => {
                    entry.stream_win = header_entry
                        .value()
                        .split(',')
                        .map(parse_duration)
                        .collect::<Result<_, _>>()?
                }
                "STREAM_PDF" => entry.stream_pdf = parse_duration(header_entry.value())?,
                "STREAM_TREE" => entry.stream_tree = parse_duration(header_entry.value())?,
                "GV_PDF" => entry.gv_pdf = parse_duration(header_entry.value())?,
                "GV_TREE" => entry.gv_tree = parse_duration(header_entry.value())?,
                _ => Err(ParseApplyError::MainKey)?,
            }
        } else {
            match header_entry.key_main() {
                "DURATION_PDF" => self.duration_pdf = parse_duration(header_entry.value())?,
                "DURATION_TREE" => self.duration_tree = parse_duration(header_entry.value())?,
                _ => Err(ParseApplyError::MainKey)?,
            }
        }
        Ok(())
    }
}

pub struct HeaderParser<T>(PhantomData<T>);

impl<S: ParseTarget> HeaderParser<S>
where
    <S as nom::InputIter>::Item: nom::AsChar,
    <S as nom::InputTakeAtPosition>::Item: nom::AsChar,
{
    fn parse_entry<E: ParseError<S> + ContextError<S>>(i: S) -> IResult<S, HeaderEntry<S>, E> {
        use nom::character::complete::char;

        context(
            "entry",
            map(
                separated_pair(
                    pair(
                        S::parse_identifier,
                        opt(delimited(char('['), S::parse_identifier, char(']'))),
                    ),
                    char(':'),
                    S::parse_ascii,
                ),
                HeaderEntry::new,
            ),
        )(i)
    }

    fn parse_general<E: ParseError<S> + ContextError<S>, T: Default + ApplyParsed>(
        i: S,
    ) -> IResult<S, T, E> {
        fold_many0(
            terminated(Self::parse_entry, many1(line_ending)),
            || Ok(T::default()),
            |acc, r| {
                let Ok(mut acc) = acc else {
                    return acc;
                };

                let (_, entry) = r.parse_to_string()?;

                match acc.apply(entry) {
                    Ok(()) => Ok(acc),
                    Err(ParseApplyError::MainKey) => Err(nom::Err::Failure(E::from_error_kind(
                        r.into_key_main(),
                        ErrorKind::Tag,
                    ))),
                    Err(ParseApplyError::SubKey) => Err(nom::Err::Failure(E::from_error_kind(
                        r.into_key_sub_or_main(),
                        ErrorKind::NonEmpty,
                    ))),
                    Err(ParseApplyError::Value) => Err(nom::Err::Failure(E::from_error_kind(
                        r.into_value(),
                        ErrorKind::Verify,
                    ))),
                }
            },
        )(i)
        .and_then(|r| match r {
            (rest, Ok(result)) => Ok((rest, result)),
            (_, Err(err)) => Err(err),
        })
    }

    pub fn parse_global<E: ParseError<S> + ContextError<S>>(
        i: S,
    ) -> IResult<S, GlobalModelMetadata, E> {
        context(
            "global",
            preceded(
                preceded(many0(line_ending), tag("[GLOBAL]\n")),
                cut(Self::parse_general),
            ),
        )(i)
    }

    pub fn parse_stream<E: ParseError<S> + ContextError<S>>(i: S) -> IResult<S, Stream, E> {
        context(
            "stream",
            preceded(
                preceded(many0(line_ending), tag("[STREAM]\n")),
                cut(Self::parse_general),
            ),
        )(i)
    }

    pub fn parse_position<E: ParseError<S> + ContextError<S>>(i: S) -> IResult<S, Position, E> {
        context(
            "position",
            preceded(
                preceded(many0(line_ending), tag("[POSITION]\n")),
                cut(Self::parse_general),
            ),
        )(i)
    }
}

#[cfg(test)]
mod tests {
    use nom::error::VerboseError;

    use crate::model::{parser::header::HeaderEntry, stream::Pattern};

    use super::{HeaderParser, PositionData, StreamModelMetadata};

    #[test]
    fn entry() {
        assert_eq!(
            HeaderParser::parse_entry::<VerboseError<&str>>(
                "GV_PDF[MCP]:1167198-1167761\nGV_PDF[LF0]:1167762-1167789"
            ),
            Ok((
                "\nGV_PDF[LF0]:1167762-1167789",
                HeaderEntry::new((("GV_PDF", Some("MCP")), "1167198-1167761"))
            ))
        );
        assert_eq!(
            HeaderParser::parse_entry::<VerboseError<&str>>("GV_PDF[LF0]:1167762-1167789"),
            Ok((
                "",
                HeaderEntry::new((("GV_PDF", Some("LF0")), "1167762-1167789"))
            ))
        );
        assert_eq!(
            HeaderParser::parse_entry::<VerboseError<&str>>(
                "GV_OFF_CONTEXT:\"*-sil+*\",\"*-pau+*\""
            ),
            Ok((
                "",
                HeaderEntry::new((("GV_OFF_CONTEXT", None), "\"*-sil+*\",\"*-pau+*\""))
            ))
        );
        assert_eq!(
            HeaderParser::parse_entry::<VerboseError<&str>>("COMMENT:"),
            Ok(("", HeaderEntry::new((("COMMENT", None), ""))))
        );
    }
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
";
    #[test]
    fn global() {
        let (rest, global) = HeaderParser::parse_global::<VerboseError<&str>>(CONTENT).unwrap();
        assert_eq!(rest.len(), 751);
        assert_eq!(global.hts_voice_version, "1.0");
        assert_eq!(
            global.gv_off_context,
            vec![
                Pattern::from_pattern_string("*-sil+*").unwrap(),
                Pattern::from_pattern_string("*-pau+*").unwrap(),
            ]
        );
    }
    #[test]
    fn stream() {
        let (rest, stream) =
            HeaderParser::parse_stream::<VerboseError<&str>>(&CONTENT[224..]).unwrap();
        assert_eq!(rest.len(), 487);
        assert_eq!(
            stream.stream.get("MCP"),
            Some(&StreamModelMetadata {
                vector_length: 35,
                num_windows: 3,
                is_msd: false,
                use_gv: true,
                option: vec!["ALPHA=0.55".to_string()],
            })
        );
    }
    #[test]
    fn position() {
        let (rest, position) =
            HeaderParser::parse_position::<VerboseError<&str>>(&CONTENT[488..]).unwrap();
        assert_eq!(rest.len(), 0);
        assert_eq!(position.duration_pdf, (0, 9803));
        assert_eq!(
            position.position.get("MCP"),
            Some(&PositionData {
                stream_win: vec![(40880, 40885), (40886, 40900), (40901, 40915)],
                stream_pdf: (40958, 788577),
                stream_tree: (850114, 940979),
                gv_pdf: (1167198, 1167761),
                gv_tree: (1167790, 1167967),
            })
        );
    }

    #[test]
    fn global_bin() {
        let (rest, global) =
            HeaderParser::parse_global::<VerboseError<&[u8]>>(CONTENT.as_bytes()).unwrap();
        assert_eq!(rest.len(), 751);
        assert_eq!(global.hts_voice_version, "1.0");
        assert_eq!(
            global.gv_off_context,
            vec![
                Pattern::from_pattern_string("*-sil+*").unwrap(),
                Pattern::from_pattern_string("*-pau+*").unwrap(),
            ]
        );
    }
}
