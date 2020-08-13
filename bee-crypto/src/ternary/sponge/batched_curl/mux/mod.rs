pub mod demultiplexer;
pub mod multiplexer;

use std::ops::Range;

#[derive(Clone)]
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

    pub fn get<'a>(&'a self, range: Range<usize>) -> BCTritSlice<'a> {
        BCTritSlice {
            lo: &self.lo[range.clone()],
            hi: &self.hi[range],
        }
    }

    pub fn get_mut<'a>(&'a mut self, range: Range<usize>) -> BCTritSliceMut<'a> {
        BCTritSliceMut {
            lo: &mut self.lo[range.clone()],
            hi: &mut self.hi[range],
        }
    }

    pub fn lo_mut(&mut self) -> &mut [usize] {
        &mut self.lo
    }

    pub fn hi_mut(&mut self) -> &mut [usize] {
        &mut self.hi
    }
}

pub struct BCTritSlice<'a> {
    lo: &'a [usize],
    hi: &'a [usize],
}

pub struct BCTritSliceMut<'a> {
    lo: &'a mut [usize],
    hi: &'a mut [usize],
}

impl<'a> BCTritSliceMut<'a> {
    pub fn copy_from_slice(&mut self, slice: BCTritSlice) {
        self.lo.copy_from_slice(slice.lo);
        self.hi.copy_from_slice(slice.hi);
    }
}
