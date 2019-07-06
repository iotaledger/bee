pub const NUM_MAX_ROUNDS: usize = 24;
pub const TROIKA_RATE: usize = 243;

pub const ROWS: usize = 3;
pub const COLUMNS: usize = 9;
pub const SLICES: usize = 27;

pub const SLICE_SIZE: usize = COLUMNS * ROWS;

#[cfg(feature = "troika")]
pub const STATE_SIZE: usize = COLUMNS * ROWS * SLICES;

#[cfg(feature = "troika")]
pub const NUM_SBOXES: usize = COLUMNS * ROWS * SLICES / 3;
