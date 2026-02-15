use proj::{Coord};

use crate::internal::{model::{config::coordinates::CoordinatesConfig, track::common::{TrackOrigin}}, service::service_errors::ServiceError};
use crate::internal::model::spatial::points::{RefPoint, RiderPoint, SpatialPoint};

/*
    Tries to convert from a vector of @spatial_points into a vector of rider points
    and computes a time offset from start

    Throws: CoordinateConversionError if any of the initial points can not be converted to the new space 
*/
pub fn spatial_to_rider(spatial_points : &[SpatialPoint], track_origin : &TrackOrigin, config : &CoordinatesConfig) -> Result<Vec<RiderPoint>, ServiceError> {
    
    let reference_points = convert_to_space(spatial_points, &config, |(x64,y64), point| {
        RiderPoint {
            x: (x64 - track_origin.epsg_x) as f32,
            y: (y64 - track_origin.epsg_y) as f32,
            z: point.elev.unwrap_or(0.0) as f32,
            delta_seconds : point.delta_seconds.unwrap_or(0.0)
        }
    })?;
    Ok(reference_points)
}

/*
    Tries to convert from a vector of @spatial_points into a vector of reference points
    and computes a rolling distance in the new coordinate space.
    Throws: CoordinateConversionError if any of the initial points can not be converted to the new space 
*/
pub fn spatial_to_reference(spatial_points : &[SpatialPoint], config : &CoordinatesConfig) -> Result<(TrackOrigin, Vec<RefPoint>), ServiceError> {
     let track_origin = get_track_origin(spatial_points, config, |(x64,y64), _| {
        TrackOrigin { 
            epsg_x: x64, 
            epsg_y: y64 
        } 
    })?;

    let mut ref_points = convert_to_space(spatial_points, &config, |(x64,y64), point| {
        RefPoint {
            x: (x64 - track_origin.epsg_x) as f32,
            y: (y64 - track_origin.epsg_y) as f32,
            z: point.elev.unwrap_or(0.0) as f32,
            total_distance: 0.0, // Placeholder
        }
    })?;

    let mut total_distance = 0.0f32;
    for i in 1..ref_points.len() {
        let previous_point = ref_points[i-1];
        let current_point = ref_points[i];
        
        let dx = current_point.x - previous_point.x;
        let dy = current_point.y - previous_point.y;
        
        total_distance += (dx * dx + dy * dy).sqrt();
        ref_points[i].total_distance = total_distance;
    }

    Ok((track_origin, ref_points))
}


/*
    Converts a vector of @spatial_points from one coordinate space to another given a projection matrix given by @config and a transform function (@transform_fn) for the new output format
    Throws: CoordinateConversionError if any of the initial points can not be converted to the new space
*/
pub fn convert_to_space<P, R>(
    spatial_points: &[P],
    config: &CoordinatesConfig,
    transform_fn: impl Fn((f64, f64), &P) -> R,
) -> Result<Vec<R>, ServiceError>
where
    P: Send + Copy + Into<(f64, f64)> + std::fmt::Debug,
    R: Send,
{   

    let projection = proj::Proj::new_known_crs(
                    config.get_origin_space(),
                    config.get_destination_space(),
                    None,
                )
                .expect("Failed to initialize projection");
            
    spatial_points
        .iter()
        .map(
            |&point| {
                let (x, y) = point.into();
                let coord = Coord::from_xy(x, y);

                projection.convert(coord)
                    .map(|coords| transform_fn(coords, &point))
                    .map_err(|e| {
                        ServiceError::coordinate_conversion(
                            config.get_origin_space().as_str(),
                            config.get_destination_space().as_str(),
                            format!("{:?}", point).as_str(),
                            e.to_string().as_str(),
                        )
                    })
            },
        )
        .collect::<Result<Vec<_>, _>>()
}

pub fn get_track_origin<P>(
    spatial_points: &[P],
    config: &CoordinatesConfig,
    transform_fn: impl Fn((f64, f64), &P) -> TrackOrigin,
) -> Result<TrackOrigin, ServiceError> 
where
    P: Copy + Into<(f64, f64)> + std::fmt::Debug,
{

    let point = *spatial_points.get(0).clone().ok_or(ServiceError::empty_track())?; 
    let (x, y)= point.into();
    
    let coord = Coord::from_xy(x, y);

    let projection = proj::Proj::new_known_crs(
                    config.get_origin_space(),
                    config.get_destination_space(),
                    None,
                )
                .expect("Failed to initialize projection");

    projection.convert(coord)
                    .map(|coords| transform_fn(coords, &point))
                    .map_err(|e| {
                        ServiceError::coordinate_conversion(
                            config.get_origin_space().as_str(),
                            config.get_destination_space().as_str(),
                            format!("{:?}", point).as_str(),
                            e.to_string().as_str(),
                        )
                    })
}