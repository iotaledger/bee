pub mod demultiplexer;
pub mod multiplexer;

pub struct BCTrits {
    pub lo: Vec<usize>,
    pub hi: Vec<usize>,
}

pub struct BCTrit {
    lo: usize,
    hi: usize,
}
