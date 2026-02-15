use crate::internal::model::{spatial::points::RefPoint, track::common::TrackOrigin};

#[derive(Clone, Debug)]
pub struct ReferenceTrack {
    pub class : String,
    pub projection : String,
    pub origin : TrackOrigin,
    pub track : Vec<RefPoint>
}