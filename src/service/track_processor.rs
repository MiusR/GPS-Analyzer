use std::path::Path;

use crate::{io::track_loader, model::{config::{coordinates::CoordinatesConfig, snapping::SnappingConfig}, spatial::grid::Grid, track::{common::TrackOrigin, reference::ReferenceTrack, riders::{MatchedTrack, RiderTrack}}}, service::{geo_conversions, service_errors::ServiceError, snapping::snap}};


/*
    Processes a ReferenceTrack from a file
    track_path - path to file which must be gpx
    class_name - name of class the reference represents
    origin_space - origin space of the points
    destination_space - destination space of the track

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
    Processes a RiderTrack from a file with a ReferenceTrack
    track_path - path to file which must be gpx
    origin_space - space of origin of the track
    destination_space - space of the resulting RiderTrack
    origin - global origin in destination_space

    Throws: 
    ServiceError if spatial conversion fails,
    IOError if file is not found
    if file contains errors
*/
pub fn process_rider_track(track_path : &Path, origin_space: &str, destination_space : &str, origin : &TrackOrigin) -> Result<RiderTrack, ServiceError> {
    let (bib, variant) = compute_bib_from_file_name(track_path)?;

    let loaded_track = track_loader::load_track(track_path)
        .map_err( |err| {ServiceError::io_error(err)})?;
    
    let conv_config = CoordinatesConfig::new(origin_space.to_string(), destination_space.to_string());
    
    let converted_track = geo_conversions::spatial_to_rider(&loaded_track.track, origin, &conv_config)?;

    Ok(RiderTrack {
        bib : bib,
        projection : destination_space.to_string(),
        start_time : loaded_track.start_time,
        track : converted_track,
        track_origin : origin.clone(),
        variant : variant
    })
}

/*
    Processes a MatchedTrack from a file with a ReferenceTrack
    track_path - path to file which must be gpx
    ref_track - track to which it can be matched
    origin_space - origin space of the points
    snapping_config - snapping settings for the track matching

    Throws: 
    ServiceError if spatial coordinates are in different spaces
    if tracks dont have the same origin,
    IOError if file is not found
    if file contains errors
*/
pub fn snap_rider_track(rider_track : &RiderTrack, ref_track: &ReferenceTrack, grid : &Grid, snapping_config : &SnappingConfig) -> Result<MatchedTrack, ServiceError> {
    if !rider_track.projection.eq_ignore_ascii_case(&ref_track.projection) {
        Err(ServiceError::track_snapping_error(format!("tracks : {}_{} and {} are not in the same space", &rider_track.bib, &rider_track.variant, &ref_track.class).as_str()))?
    }

    if rider_track.track_origin != ref_track.origin {
        Err(ServiceError::track_snapping_error(format!("tracks : {}_{} and {} are dont have the same track origin", &rider_track.bib, &rider_track.variant, &ref_track.class).as_str()))?
    }

    let mut mapped_track = Vec::new();

    snap(&rider_track.track,&ref_track.track, grid, &mut mapped_track, snapping_config);

    Ok(MatchedTrack {
        bib : rider_track.bib.clone(),
        variant : rider_track.variant.clone(),
        projection : ref_track.projection.clone(),
        start_time : rider_track.start_time,
        track : mapped_track,
        track_origin : ref_track.origin
    })

}

/*
    Processes a MatchedTrack from a file with a ReferenceTrack BUT it tries to snap the ReferenceTrack to the file track
    at the cost of losing data about the seconds in the track
    track_path - path to file which must be gpx
    ref_track - track to which it can be matched
    origin_space - origin space of the points
    snapping_config - snapping settings for the track matching

    Throws: 
    ServiceError if spatial coordinates are in different spaces
    if tracks dont have the same origin,
    IOError if file is not found
    if file contains errors
*/
pub fn snap_rider_track_inverse(rider_track : &RiderTrack, ref_track: &ReferenceTrack, grid : &Grid, snapping_config : &SnappingConfig) -> Result<MatchedTrack, ServiceError> {
    if !rider_track.projection.eq_ignore_ascii_case(&ref_track.projection) {
        Err(ServiceError::track_snapping_error(format!("tracks : {}_{} and {} are not in the same space", &rider_track.bib, &rider_track.variant, &ref_track.class).as_str()))?
    }

    if rider_track.track_origin != ref_track.origin {
        Err(ServiceError::track_snapping_error(format!("tracks : {}_{} and {} are dont have the same track origin", &rider_track.bib, &rider_track.variant, &ref_track.class).as_str()))?
    }

    let mut mapped_track = Vec::new();

    snap(&ref_track.track, &rider_track.track, grid, &mut mapped_track, snapping_config);

    Ok(MatchedTrack {
        bib : rider_track.bib.clone(),
        variant : rider_track.variant.clone(),
        projection : ref_track.projection.clone(),
        start_time : rider_track.start_time,
        track : mapped_track,
        track_origin : ref_track.origin
    })

}



/*
    Get bib and variant from path of the form
    day_bib_variant
*/
fn compute_bib_from_file_name(path: &Path) -> Result<(u32, u32), ServiceError> {
    let stem = path
        .file_stem()
        .ok_or(ServiceError::invalid_data("file does not have a stem"))?
        .to_str()
        .ok_or(ServiceError::invalid_data("file stem is not string compatible"))?;

    let parts: Vec<&str> = stem.split('_').collect();

    if parts.len() >= 3 {
        Ok((
            parts[1].trim().parse()
            .map_err(|_| ServiceError::invalid_data("bib part is not of unsigned integer type or does not fit on 32 bits"))?, 
            parts[2].trim().parse()
            .map_err(|_| ServiceError::invalid_data("variant part is not of unsigned integer type or does not fit on 32 bits"))?
        ))
    } else {
        Err(ServiceError::invalid_data("file name is not of format {day}_{bib}_{variant}"))
    }
}