pub trait Point {
    fn x(&self) -> f32;
    fn y(&self) -> f32;
    fn z(&self) -> f32;
    fn delta_seconds(&self) -> f64;
}

#[derive(Clone, Copy, Debug)]
pub struct SpatialPoint {
    pub lon: f64,
    pub lat: f64,
    pub elev: Option<f64>,
    pub delta_seconds: Option<f64>
}

impl From<SpatialPoint> for (f64, f64) {
    fn from(sp: SpatialPoint) -> Self {
        (sp.lon, sp.lat)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RefPoint {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub total_distance: f32
}

#[derive(Clone, Copy, Debug)]
pub struct RiderPoint {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub delta_seconds : f64
}

#[derive(Clone, Copy, Debug)]
pub struct MatchPoint {
    pub reference_index: u32,
    pub delta_seconds: f64,
    pub direction_similarity : f32,
    pub lateral: f32,
    pub distance_z: f32,
    pub count_to_error : bool
}

impl Point for RefPoint {
    #[inline(always)]
    fn x(&self) -> f32 { self.x }

    #[inline(always)]
    fn y(&self) -> f32 { self.y }
    
    fn delta_seconds(&self) -> f64 {
        0.0
    }
    
    fn z(&self) -> f32 {
        self.z
    }
}

impl Point for RiderPoint {
    #[inline(always)]
    fn x(&self) -> f32 { self.x }

    #[inline(always)]
    fn y(&self) -> f32 { self.y }
    
    fn delta_seconds(&self) -> f64 {
        self.delta_seconds
    }
    
    fn z(&self) -> f32 {
        self.z
    }
}

