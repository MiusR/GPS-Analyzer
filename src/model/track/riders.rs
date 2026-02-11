
use chrono::{DateTime, Utc};

use crate::model::{spatial::points::{MatchPoint, RiderPoint}, track::common::TrackOrigin};

#[derive(Clone)]
pub struct MatchedTrack {
    pub bib : u32,
    pub variant : u32,
    pub projection : String,
    pub start_time : DateTime<Utc>,
    pub track_origin : TrackOrigin,
    pub track : Vec<MatchPoint>
}

#[derive(Clone)]
pub struct RiderTrack {
    pub bib : u32,
    pub variant : u32,
    pub projection : String,
    pub start_time : DateTime<Utc>,
    pub track_origin : TrackOrigin,
    pub track : Vec<RiderPoint>
}