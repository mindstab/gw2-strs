use bitvec::prelude::*;
use bytes::{Buf, Bytes};
use plain::{Error::*, Plain};
use std::char::{decode_utf16, from_u32, REPLACEMENT_CHARACTER};
use std::convert::TryInto;
use std::mem::size_of;

use string_dec::decrypt_content;

mod reader;

pub use reader::Reader;

static CHAR_TABLE: &'static [char; 32] = &[
    '0', '1', '2', '3', '4', '5', '6', 's', 't', 'r', 'n', 'u', 'm', '(', ')', '[', ']', '<', '>',
    '%', '#', '/', ':', '-', '\'', '"', ' ', ',', '.', '!', '\n', '\x00',
];

#[derive(Default, Debug, PartialEq)]
pub enum Language {
    #[default]
    English,
    Korean,
    French,
    German,
    Spanish,
    Chinese,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    HeaderTooShort,
    InvalidFile,
    LanguageNotSupported,
    StringIndexOutOfRange,
    NoEncryptionKeyProvided,
    Unexpected,
}

impl From<plain::Error> for Error {
    fn from(err: plain::Error) -> Error {
        match err {
            TooShort => Error::HeaderTooShort,
            _ => Error::InvalidFile,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[repr(C)]
#[derive(Default)]
struct Header {
    pub magic: u32,
}

unsafe impl Plain for Header {}

const MAGIC: u32 = 0x73727473;

impl Header {
    pub fn from_bytes(buf: &[u8]) -> Result<Self> {
        let mut hdr = Header::default();
        hdr.copy_from_bytes(buf)?;
        if hdr.magic != MAGIC {
            Err(Error::InvalidFile)
        } else {
            Ok(hdr)
        }
    }
}

#[repr(C)]
#[derive(Default)]
struct Trailer {
    pub language: u8,
    pub index: u8,
}

unsafe impl Plain for Trailer {}

impl Trailer {
    pub fn from_bytes(buf: &[u8]) -> Result<Self> {
        let mut trl = Trailer::default();
        trl.copy_from_bytes(buf)?;
        Ok(trl)
    }
}

#[repr(C)]
#[derive(Default)]
struct Entry {
    pub size: u16,
    pub offset: u16,
    pub bits_per_unit: u8,
}

unsafe impl Plain for Entry {}

impl Entry {
    pub fn from_bytes(buf: &[u8]) -> Result<Self> {
        let mut hdr = Entry::default();
        hdr.copy_from_bytes(buf)?;
        if hdr.bits_per_unit == 0 {
            return Err(Error::Unexpected);
        }
        if (hdr.size as usize) < size_of::<Entry>() {
            return Err(Error::InvalidFile);
        }
        Ok(hdr)
    }
}

struct StringData(Entry, Bytes);

impl StringData {
    pub fn get_string(&self, encryption_key: Option<u64>) -> Result<String> {
        match (self.0.offset, encryption_key) {
            (1..=u16::MAX, Some(encryption_key)) => self.get_encrypted_string(encryption_key),
            (1..=u16::MAX, None) => Err(Error::NoEncryptionKeyProvided),
            (0, _) => Ok(Self::get_raw_string(0, self.0.bits_per_unit, &self.1)),
        }
    }

    fn get_encrypted_string(&self, encryption_key: u64) -> Result<String> {
        let buffer = decrypt_content(self.1.chunk(), encryption_key);
        Ok(Self::get_raw_string(
            self.0.offset,
            self.0.bits_per_unit,
            &Bytes::from(buffer),
        ))
    }

    fn get_raw_string(offset: u16, bits_per_unit: u8, buffer: &Bytes) -> String {
        if offset != 0 || bits_per_unit != 16 {
            let bits = buffer.view_bits::<bitvec::order::Lsb0>();
            let len = buffer.len() * 8 / bits_per_unit as usize;
            (0..len)
                .map(|i| {
                    let x = i * bits_per_unit as usize;
                    let y = x + bits_per_unit as usize;
                    let ch = bits[x..y].load::<u32>();
                    match ch {
                        0 => '\0',
                        1..=31 => *CHAR_TABLE.get((ch - 1) as usize).unwrap(),
                        _ => from_u32(ch - 32 + offset as u32).unwrap_or(REPLACEMENT_CHARACTER),
                    }
                })
                .collect()
        } else {
            decode_utf16(
                buffer
                    .chunks_exact(2)
                    .map(|ch| u16::from_le_bytes(ch.try_into().unwrap())),
            )
            .map(|r| r.unwrap_or(REPLACEMENT_CHARACTER))
            .collect::<String>()
        }
    }
}
