use std::{collections::HashMap, str::FromStr};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_till, take_till1, take_until, take_while, take_while1},
    character::{complete::line_ending, is_alphabetic},
    error::{ErrorKind, ParseError},
    multi::{fold_many0, many0, many1},
    sequence::{delimited, pair, preceded, separated_pair, terminated},
    IResult,
};

// fn parse_utf8<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, String, E> {
//     match String::from_utf8(i.to_vec()) {
//         Ok(str) => Ok((&[], str)),
//         Err(_) => Err(nom::Err::Failure(E::from_error_kind(i, ErrorKind::Fail))),
//     }
// }

fn parse_identifier<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    // verify(parse_utf8, |s: &str| {
    //     s.chars().all(|c| matches!(c, 'a'..='z' | '_'))
    // })(i)
    take_while1(|c: char| c.is_alphanumeric() || c == '_')(i)
}

fn parse_ascii<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    // verify(parse_utf8, |s: &str| s.is_ascii())(i)
    take_while(|c: char| c.is_ascii() && c != '\n')(i)
}

fn parse_entry<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, ((&'a str, &'a str), &'a str), E> {
    use nom::character::complete::char;
    separated_pair(
        pair(
            parse_identifier,
            alt((delimited(char('['), parse_identifier, char(']')), tag(""))),
        ),
        char(':'),
        parse_ascii,
    )(i)
}

trait ApplyParsed {
    fn apply<'a, E: ParseError<&'a str>>(
        &mut self,
        k: (&'a str, &'a str),
        v: &'a str,
    ) -> Result<(), nom::Err<E>>;
    fn parse<'a, T: FromStr, E: ParseError<&'a str>>(v: &'a str) -> Result<T, nom::Err<E>> {
        v.parse()
            .map_err(|_| nom::Err::Failure(E::from_error_kind(v, ErrorKind::Digit)))
    }
}

#[derive(Debug, Clone, Default)]
pub struct Global {
    hts_voice_version: String,
    sampling_frequency: usize,
    frame_period: usize,
    num_voices: usize,
    num_states: usize,
    num_streams: usize,
    stream_type: Vec<String>,
    fullcontext_format: String,
    fullcontext_version: String,
    gv_off_context: Vec<String>,
}

