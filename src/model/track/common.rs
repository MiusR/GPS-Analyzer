use chrono::{DateTime, Utc};

use crate::model::spatial::points::SpatialPoint;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrackOrigin {
    pub epsg_x: f64,
    pub epsg_y: f64,
}

pub struct SpatialTrack {
    pub track : Vec<SpatialPoint>,
    pub start_time :  DateTime<Utc>
}