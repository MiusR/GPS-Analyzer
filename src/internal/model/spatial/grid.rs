use core::f32;
use std::{u32};

use glam::Vec2;

use crate::internal::model::track::reference::ReferenceTrack;



pub struct GridCell {
    pub start: usize,
    pub count: usize,
}

pub struct Grid {
    pub min: Vec2,
    pub inv_cell: f32,
    pub width: usize,
    pub height: usize,
    pub cells: Vec<GridCell>,
    pub indices: Vec<u32>,
}

impl Grid {
    // Build a grid from a reference track, cell_size is given in meters. A cell is cell_size * cell_size patch of real world space.
    pub fn from_track(ref_track : &ReferenceTrack, cell_size : f32) -> Self {
        // Find bounds of grid
        let first = ref_track.track.get(0).expect("Track cannot be empty"); // FIXME : why are we expecting on track in frid implementation? do you want to kill app randomly
        let mut min_x = first.x;
        let mut min_y = first.y;
        let mut max_x = first.x;
        let mut max_y = first.y;

        for point in ref_track.track.iter().skip(1) {
            min_x = min_x.min(point.x);
            min_y = min_y.min(point.y);
            max_x = max_x.max(point.x);
            max_y = max_y.max(point.y);
        }
                
        // Find dimensions of grid
        let cell_padding = 1;
        let width = ((max_x - min_x) / cell_size).ceil() as usize + cell_padding;
        let height = ((max_y - min_y) / cell_size).ceil() as usize + cell_padding;
        let cell_count = width * height;
        let inv_cell = 1.0 / cell_size;


        // Build cell buckets
        let mut buckets: Vec<Vec<u32>> = (0..cell_count)
        .map(|_| Vec::new())
        .collect();

        // Map point to bucket
        for (index, reference_point) in ref_track.track.iter().enumerate() {
            let mut cell_x = ((reference_point.x - min_x) * inv_cell) as usize;
            let mut cell_y = ((reference_point.y - min_y) * inv_cell) as usize;

            cell_x = cell_x.min(width  - 1);
            cell_y = cell_y.min(height - 1);

            let cell = cell_y * width + cell_x;
            buckets[cell].push(index as u32);
        }

        let mut cells = Vec::with_capacity(cell_count);
        let mut indices = Vec::new();

        let mut offset = 0usize;

        // Build cells from buckets
        for bucket in buckets {
            let count = bucket.len();

            cells.push(GridCell {
                start: offset,
                count,
            });

            indices.extend(bucket);
            offset += count;
        }



        Grid { 
            min: Vec2 { x: min_x, y: min_y }, 
            inv_cell, 
            width: width,
            height : height,
            cells: cells, 
            indices: indices 
        }
    }

    #[inline(always)]
    pub fn cell_index(&self, x: f32, y: f32) -> usize {
        let ix = (((x - self.min.x) * self.inv_cell) as isize).clamp(0, self.width as isize - 1) as usize;
        let iy = (((y - self.min.y) * self.inv_cell) as isize).clamp(0, self.height as isize - 1) as usize;
        (iy as usize) * self.width + (ix as usize)
    }

    #[inline(always)]
    pub fn neighbors(&self, cell: usize, out: &mut [usize; 9]) {
        let iw = self.width;
        let ih = self.cells.len() / iw;

        let x = (cell % iw) as isize;
        let y = (cell / iw) as isize;

        let mut i = 0;
        for neighbour_y in -1..=1 {
            for neighbour_x in -1..=1 {
                let nx = x + neighbour_x;
                let ny = y + neighbour_y;

                if nx >= 0 && nx < iw as isize && ny >= 0 && ny < ih as isize {
                    out[i] = ny as usize * iw + nx as usize;
                } else {
                    out[i] = cell; 
                }
                i += 1;
            }
        }
    }
}