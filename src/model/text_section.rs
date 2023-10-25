use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct TextSection {
    global: Global,
    streams: HashMap<String, Stream>,
}

impl TextSection {
    pub fn parse(text: String) -> Result<TextSection, Box<dyn std::error::Error>> {
        #[derive(Debug)]
        enum Section {
            Global,
            Stream,
            Position,
        }

        let mut section = Section::Global;

        let mut global = Global::default();
        let mut position = Position::default();
        let mut streams: HashMap<&str, Stream> = HashMap::new();

        for line in text.split("\n") {
            match line {
                "[GLOBAL]" => {
                    section = Section::Global;
                    continue;
                }
                "[STREAM]" => {
                    section = Section::Stream;
                    continue;
                }
                "[POSITION]" => {
                    section = Section::Position;
                    continue;
                }
                "" => continue,
                _ => (),
            }

            let Some((key, value)) = line.split_once(':') else {
                return Err(anyhow::anyhow!("Cannot parse line {}", line).into());
            };
            let (key_main, stream) = match key.split_once('[') {
                Some((m, s)) => (m, Some(streams.entry(&s[..s.len() - 1]).or_default())),
                None => (key, None),
            };
            match (&section, stream) {
                (Section::Global, None) => global.parse(key_main, value)?,
                (Section::Position, None) => position.parse(key_main, value)?,
                (Section::Stream, Some(stream)) => {
                    stream.stream.parse(key_main, value)?;
                }
                (Section::Position, Some(stream)) => {
                    stream.position.parse(key_main, value)?;
                }
                _ => (),
            }
        }

        Ok(TextSection {
            global,
            streams: HashMap::from_iter(streams.into_iter().map(|(k, v)| (k.to_string(), v))),
        })
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

impl Global {
    fn parse(&mut self, key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
        match key {
            "HTS_VOICE_VERSION" => self.hts_voice_version = value.to_string(),
            "SAMPLING_FREQUENCY" => self.sampling_frequency = value.parse()?,
            "FRAME_PERIOD" => self.frame_period = value.parse()?,
            "NUM_STATES" => self.num_states = value.parse()?,
            "NUM_STREAMS" => self.num_streams = value.parse()?,
            "STREAM_TYPE" => self.stream_type = value.split(",").map(|s| s.to_string()).collect(),
            "FULLCONTEXT_FORMAT" => self.fullcontext_format = value.to_string(),
            "FULLCONTEXT_VERSION" => self.fullcontext_version = value.to_string(),
            "GV_OFF_CONTEXT" => {
                self.gv_off_context = value.split(",").map(|s| s.to_string()).collect()
            }
            "COMMENT" => (),
            _ => Err(anyhow::anyhow!("Unknown key {}", key))?,
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct Stream {
    stream: StreamData,
    position: PositionData,
}

#[derive(Debug, Clone, Default)]
pub struct StreamData {
    vector_length: usize,
    num_windows: usize,
    is_msd: bool,
    use_gv: bool,
    option: Vec<String>,
}
impl StreamData {
    fn parse(&mut self, key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
        match key {
            "VECTOR_LENGTH" => self.vector_length = value.parse()?,
            "IS_MSD" => self.is_msd = parse_bool(value)?,
            "NUM_WINDOWS" => self.num_windows = value.parse()?,
            "USE_GV" => self.use_gv = parse_bool(value)?,
            "OPTION" => self.option = value.split(",").map(|s| s.to_string()).collect(),
            _ => Err(anyhow::anyhow!("Unknown key {}", key))?,
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct Position {
    duration_pdf: (usize, usize),
    duration_tree: (usize, usize),
}
impl Position {
    fn parse(&mut self, key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
        match key {
            "DURATION_PDF" => self.duration_pdf = parse_range(value)?,
            "DURATION_TREE" => self.duration_tree = parse_range(value)?,
            _ => Err(anyhow::anyhow!("Unknown key {}", key))?,
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct PositionData {
    stream_win: Vec<(usize, usize)>,
    stream_pdf: (usize, usize),
    stream_tree: (usize, usize),
    gv_pdf: (usize, usize),
    gv_tree: (usize, usize),
}

impl PositionData {
    fn parse(&mut self, key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
        match key {
            "STREAM_WIN" => {
                self.stream_win = value
                    .split(",")
                    .map(|s| parse_range(s))
                    .collect::<Result<_, _>>()?
            }
            "STREAM_PDF" => self.stream_pdf = parse_range(value)?,
            "STREAM_TREE" => self.stream_tree = parse_range(value)?,
            "GV_PDF" => self.gv_pdf = parse_range(value)?,
            "GV_TREE" => self.gv_tree = parse_range(value)?,
            _ => Err(anyhow::anyhow!("Unknown key {}", key))?,
        }
        Ok(())
    }
}

fn parse_bool(value: &str) -> Result<bool, Box<dyn std::error::Error>> {
    match value {
        "1" => Ok(true),
        "0" => Ok(false),
        _ => Err(anyhow::anyhow!("Cannot parse {} as bool", value).into()),
    }
}

fn parse_range(value: &str) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    match value.split_once('-') {
        Some((s, e)) => Ok((s.parse()?, e.parse()?)),
        _ => Err(anyhow::anyhow!("Cannot parse {} as range", value).into()),
    }
}

#[cfg(test)]
mod tests {
    use super::TextSection;

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
    fn parse() {
        TextSection::parse(CONTENT.to_string()).unwrap();
    }
}
