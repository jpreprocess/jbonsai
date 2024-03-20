#[derive(Debug, thiserror::Error)]
pub enum LabelError {
    #[error("jlabel failed to parse fullcontext-label: {0}")]
    JLabelParse(#[from] jlabel::ParseError),
    #[error("Expected a fullcontext-label in {0}")]
    MissingLabel(String),
    #[error("Failed to parse as floating-point number")]
    FloatParse(#[from] std::num::ParseFloatError),

    #[error("The length of `times` and `labels` must be the same")]
    LengthMismatch,
}

pub struct Labels {
    labels: Vec<jlabel::Label>,
    times: Vec<(f64, f64)>,
}

impl Labels {
    pub fn load_from_strings<S: AsRef<str>>(
        sampling_rate: usize,
        fperiod: usize,
        lines: &[S],
    ) -> Result<Self, LabelError> {
        let mut labels = Vec::with_capacity(lines.len());
        let mut times = Vec::with_capacity(lines.len());

        let rate = sampling_rate as f64 / (fperiod as f64 * 1e+7);

        for line in lines {
            let line = line.as_ref();

            let mut split = line.splitn(3, ' ');
            let first = split
                .next()
                .expect("`splitn` is expected to always have at least one element.");

            if let Some(second) = split.next() {
                let third = split
                    .next()
                    .ok_or_else(|| LabelError::MissingLabel(line.to_string()))?;

                let mut start: f64 = first.parse()?;
                let mut end: f64 = second.parse()?;

                start *= rate;
                end *= rate;

                let label = third.parse()?;

                times.push((start, end));
                labels.push(label);
            } else if first.is_empty() {
                continue;
            } else {
                let label = first.parse()?;
                times.push((-1.0, -1.0));
                labels.push(label);
            }
        }

        Self::new(labels, Some(times))
    }

    pub fn new(
        labels: Vec<jlabel::Label>,
        times: Option<Vec<(f64, f64)>>,
    ) -> Result<Self, LabelError> {
        if let Some(mut times) = times {
            if labels.len() != times.len() {
                return Err(LabelError::LengthMismatch);
            }

            for i in 0..times.len() {
                if i + 1 < times.len() {
                    if times[i].1 < 0.0 && times[i + 1].0 >= 0.0 {
                        times[i].1 = times[i + 1].0;
                    } else if times[i].1 >= 0.0 && times[i + 1].0 < 0.0 {
                        times[i + 1].0 = times[i].1;
                    }
                }

                if times[i].0 < 0.0 {
                    times[i].0 = -1.0;
                }
                if times[i].1 < 0.0 {
                    times[i].1 = -1.0;
                }
            }

            Ok(Self { times, labels })
        } else {
            Ok(Self {
                times: vec![(-1.0, -1.0); labels.len()],
                labels,
            })
        }
    }

    pub fn labels(&self) -> &[jlabel::Label] {
        &self.labels
    }
    pub fn times(&self) -> &[(f64, f64)] {
        &self.times
    }
}
