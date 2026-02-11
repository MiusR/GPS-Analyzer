use std::ops::{Add, Mul, Sub};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use wide::f32x8;

use crate::model::{config::snapping::SnappingConfig, spatial::{grid::Grid, points::{MatchPoint, Point}}, track::{reference::ReferenceTrack, riders::{MatchedTrack, RiderTrack}}};

/*
    Snaps given point (px, py) to the closest reference point (refs).
    The refs are considered to be a slice of a bigger grid, as such indices are used to map to the original space.
    
    Dev Notes : uses smid, might break on different architectures but we get a small speed increase in snapping

    px - x component of given point
    py - y component of given point
    refs - reference points
    indices - global indices of reference points
*/
#[inline(always)]
pub fn min_distance<T : Point>(
    px : f32,
    py : f32,
    refs : &[T],
    indices : &[u32]
) -> (f32, u32) {
    let mut best_d2 = f32::MAX;
    let mut best_idx = 0u32; 

    
    // Hot loop - smid
    let pxv = f32x8::splat(px);
    let pyv = f32x8::splat(py);

    let mut i = 0;
    while i + 8 <= indices.len() {
        let mut rx = [0.0; 8];
        let mut ry = [0.0; 8];

        for j in 0..8 {
            let r = &refs[indices[i+j] as usize];
            rx[j] = r.x();
            ry[j] = r.y();
        }

        let rxv = f32x8::from(rx);
        let ryv = f32x8::from(ry);

        // Compute distance and select best match
        let dx = pxv.sub(rxv);
        let dy = pyv.sub(ryv);
        let d2v = dx.mul(dx).add(dy.mul(dy));

        let d2_arr  = d2v.to_array();
        for lane in 0..8 {
            let d2 = d2_arr[lane];
            if d2 < best_d2 {
                best_d2 = d2;
                best_idx = indices[i + lane];
            }
        }

        i+=8;
    }

    // Unbatched indices handling
    for &idx in &indices[i..] {
        let r = &refs[idx as usize];
        let dx = px - r.x();
        let dy = py - r.y();
        let d2 = dx * dx + dy * dy;
        if d2 < best_d2 {
            best_d2 = d2;
            best_idx = idx;
        }
    }

    (best_d2, best_idx)
}


/*
    Tries to snap a track to onto another track.
    rider - set of rider points
    refs - reference track
    grid - canonical uniform grid
    out - output parameter
    config - uses continuity_clamp - if a "matched point" is not within the clamp the warning of no known best clamp
*/
pub fn snap<T: Point, U : Point>(
    rider : &[T],
    refs : &[U],
    grid : &Grid,
    out : &mut Vec<MatchPoint>,
    config : &SnappingConfig
) {
    let mut neighbors = [0usize; 9];
    let mut last_reference: Option<u32> = None;
    let mut ref_direction_vec : (f32, f32) = (0.0, 0.0);
    let mut rider_direction_vec : (f32, f32) = (0.0, 0.0);
    
    for (ridx, rider_point) in rider.iter().enumerate() {
        let cell = grid.cell_index(rider_point.x(), rider_point.y());
        grid.neighbors(cell, &mut neighbors);

        let mut best_squared_distance = f32::MAX;
        let mut best_index = 0u32;
        let mut direction_similarity = 0.0;

        for &n in &neighbors {
            let grid_cell = &grid.cells[n];

            if grid_cell.count == 0 {
                continue;
            }

            let cell_indices = &grid.indices[grid_cell.start..grid_cell.start+grid_cell.count];
            let (squared_distance, idx) = min_distance(rider_point.x(),rider_point.y(), refs, cell_indices);

            if squared_distance < best_squared_distance {
                best_squared_distance = squared_distance;
                best_index = idx;
            }
        }

        let cc = config.get_continuity_clamp();
        if let Some(prev) = last_reference {
            if best_index + cc < prev {
                best_index = prev;
            }
        }

        last_reference = Some(best_index);

        // Compute directions 
        if ridx >= 1 && best_index >= 1 {
            ref_direction_vec.0  = refs[best_index as usize].x() - refs[best_index as usize - 1].x();
            ref_direction_vec.1 = refs[best_index as usize].y() - refs[best_index as usize - 1].y();
            rider_direction_vec.0  = rider[ridx].x() - rider[ridx - 1].x();
            rider_direction_vec.1 = rider[ridx].y() - rider[ridx - 1].y();

            // Dot product
            let dot_product = ref_direction_vec.0 * rider_direction_vec.0 + ref_direction_vec.1 * rider_direction_vec.1;
            // TODO find a more efficent way to compute the magnitude ?? reinvent math ?? 
            let comp_magnitude = ((ref_direction_vec.0 * ref_direction_vec.0 + ref_direction_vec.1 * ref_direction_vec.1) + 
                                (rider_direction_vec.0 * rider_direction_vec.0 + rider_direction_vec.1 * rider_direction_vec.1)).sqrt();

            direction_similarity  = dot_product / comp_magnitude;
        }

        let r = &refs[best_index as usize];
        out.push(
            MatchPoint { 
                reference_index: best_index, 
                delta_seconds: rider_point.delta_seconds(),
                direction_similarity : direction_similarity,
                lateral: best_squared_distance.sqrt(), 
                distance_z: rider_point.z() - r.z(),
                count_to_error : false
            }
        );
    }
}



/*
    Parralel snapping of multiple rider tracks to a single track.
*/
pub fn snap_all(
    riders : &[RiderTrack],
    refs : &ReferenceTrack,
    grid : &Grid,
    config : &SnappingConfig
) -> Vec<MatchedTrack> {
    riders.par_iter()
    .map(|rider| {
        let mut out = Vec::with_capacity(rider.track.len());
        snap(&rider.track, &refs.track, grid, &mut out, config);
        MatchedTrack { 
            bib: rider.bib,
            projection : refs.projection.clone(),
            variant: rider.variant,
            track_origin : rider.track_origin,
            start_time: rider.start_time,
            track: out
        }
    }).collect()
}