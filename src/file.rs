// Copyright © 2014, Peter Atashian
//! Stuff for working with NX files

use memmap::{Mmap};
use memmap::Protection::{Read};
use std::error::Error as StdError;
use std::fmt::{Display, Formatter};
use std::fmt::Error as FmtError;
use std::io::Error as IoError;
use std::mem::{size_of, transmute};
use std::path::{Path};
use std::result::{Result};
use std::slice::{from_raw_parts};

use repr::{self, Header};

pub use node::{Node};
pub use node::{GenericNode};
pub use node::{Type};

/// An error occuring anywhere in the library.
#[derive(Debug)]
pub enum Error {
    /// An internal IoError.
    Io(IoError),
    /// Magic value in header was incorrect.
    InvalidMagic,
    /// File was too short.
    TooShort,
}
impl StdError for Error {
    fn description(&self) -> &str {
        match self {
            &Error::Io(ref e) => e.description(),
            &Error::InvalidMagic => "Header magic value was invalid",
            &Error::TooShort => "File was too short for header",
        }
    }
    fn cause(&self) -> Option<&StdError> {
        match self {
            &Error::Io(ref e) => Some(e),
            _ => None,
        }
    }
}
impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        match self.cause() {
            Some(cause) => write!(fmt, "{} ({})", self.description(), cause),
            None => write!(fmt, "{}", self.description()),
        }
    }
}
impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Error::Io(err)
    }
}

/// A memory-mapped NX file.
pub struct File {
    #[allow(dead_code)]
    map: Mmap,
    data: *const u8,
    header: *const Header,
    nodetable: *const repr::Node,
    stringtable: *const u64,
    audiotable: *const u64,
}

impl File {
    /// Opens an NX file via memory-mapping. This also checks the magic bytes in the header.
    pub fn open(path: &Path) -> Result<File, Error> {
        let map = try!(Mmap::open_path(path, Read));
        if map.len() < size_of::<Header>() {
            return Err(Error::TooShort)
        }
        let data = map.ptr();
        let header = data as *const Header;
        if unsafe { (*header).magic } != 0x34474B50 {
            return Err(Error::InvalidMagic)
        }
        let nodetable = unsafe { data.offset((*header).nodeoffset as isize) as *const repr::Node };
        let stringtable = unsafe { data.offset((*header).stringoffset as isize) as *const u64 };
        let audiotable = unsafe { data.offset((*header).audiooffset as isize) as *const u64 };
        Ok(File {
            map: map,
            data: data,
            header: header,
            nodetable: nodetable,
            stringtable: stringtable,
            audiotable: audiotable,
        })
    }
    /// Gets the file header.
    #[inline]
    fn header(&self) -> &Header {
        unsafe { &*self.header }
    }
    /// Number of nodes in the file
    #[inline]
    pub fn node_count(&self) -> u32 {
        self.header().nodecount
    }
    /// Gets the root node of the file.
    #[inline]
    pub fn root<'a>(&'a self) -> Node<'a> {
        unsafe { Node::construct(&*self.nodetable, self) }
    }
    /// Gets the string at the specified index in the string table.
    #[inline]
    pub unsafe fn get_str(&self, index: u32) -> &str {
        let off = *self.stringtable.offset(index as isize);
        let ptr = self.data.offset(off as isize);
        let size = ptr as *const u16;
        transmute(from_raw_parts(ptr.offset(2), (*size) as usize))
    }
    /// Gets the node data at the specified index in the node table.
    #[inline]
    pub unsafe fn get_node(&self, index: u32) -> &repr::Node {
        &*self.nodetable.offset(index as isize)
    }
    /// Gets the audio data at the specified index in the node table.
    #[inline]
    pub unsafe fn get_audio(&self, index: u32, length: u32) -> &[u8] {
        let off = *self.audiotable.offset(index as isize);
        let ptr = self.data.offset(off as isize);
        transmute(from_raw_parts(ptr, length as usize))
    }
}
unsafe impl Send for File {}
unsafe impl Sync for File {}
