//! Sequence of [`jlabel::Label`]s, with time alignments.
//!
//! Each [`jlabel::Label`] corresponds to one phoneme.

use crate::engine::Condition;

/// Error while parsing labels.
#[derive(Debug, thiserror::Error)]
pub enum LabelError {
    /// jlabel failed to parse full-context label.
    #[error("jlabel failed to parse fullcontext-label: {0}")]
    JLabelParse(#[from] jlabel::ParseError),
    /// Full-context label was not included in the provided string.
    #[error("Expected a fullcontext-label in {0}")]
    MissingLabel(String),
    /// Failed to parse time alignment as `f64`.
    #[error("Failed to parse as floating-point number")]
    FloatParse(#[from] std::num::ParseFloatError),

    /// The length of the provided `times` and `labels` differed.
    #[error("The length of `times` and `labels` must be the same")]
    LengthMismatch,
}

/// Sequence of [`jlabel::Label`] with time alignments.
pub struct Labels {
    labels: Vec<jlabel::Label>,
    times: Vec<(f64, f64)>,
}

impl Labels {
    /// Load [`Labels`] from sequence of strings.
    ///
    /// If the string contains time alignments, [`Labels::load_from_strings`] will parse and store it in [`Labels`].
    pub fn load_from_strings<S: AsRef<str>>(
        sampling_rate: usize,
        fperiod: usize,
        lines: &[S],
    ) -> Result<Self, LabelError> {
        let mut labels = Vec::with_capacity(lines.len());
        let mut times = Vec::with_capacity(lines.len());

        // start/end times are multiplied with 1e+7
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

    /// Create a new [`Labels`] directly from labels and time alignments.
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

    /// Get labels ([`jlabel::Label`]).
    pub fn labels(&self) -> &[jlabel::Label] {
        &self.labels
    }
    /// Get time alignments.
    pub fn times(&self) -> &[(f64, f64)] {
        &self.times
    }
}

/// Structs that can be converted into [`Labels`].
pub trait ToLabels {
    /// Convert this into [`Labels`].
    fn to_labels(self, condition: &Condition) -> Result<Labels, LabelError>;
}

impl ToLabels for Vec<jlabel::Label> {
    fn to_labels(self, _condition: &Condition) -> Result<Labels, LabelError> {
        Labels::new(self, None)
    }
}

impl<S: AsRef<str>> ToLabels for &[S] {
    fn to_labels(self, condition: &Condition) -> Result<Labels, LabelError> {
        Labels::load_from_strings(
            condition.get_sampling_frequency(),
            condition.get_fperiod(),
            self,
        )
    }
}

impl<const N: usize, S: AsRef<str>> ToLabels for &[S; N] {
    fn to_labels(self, condition: &Condition) -> Result<Labels, LabelError> {
        self.as_slice().to_labels(condition)
    }
}

impl ToLabels for Vec<String> {
    fn to_labels(self, condition: &Condition) -> Result<Labels, LabelError> {
        self.as_slice().to_labels(condition)
    }
}

#[cfg(test)]
mod tests {
    use super::Labels;

    #[test]
    fn with_alignment() {
        let lines = [
            "0 14925000 xx^xx-sil+b=o/A:xx+xx+xx/B:xx-xx_xx/C:xx_xx+xx/D:xx+xx_xx/E:xx_xx!xx_xx-xx/F:xx_xx#xx_xx@xx_xx|xx_xx/G:4_4%0_xx_xx/H:xx_xx/I:xx-xx@xx+xx&xx-xx|xx+xx/J:1_4/K:1+1-4",
            "14925000 16725000 xx^sil-b+o=N/A:-3+1+4/B:xx-xx_xx/C:02_xx+xx/D:xx+xx_xx/E:xx_xx!xx_xx-xx/F:4_4#0_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:xx_xx/I:1-4@1+1&1-1|1+4/J:xx_xx/K:1+1-4",
            "16725000 17525000 sil^b-o+N=s/A:-3+1+4/B:xx-xx_xx/C:02_xx+xx/D:xx+xx_xx/E:xx_xx!xx_xx-xx/F:4_4#0_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:xx_xx/I:1-4@1+1&1-1|1+4/J:xx_xx/K:1+1-4",
            "17525000 18125000 b^o-N+s=a/A:-2+2+3/B:xx-xx_xx/C:02_xx+xx/D:xx+xx_xx/E:xx_xx!xx_xx-xx/F:4_4#0_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:xx_xx/I:1-4@1+1&1-1|1+4/J:xx_xx/K:1+1-4",
            "18125000 19725000 o^N-s+a=i/A:-1+3+2/B:xx-xx_xx/C:02_xx+xx/D:xx+xx_xx/E:xx_xx!xx_xx-xx/F:4_4#0_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:xx_xx/I:1-4@1+1&1-1|1+4/J:xx_xx/K:1+1-4",
            "19725000 20825000 N^s-a+i=sil/A:-1+3+2/B:xx-xx_xx/C:02_xx+xx/D:xx+xx_xx/E:xx_xx!xx_xx-xx/F:4_4#0_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:xx_xx/I:1-4@1+1&1-1|1+4/J:xx_xx/K:1+1-4",
            "20825000 22725000 s^a-i+sil=xx/A:0+4+1/B:xx-xx_xx/C:02_xx+xx/D:xx+xx_xx/E:xx_xx!xx_xx-xx/F:4_4#0_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:xx_xx/I:1-4@1+1&1-1|1+4/J:xx_xx/K:1+1-4",
            "22725000 30325000 a^i-sil+xx=xx/A:xx+xx+xx/B:xx-xx_xx/C:xx_xx+xx/D:xx+xx_xx/E:4_4!0_xx-xx/F:xx_xx#xx_xx@xx_xx|xx_xx/G:xx_xx%xx_xx_xx/H:1_4/I:xx-xx@xx+xx&xx-xx|xx+xx/J:xx_xx/K:1+1-4",
    ];
        let labels = Labels::load_from_strings(48000, 240, &lines).unwrap();
        let times = labels.times();

        let answer = [
            (0.0, 298.5),
            (298.5, 334.5),
            (334.5, 350.5),
            (350.5, 362.5),
            (362.5, 394.5),
            (394.5, 416.5),
            (416.5, 454.5),
            (454.5, 606.5),
        ];

        assert_eq!(times.len(), answer.len());

        for (time, ans) in times.iter().zip(answer) {
            approx::assert_ulps_eq!(time.0, ans.0);
            approx::assert_ulps_eq!(time.1, ans.1);
        }
    }
}
