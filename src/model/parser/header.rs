use std::{collections::HashMap, marker::PhantomData, str::FromStr};

use nom::{
    bytes::complete::tag,
    character::complete::line_ending,
    combinator::{cut, opt},
    error::{context, ContextError, ErrorKind, ParseError},
    multi::{fold_many0, many0, many1},
    sequence::{delimited, pair, preceded, separated_pair, terminated},
    IResult,
};

use crate::model::{model::StreamModelMetadata, GlobalModelManifest};

use super::base::ParseTarget;

#[derive(Debug)]
enum ParseApplyError {
    MainKey,
    SubKey,
    Value,
}

trait ApplyParsed {
    fn apply<'a>(
        &mut self,
        k: (&'a str, Option<&'a str>),
        v: &'a str,
    ) -> Result<(), ParseApplyError>;
    fn parse_value<'a, T: FromStr>(v: &'a str) -> Result<T, ParseApplyError> {
        v.parse().map_err(|_| ParseApplyError::Value)
    }
}

impl ApplyParsed for GlobalModelManifest {
    fn apply<'a>(
        &mut self,
        k: (&'a str, Option<&'a str>),
        v: &'a str,
    ) -> Result<(), ParseApplyError> {
        if k.1.is_some() {
            return Err(ParseApplyError::SubKey);
        }
        match k.0 {
            "HTS_VOICE_VERSION" => self.hts_voice_version = v.to_string(),
            "SAMPLING_FREQUENCY" => self.sampling_frequency = Self::parse_value(v)?,
            "FRAME_PERIOD" => self.frame_period = Self::parse_value(v)?,
            "NUM_VOICES" => self.num_voices = Self::parse_value(v)?,
            "NUM_STATES" => self.num_states = Self::parse_value(v)?,
            "NUM_STREAMS" => self.num_streams = Self::parse_value(v)?,
            "STREAM_TYPE" => self.stream_type = v.split(',').map(|s| s.to_string()).collect(),
            "FULLCONTEXT_FORMAT" => self.fullcontext_format = v.to_string(),
            "FULLCONTEXT_VERSION" => self.fullcontext_version = v.to_string(),
            "GV_OFF_CONTEXT" => {
                self.gv_off_context = ParseTarget::parse_pattern_list::<()>(v)
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
    fn apply<'a>(
        &mut self,
        k: (&'a str, Option<&'a str>),
        v: &'a str,
    ) -> Result<(), ParseApplyError> {
        let Some(subkey) = k.1 else {
            return Err(ParseApplyError::SubKey);
        };
        let entry = self.stream.entry(subkey.to_string()).or_default();
        match k.0 {
            "VECTOR_LENGTH" => entry.vector_length = Self::parse_value(v)?,
            "NUM_WINDOWS" => entry.num_windows = Self::parse_value(v)?,
            "IS_MSD" => entry.is_msd = v == "1",
            "USE_GV" => entry.use_gv = v == "1",
            "OPTION" => entry.option = v.split(',').map(|s| s.to_string()).collect(),
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
    fn apply<'a>(
        &mut self,
        k: (&'a str, Option<&'a str>),
        v: &'a str,
    ) -> Result<(), ParseApplyError> {
        fn parse_duration<'a>(v: &'a str) -> Result<(usize, usize), ParseApplyError> {
            match v.split_once('-') {
                Some((s, e)) => Ok((Position::parse_value(s)?, Position::parse_value(e)?)),
                _ => Err(ParseApplyError::Value),
            }
        }
        if let Some(subkey) = k.1 {
            let entry = self.position.entry(subkey.to_string()).or_default();
            match k.0 {
                "STREAM_WIN" => {
                    entry.stream_win = v
                        .split(',')
                        .map(|s| parse_duration(s))
                        .collect::<Result<_, _>>()?
                }
                "STREAM_PDF" => entry.stream_pdf = parse_duration(v)?,
                "STREAM_TREE" => entry.stream_tree = parse_duration(v)?,
                "GV_PDF" => entry.gv_pdf = parse_duration(v)?,
                "GV_TREE" => entry.gv_tree = parse_duration(v)?,
                _ => Err(ParseApplyError::MainKey)?,
            }
        } else {
            match k.0 {
                "DURATION_PDF" => self.duration_pdf = parse_duration(v)?,
                "DURATION_TREE" => self.duration_tree = parse_duration(v)?,
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
    fn parse_entry<'a, E: ParseError<S> + ContextError<S>>(
        i: S,
    ) -> IResult<S, ((S, Option<S>), S), E> {
        use nom::character::complete::char;
        context(
            "entry",
            separated_pair(
                pair(
                    S::parse_identifier,
                    opt(delimited(char('['), S::parse_identifier, char(']'))),
                ),
                char(':'),
                S::parse_ascii,
            ),
        )(i)
    }

    fn parse_general<'a, E: ParseError<S> + ContextError<S>, T: Default + ApplyParsed>(
        i: S,
    ) -> IResult<S, T, E> {
        fold_many0(
            terminated(Self::parse_entry, many1(line_ending)),
            || Ok(T::default()),
            |acc, r| {
                let Ok(mut acc) = acc else {
                    return acc;
                };

                let ((key_main, key_sub), value) = {
                    let (_, k0) = r.0 .0.parse_ascii_to_string()?;
                    let k1 =
                        r.0 .1
                            .as_ref()
                            .map(|s| s.parse_ascii_to_string().map(|s| s.1))
                            .transpose()?;
                    let (_, v) = r.1.parse_ascii_to_string()?;
                    ((k0, k1), v)
                };

                match acc.apply((&key_main, key_sub.as_ref().map(|x| x.as_str())), &value) {
                    Ok(()) => Ok(acc),
                    Err(ParseApplyError::MainKey) => Err(nom::Err::Failure(E::from_error_kind(
                        r.0 .0,
                        ErrorKind::Tag,
                    ))),
                    Err(ParseApplyError::SubKey) => Err(nom::Err::Failure(E::from_error_kind(
                        r.0 .1.unwrap_or(r.0 .0),
                        ErrorKind::NonEmpty,
                    ))),
                    Err(ParseApplyError::Value) => Err(nom::Err::Failure(E::from_error_kind(
                        r.1,
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

    pub fn parse_global<'a, E: ParseError<S> + ContextError<S>>(
        i: S,
    ) -> IResult<S, GlobalModelManifest, E> {
        context(
            "global",
            preceded(
                preceded(many0(line_ending), tag("[GLOBAL]\n")),
                cut(Self::parse_general),
            ),
        )(i)
    }

    pub fn parse_stream<'a, E: ParseError<S> + ContextError<S>>(i: S) -> IResult<S, Stream, E> {
        context(
            "stream",
            preceded(
                preceded(many0(line_ending), tag("[STREAM]\n")),
                cut(Self::parse_general),
            ),
        )(i)
    }

    pub fn parse_position<'a, E: ParseError<S> + ContextError<S>>(i: S) -> IResult<S, Position, E> {
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

    use crate::model::model::Pattern;

    use super::{HeaderParser, PositionData, StreamModelMetadata};

    #[test]
    fn entry() {
        assert_eq!(
            HeaderParser::parse_entry::<VerboseError<&str>>(
                "GV_PDF[MCP]:1167198-1167761\nGV_PDF[LF0]:1167762-1167789"
            ),
            Ok((
                "\nGV_PDF[LF0]:1167762-1167789",
                (("GV_PDF", Some("MCP")), "1167198-1167761")
            ))
        );
        assert_eq!(
            HeaderParser::parse_entry::<VerboseError<&str>>("GV_PDF[LF0]:1167762-1167789"),
            Ok(("", (("GV_PDF", Some("LF0")), "1167762-1167789")))
        );
        assert_eq!(
            HeaderParser::parse_entry::<VerboseError<&str>>(
                "GV_OFF_CONTEXT:\"*-sil+*\",\"*-pau+*\""
            ),
            Ok(("", (("GV_OFF_CONTEXT", None), "\"*-sil+*\",\"*-pau+*\"")))
        );
        assert_eq!(
            HeaderParser::parse_entry::<VerboseError<&str>>("COMMENT:"),
            Ok(("", (("COMMENT", None), "")))
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
