
use crate::internal::model::{config::analysis::AnalysisConfig, spatial::points::MatchPoint};

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub enum Severity {
    Ok = 0,
    Minor = 1,
    Moderate = 2,
    Severe = 3,
    Max = 4,
}

impl Severity {
    fn from_u16(v: u16) -> Self {
        match v {
            1 => Severity::Minor,
            2 => Severity::Moderate,
            3 => Severity::Severe,
            4..=u16::MAX => Severity::Max,
            _ => Severity::Ok,
        }
    }
}

/*
    Fileters the used point errors to prevent the buildup of one-off errors. Look at AnaltsysConfig.minimum_continuous_error
*/
fn set_error_flags(
    matches: &mut [MatchPoint],
    severity : &mut Vec<Severity>,
    config: &AnalysisConfig
) {
    let mut error_count = 0;
    for point_index in 0..matches.len() {
        match severity[point_index] {
            Severity::Ok => {
                if error_count >= config.get_minimum_cont_error() {
                    for previous_index in 1..=error_count {
                        matches[point_index - previous_index].count_to_error = true;
                    }
                }else {
                    for previous_index in 0..error_count {
                        severity[point_index - previous_index] = Severity::Ok;
                    }
                }
                error_count = 0;
            }
            _ => {
                error_count += 1;
            }
        }
    }
}

/*
    Returns an ordered Vec<Severity> where v[i] refers to point i in matches.
    The severity is based on the point lateral, if at any point the lateral is > config lateral.
*/
pub fn classify_lateral(
    matches: &mut [MatchPoint], 
    config: &AnalysisConfig
) -> Vec<Severity> {
    let mut computed_severity = matches
        .iter()
        .map(|matched_point| {
            let allowed_dev = config.get_allowed_deviance();
            
            if matched_point.lateral > allowed_dev {
                let deviance = matched_point.lateral - allowed_dev;
                let raw_severity = (deviance.round() as u32 / config.get_incremental_severity().round() as u32) as u32 + 1 as u32;

                if raw_severity >= Severity::Max as u32{
                    Severity::Max
                } else {
                    Severity::from_u16(raw_severity as u16)
                }
            } else {
                Severity::Ok
            }
        })
        .collect();
    set_error_flags(matches, &mut computed_severity, config);
    computed_severity
}

// TODO : use this result or similar to account for gps errors (spatial shifting that is > expected distance from track but has the same track pattern)
/*
    Returns an ordered Vec<Severity> where v[i] refers to point i in matches.
    The severity is based on the track direction, if at any point the "forward" direction of the track differs from the reference track.
*/
pub fn classify_directional(
    matches: &mut [MatchPoint], 
    config: &AnalysisConfig
) -> Vec<Severity> {
    let mut computed_severity = matches
        .iter()
        .map(|matched_point| {
            let allowed_deviation = config.get_directional_deviance();

            if matched_point.direction_similarity <= 0.0 {
                Severity::Max
            }else if matched_point.direction_similarity < allowed_deviation {
                let deviance = allowed_deviation - matched_point.direction_similarity;
                let raw_severity = (((deviance / config.get_incremental_severity().round() / 10.0) + 0.1) * 10.0).round() as u32 ;

                if raw_severity >= Severity::Max as u32{
                    Severity::Max
                } else {

                    Severity::from_u16(raw_severity as u16)
                }
            } else {
                Severity::Ok
            }
        })
        .collect();
    set_error_flags(matches, &mut computed_severity, config);
    computed_severity
}