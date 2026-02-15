#[derive(Clone, Copy, Debug)]
pub struct SnappingConfig {
    continuity_clamp : u32,     // How many reference indices can we skip before we give a fragmented track warning
}


impl SnappingConfig {
    pub fn new(continuity_clamp : u32) -> Self {
        SnappingConfig {
            continuity_clamp : continuity_clamp,
        }
    }

    pub fn get_continuity_clamp(&self) -> u32 {
        self.continuity_clamp
    }
}