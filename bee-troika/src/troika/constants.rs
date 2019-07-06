use crate::types::Trit;

pub const PADDING: Trit = 1;

pub const STATE_SIZE: usize = COLUMNS * ROWS * SLICES;
pub const NUM_SBOXES: usize = COLUMNS * ROWS * SLICES / 3;

//pub const MUX_SIZE: usize = 8064;
//pub const STRIT_SIZE: usize = 64;
//pub const STRIT_BASE_SIZE: usize = 64;
