use std::path::Path;

use uuid::Uuid;

use crate::{errors::service_errors::ServiceError, internal::{io::track_loader, model::{config::{coordinates::CoordinatesConfig, snapping::SnappingConfig}, spatial::grid::Grid, track::{common::TrackOrigin, reference::ReferenceTrack, riders::{MatchedTrack, RiderTrack}}}, service::{geo_conversions, snapping::snap}}};


// FIXME class_name should not be here, it should not be sored in ReferenceTrack, we should have a separate structure that composes a reference track and holds metadata about it!
/*
    Generate a ReferenceTrack from a file found at @track_path.
    Throws: 
    ServiceError if spatial conversion fails,
    IOError if file is not found
    if file contains errors
*/
pub fn process_reference_track(track_path : &Path, class_name : &str, origin_space: &str, destination_space : &str) -> Result<ReferenceTrack, ServiceError> {
    let loaded_track = track_loader::load_track(track_path).map_err(
        |err| {ServiceError::io_error(err)}
    )?;
    
    // Conversion Settings
    let conv_config = CoordinatesConfig::new(origin_space.to_string(), destination_space.to_string());

    let (track_origin, converted_track) = geo_conversions::spatial_to_reference(&loaded_track.track, &conv_config)?;

    Ok(ReferenceTrack{
        class : class_name.to_string(),
        projection : destination_space.to_string(),
        track : converted_track,
        origin : track_origin
    })
}

/*
    Generates a RiderTrack from a file found at @track_path 
    Throws: 
    ServiceError if spatial conversion fails,
    IOError if file is not found
    if file contains errors
*/
pub fn process_rider_track(track_path : &Path, rider_uuid : Uuid, variant : u32, origin_space: &str, destination_space : &str, origin : &TrackOrigin) -> Result<RiderTrack, ServiceError> {

    let loaded_track = track_loader::load_track(track_path)
        .map_err( |err| {ServiceError::io_error(err)})?;
    
    let conv_config = CoordinatesConfig::new(origin_space.to_string(), destination_space.to_string());
    
    let converted_track = geo_conversions::spatial_to_rider(&loaded_track.track, origin, &conv_config)?;

    Ok(RiderTrack {
        rider_uuid,
        projection : destination_space.to_string(),
        start_time : loaded_track.start_time,
        track : converted_track,
        track_origin : origin.clone(),
        variant : variant
    })
}

/*
    Generates a MatchedTrack from a @rider_track with a @ref_track.
    Throws: 
    ServiceError if spatial coordinates are in different spaces
    if tracks dont have the same origin,
    IOError if file is not found
    if file contains errors
*/
pub fn snap_rider_track(rider_track : &RiderTrack, ref_track: &ReferenceTrack, grid : &Grid, snapping_config : &SnappingConfig) -> Result<MatchedTrack, ServiceError> {
    if !rider_track.projection.eq_ignore_ascii_case(&ref_track.projection) {
        Err(ServiceError::track_snapping_error(format!("tracks : {}_{} and {} are not in the same space", &rider_track.rider_uuid, &rider_track.variant, &ref_track.class).as_str()))?
    }

    if rider_track.track_origin != ref_track.origin {
        Err(ServiceError::track_snapping_error(format!("tracks : {}_{} and {} are dont have the same track origin", &rider_track.rider_uuid, &rider_track.variant, &ref_track.class).as_str()))?
    }

    let mut mapped_track = Vec::new();

    snap(&rider_track.track,&ref_track.track, grid, &mut mapped_track, snapping_config);

    Ok(MatchedTrack {
        bound_uuid : rider_track.rider_uuid.clone(),
        variant : rider_track.variant.clone(),
        projection : ref_track.projection.clone(),
        start_time : rider_track.start_time,
        track : mapped_track,
        track_origin : ref_track.origin
    })

}

/*
    Generates a MatchedTrack from a @rider_track with a @ref_track BUT it tries to snap the @ref_track to the @rider_track
    at the cost of losing data about the seconds in the track
    Throws: 
    ServiceError if spatial coordinates are in different spaces
    if tracks dont have the same origin,
    IOError if file is not found
    if file contains errors
*/
pub fn snap_rider_track_inverse(rider_track : &RiderTrack, ref_track: &ReferenceTrack, grid : &Grid, snapping_config : &SnappingConfig) -> Result<MatchedTrack, ServiceError> {
    if !rider_track.projection.eq_ignore_ascii_case(&ref_track.projection) {
        Err(ServiceError::track_snapping_error(format!("tracks : {}_{} and {} are not in the same space", &rider_track.rider_uuid, &rider_track.variant, &ref_track.class).as_str()))?
    }

    if rider_track.track_origin != ref_track.origin {
        Err(ServiceError::track_snapping_error(format!("tracks : {}_{} and {} are dont have the same track origin", &rider_track.rider_uuid, &rider_track.variant, &ref_track.class).as_str()))?
    }

    let mut mapped_track = Vec::new();

    snap(&ref_track.track, &rider_track.track, grid, &mut mapped_track, snapping_config);

    Ok(MatchedTrack {
        bound_uuid : rider_track.rider_uuid.clone(),
        variant : rider_track.variant.clone(),
        projection : ref_track.projection.clone(),
        start_time : rider_track.start_time,
        track : mapped_track,
        track_origin : ref_track.origin
    })

}

