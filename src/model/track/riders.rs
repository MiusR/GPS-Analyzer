
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::model::{spatial::points::{MatchPoint, RiderPoint}, track::common::TrackOrigin};

#[derive(Clone)]
pub struct MatchedTrack {
    pub bound_uuid : Uuid,
    pub variant : u32,
    pub projection : String,
    pub start_time : DateTime<Utc>,
    pub track_origin : TrackOrigin,
    pub track : Vec<MatchPoint>
}

#[derive(Clone)]
pub struct RiderTrack {
    pub rider_uuid : Uuid,
    pub variant : u32,
    pub projection : String,
    pub start_time : DateTime<Utc>,
    pub track_origin : TrackOrigin,
    pub track : Vec<RiderPoint>
}