use inkwell::OptimizationLevel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptLevel {
    O0,
    O1,
    O2,
    O3,
}

impl Default for OptLevel {
    fn default() -> Self {
        OptLevel::O2
    }
}

impl From<OptLevel> for OptimizationLevel {
    fn from(val: OptLevel) -> Self {
        match val {
            OptLevel::O0 => OptimizationLevel::None,
            OptLevel::O1 => OptimizationLevel::Less,
            OptLevel::O2 => OptimizationLevel::Default,
            OptLevel::O3 => OptimizationLevel::Aggressive,
        }
    }
}
