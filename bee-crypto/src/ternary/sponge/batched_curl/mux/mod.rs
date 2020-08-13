pub mod demultiplexer;
pub mod multiplexer;

pub struct BCTritBuf {
    lo: Vec<usize>,
    hi: Vec<usize>,
}

impl BCTritBuf {
    pub fn filled(value: usize, len: usize) -> Self {
        BCTritBuf {
            lo: vec![value; len],
            hi: vec![value; len],
        }
    }

    pub fn zeros(len: usize) -> Self {
        Self::filled(0, len)
    }

    pub fn len(&self) -> usize {
        self.lo.len()
    }

    pub fn lo(&self) -> &[usize] {
        &self.lo
    }

    pub fn hi(&self) -> &[usize] {
        &self.hi
    }

    pub fn lo_mut(&mut self) -> &mut [usize] {
        &mut self.lo
    }

    pub fn hi_mut(&mut self) -> &mut [usize] {
        &mut self.hi
    }

    pub fn copy_from_slices(&mut self, lo: &[usize], hi: &[usize]) {
        self.lo.copy_from_slice(lo);
        self.hi.copy_from_slice(hi);
    }
}
