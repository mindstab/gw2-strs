use super::{Entry, Error, Header, Language, Result, StringData, Trailer};
use bytes::{Buf, Bytes};
use std::mem::size_of;

pub struct Reader {
    strings: Vec<StringData>,
    pub language: Language,
}

impl Reader {
    pub fn from(buffer: Bytes) -> Result<Self> {
        let _ = Header::from_bytes(buffer.chunk())?;

        let mut strings = Vec::with_capacity(1024);
        let length = buffer.len() - size_of::<Trailer>();
        let mut offset = size_of::<Header>();

        while offset < length {
            let entry = Entry::from_bytes(&buffer.chunk()[offset..])?;
            let string_offset = offset + size_of::<Entry>();
            offset += entry.size as usize;
            strings.push(StringData(entry, buffer.slice(string_offset..offset)));
        }

        let Trailer { language, .. } = Trailer::from_bytes(&buffer.chunk()[offset..])?;

        Ok(Self {
            strings,
            language: match language {
                0 => Ok(Language::English),
                1 => Ok(Language::Korean),
                2 => Ok(Language::French),
                3 => Ok(Language::German),
                4 => Ok(Language::Spanish),
                5 => Ok(Language::Chinese),
                _ => Err(Error::LanguageNotSupported),
            }?,
        })
    }

    pub fn get_string(&self, index: usize) -> Result<String> {
        match self.strings.get(index) {
            Some(data) => data.get_string(None),
            _ => Err(Error::StringIndexOutOfRange),
        }
    }

    pub fn get_encrypted_string(&self, index: usize, encryption_key: u64) -> Result<String> {
        match self.strings.get(index) {
            Some(data) => data.get_string(Some(encryption_key)),
            _ => Err(Error::StringIndexOutOfRange),
        }
    }
}