impl ApplyParsed for Global {
    fn apply<'a, E: ParseError<&'a str>>(
        &mut self,
        k: (&'a str, &'a str),
        v: &'a str,
    ) -> Result<(), nom::Err<E>> {
        if !k.1.is_empty() {
            return Err(nom::Err::Failure(E::from_error_kind(k.1, ErrorKind::Count)));
        }
        match k.0 {
            "HTS_VOICE_VERSION" => self.hts_voice_version = v.to_string(),
            "SAMPLING_FREQUENCY" => self.sampling_frequency = Self::parse(v)?,
            "FRAME_PERIOD" => self.frame_period = Self::parse(v)?,
            "NUM_VOICES" => self.num_voices = Self::parse(v)?,
            "NUM_STATES" => self.num_states = Self::parse(v)?,
            "NUM_STREAMS" => self.num_streams = Self::parse(v)?,
            "STREAM_TYPE" => self.stream_type = v.split(',').map(|s| s.to_string()).collect(),
            "FULLCONTEXT_FORMAT" => self.fullcontext_format = v.to_string(),
            "FULLCONTEXT_VERSION" => self.fullcontext_version = v.to_string(),
            "GV_OFF_CONTEXT" => self.gv_off_context = v.split(',').map(|s| s.to_string()).collect(),
            "COMMENT" => (),
            _ => Err(nom::Err::Failure(E::from_error_kind(k.0, ErrorKind::Tag)))?,
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct Stream {
    stream: HashMap<String, StreamData>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StreamData {
    vector_length: usize,
    num_windows: usize,
    is_msd: bool,
    use_gv: bool,
    option: Vec<String>,
}

impl ApplyParsed for Stream {
    fn apply<'a, E: ParseError<&'a str>>(
        &mut self,
        k: (&'a str, &'a str),
        v: &'a str,
    ) -> Result<(), nom::Err<E>> {
        let entry = self.stream.entry(k.1.to_string()).or_default();
        match k.0 {
            "VECTOR_LENGTH" => entry.vector_length = Self::parse(v)?,
            "NUM_WINDOWS" => entry.num_windows = Self::parse(v)?,
            "IS_MSD" => entry.is_msd = v == "1",
            "USE_GV" => entry.use_gv = v == "1",
            "OPTION" => entry.option = v.split(',').map(|s| s.to_string()).collect(),
            _ => Err(nom::Err::Failure(E::from_error_kind(k.0, ErrorKind::Tag)))?,
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct Position {
    duration_pdf: (usize, usize),
    duration_tree: (usize, usize),
    position: HashMap<String, PositionData>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PositionData {
    stream_win: Vec<(usize, usize)>,
    stream_pdf: (usize, usize),
    stream_tree: (usize, usize),
    gv_pdf: (usize, usize),
    gv_tree: (usize, usize),
}

impl ApplyParsed for Position {
    fn apply<'a, E: ParseError<&'a str>>(
        &mut self,
        k: (&'a str, &'a str),
        v: &'a str,
    ) -> Result<(), nom::Err<E>> {
        fn parse_duration<'a, E: ParseError<&'a str>>(
            v: &'a str,
        ) -> Result<(usize, usize), nom::Err<E>> {
            match v.split_once('-') {
                Some((s, e)) => Ok((Position::parse(s)?, Position::parse(e)?)),
                _ => Err(nom::Err::Failure(E::from_error_kind(v, ErrorKind::Digit))),
            }
        }
        if k.1.is_empty() {
            match k.0 {
                "DURATION_PDF" => self.duration_pdf = parse_duration(v)?,
                "DURATION_TREE" => self.duration_tree = parse_duration(v)?,
                _ => Err(nom::Err::Failure(E::from_error_kind(k.0, ErrorKind::Tag)))?,
            }
        } else {
            let entry = self.position.entry(k.1.to_string()).or_default();
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
                _ => Err(nom::Err::Failure(E::from_error_kind(k.0, ErrorKind::Tag)))?,
            }
        }
        Ok(())
    }
}

fn parse_general<'a, E: ParseError<&'a str>, T: Default + ApplyParsed>(
    i: &'a str,
) -> IResult<&'a str, T, E> {
    fold_many0(
        terminated(parse_entry, many1(line_ending)),
        || Ok(T::default()),
        |acc, (k, v)| {
            let Ok(mut acc) = acc else {
                return acc;
            };
            match acc.apply(k, v) {
                Ok(()) => Ok(acc),
                Err(err) => Err(err),
            }
        },
    )(i)
    .and_then(|r| match r {
        (rest, Ok(result)) => Ok((rest, result)),
        (_, Err(err)) => Err(err),
    })
}

fn parse_global<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Global, E> {
    preceded(
        preceded(many0(line_ending), tag("[GLOBAL]\n")),
        parse_general,
    )(i)
}

fn parse_stream<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Stream, E> {
    preceded(
        preceded(many0(line_ending), tag("[STREAM]\n")),
        parse_general,
    )(i)
}

fn parse_position<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Position, E> {
    preceded(
        preceded(many0(line_ending), tag("[POSITION]\n")),
        parse_general,
    )(i)
}

#[cfg(test)]
mod tests {
    use nom::error::VerboseError;

    use crate::model::nom::{parse_entry, parse_position, parse_stream, PositionData, StreamData};

    use super::{parse_ascii, parse_global};

    #[test]
    fn ascii() {
        assert_eq!(
            parse_ascii::<VerboseError<&str>>("hogehoge\"\nfugafuga"),
            Ok(("\nfugafuga", "hogehoge\""))
        );
    }
    #[test]
    fn entry() {
        assert_eq!(
            parse_entry::<VerboseError<&str>>(
                "GV_PDF[MCP]:1167198-1167761\nGV_PDF[LF0]:1167762-1167789"
            ),
            Ok((
                "\nGV_PDF[LF0]:1167762-1167789",
                (("GV_PDF", "MCP"), "1167198-1167761")
            ))
        );
        assert_eq!(
            parse_entry::<VerboseError<&str>>("GV_PDF[LF0]:1167762-1167789"),
            Ok(("", (("GV_PDF", "LF0"), "1167762-1167789")))
        );
        assert_eq!(
            parse_entry::<VerboseError<&str>>("GV_OFF_CONTEXT:\"*-sil+*\",\"*-pau+*\""),
            Ok(("", (("GV_OFF_CONTEXT", ""), "\"*-sil+*\",\"*-pau+*\"")))
        );
        assert_eq!(
            parse_entry::<VerboseError<&str>>("COMMENT:"),
            Ok(("", (("COMMENT", ""), "")))
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
        let (rest, global) = parse_global::<VerboseError<&str>>(CONTENT).unwrap();
        assert_eq!(rest.len(), 751);
        assert_eq!(global.hts_voice_version, "1.0");
        assert_eq!(global.gv_off_context, vec!["\"*-sil+*\"", "\"*-pau+*\"",]);
    }
    #[test]
    fn stream() {
        let (rest, stream) = parse_stream::<VerboseError<&str>>(&CONTENT[224..]).unwrap();
        assert_eq!(rest.len(), 487);
        assert_eq!(
            stream.stream.get("MCP"),
            Some(&StreamData {
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
        let (rest, position) = parse_position::<VerboseError<&str>>(&CONTENT[488..]).unwrap();
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
}
