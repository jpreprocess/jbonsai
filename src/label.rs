use std::str::FromStr;

struct LabelString {
    content: jlabel::Label,
    start: f64,
    end: f64,
}

impl LabelString {
    fn parse(s: &str, rate: f64) -> Self {
        Self::parse_digit_string(s, rate).unwrap_or(Self {
            // TODO: remove this unwrap
            content: jlabel::Label::from_str(s).unwrap(),
            start: -1.0,
            end: -1.0,
        })
    }
    fn parse_digit_string(s: &str, rate: f64) -> Option<Self> {
        let mut iter = s.splitn(3, ' ');
        let start: f64 = iter.next().and_then(|s| s.parse().ok())?;
        let end: f64 = iter.next().and_then(|s| s.parse().ok())?;
        let content = iter.next()?.parse().ok()?;
        Some(Self {
            content,
            start: rate * start,
            end: rate * end,
        })
    }
}

pub struct Label {
    strings: Vec<LabelString>,
}

impl Label {
    pub fn load_from_strings(sampling_rate: usize, fperiod: usize, lines: &[String]) -> Self {
        let rate = sampling_rate as f64 / (fperiod as f64 * 1e+7);
        let mut strings = Vec::with_capacity(lines.len());

        for line in lines {
            let Some(first_char) = line.chars().next() else {
                break;
            };
            if !first_char.is_ascii_graphic() {
                break;
            }

            strings.push(LabelString::parse(line, rate));
        }

        for i in 0..strings.len() {
            if i + 1 < strings.len() {
                if strings[i].end < 0.0 && strings[i + 1].start >= 0.0 {
                    strings[i].end = strings[i + 1].start;
                } else if strings[i].end >= 0.0 && strings[i + 1].start < 0.0 {
                    strings[i + 1].start = strings[i].end;
                }
            }
            if strings[i].start < 0.0 {
                strings[i].start = -1.0;
            }
            if strings[i].end < 0.0 {
                strings[i].end = -1.0;
            }
        }

        Self { strings }
    }

    pub fn get_size(&self) -> usize {
        self.strings.len()
    }
    pub fn get_label(&self, index: usize) -> &jlabel::Label {
        &self.strings[index].content
    }
    pub fn get_start_frame(&self, index: usize) -> f64 {
        self.strings[index].start
    }
    pub fn get_end_frame(&self, index: usize) -> f64 {
        self.strings[index].end
    }
}
