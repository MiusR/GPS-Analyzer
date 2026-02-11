use std::path::Path;
use std::time::Instant;

use crate::model::config::analysis::AnalysisConfig;
use crate::model::config::snapping::SnappingConfig;
use crate::model::spatial::grid::Grid;
use crate::service::track_processor::{process_rider_track, snap_rider_track};
use crate::{service::track_processor::process_reference_track};

pub mod model;
pub mod service;
pub mod io;

fn main() {
    // Transformer from EPSG:4326 - long latitude elevation
    // to EPSG:3844 Romania spatial coordinates 
    let mut before = Instant::now();
    let ref_track  = process_reference_track(Path::new("C:\\Users\\ratiu\\Desktop\\gold_track.gpx"), "gold", "EPSG:4326", "EPSG:3844").unwrap();
    let mut duration = before.elapsed();
    println!("Parsing ref time : {:?}", duration);
    
    before = Instant::now();
    let grid = Grid::from_track(&ref_track, 30.0);
    duration = before.elapsed();
    println!("Grid creation time : {:?}", duration);

    let snapping_config = SnappingConfig::new(1);
    let analysis_config = AnalysisConfig::new(1.0, 45.0, 5.0, 6);

    before = Instant::now();
    let rider_track  = process_rider_track(Path::new("C:\\Users\\ratiu\\Desktop\\4_1_1.gpx"),  "EPSG:4326", &ref_track.projection, &ref_track.origin).unwrap();
    duration = before.elapsed();
    println!("Parsing rider time : {:?}", duration);

    before = Instant::now();
    let mut matched_rider_track  = snap_rider_track(&rider_track, &ref_track, &grid, &snapping_config).unwrap();
    duration = before.elapsed();
    println!("Match rider time : {:?}", duration);
}
