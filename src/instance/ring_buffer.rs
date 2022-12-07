use std::mem::MaybeUninit;

use bincode::error::DecodeError;
use bytes::buf::UninitSlice;

const MAX: usize = u16::MAX as usize;

#[derive(Debug)]
pub(crate) struct RingBuffer {
    start: u16,
    stop: u16,
    data: Box<[MaybeUninit<u8>; MAX + 1]>,
    back: u16,
}

impl RingBuffer{
    pub fn new() -> Self {
        Self { start: 0, stop: 0, data: Box::new([MaybeUninit::uninit(); MAX + 1]), back: 0 }
    }

    pub fn back(&mut self) {
        self.start = self.back;
    }

    pub fn fwd(&mut self) {
        self.back = self.start;
    }

    pub fn clear(&mut self) {
        //option: return illegal buffer content
        self.start = 0; self.stop = 0; self.back = 0;
    }
}

unsafe impl bytes::buf::BufMut for RingBuffer {
    fn remaining_mut(&self) -> usize {
        (u16::MAX - self.start.wrapping_sub(self.stop)) as usize
    }

    unsafe fn advance_mut(&mut self, cnt: usize) {
        self.stop = self.stop.wrapping_add(cnt as u16);
    }
    
    fn chunk_mut(&mut self) -> &mut UninitSlice {
        let end = 
        if self.start > self.stop { self.start }
        else { u16::MAX }
        as usize;

        let ptr = unsafe {self.data[self.stop as usize].assume_init_mut()};

        unsafe{ UninitSlice::from_raw_parts_mut(ptr, end - self.stop as usize) }
    }
}

impl bincode::de::read::Reader for RingBuffer{
    fn read(&mut self, bytes: &mut [u8]) -> Result<(), bincode::error::DecodeError> {
        if (self.stop.wrapping_sub(self.start) as usize) < bytes.len() {
            return Err(DecodeError::UnexpectedEnd{additional: bytes.len() - (self.stop.wrapping_sub(self.start) as usize) })
        }
        for i in bytes{
            *i = unsafe {self.data[self.start as usize].assume_init()};
            self.start = self.start.wrapping_add(1);
        }
        Ok(())
    }
}