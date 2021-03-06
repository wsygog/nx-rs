// Copyright © 2015-2018, Peter Atashian
//! Audio in NX files

/// Some audio, possibly a sound effect or music
#[derive(Clone, Copy)]
pub struct Audio<'a> {
    data: &'a [u8],
}
impl<'a> Audio<'a> {
    /// Creates an Audio from the supplied data
    pub unsafe fn construct(data: &'a [u8]) -> Audio<'a> {
        Audio { data: data }
    }
    /// Returns the audio data, not including the wz audio header
    pub fn data(&self) -> &[u8] {
        &self.data[82..]
    }
    /// Returns the wz audio header
    pub fn header(&self) -> &[u8; 82] {
        assert!(self.data.len() >= 82);
        unsafe { &*(self.data.as_ptr() as *const [u8; 82]) }
    }
}
