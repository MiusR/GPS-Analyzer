#[derive(Clone, Debug)]
pub struct AnalysisConfig {
    directional_deviance : f32,      // How close must the travel direction of any matched point be to the reference track (0...1) 
    allowed_deviance : f32,          // Allowed lateral distance from a track reference point for which no penalty is applied
    incremental_severity: f32,       // Severity step, every *incremental_severity* meters after the allowed_deviance the severity of the deviation increases
    minimum_continuous_error : usize // Minimum number of continuous "deviations" for it to actually be marked 
}

impl AnalysisConfig {
    
    pub fn new(directional_deviance : f32, allowed_deviance : f32, incremental_severity: f32, minimum_continuous_error : usize) -> Self {
        AnalysisConfig { 
            directional_deviance : directional_deviance,
            allowed_deviance : allowed_deviance,
            incremental_severity : incremental_severity,
            minimum_continuous_error : minimum_continuous_error
        }
    }

    pub fn get_directional_deviance(&self) -> f32 {
        self.directional_deviance
    }

        pub fn get_allowed_deviance(&self) -> f32 {
        self.allowed_deviance
    }

    pub fn get_incremental_severity(&self) -> f32 {
        self.incremental_severity
    }

    pub fn get_minimum_cont_error(&self) -> usize {
        self.minimum_continuous_error
    }
}