use std::io;
use std::result;
use byteorder::{ LittleEndian, ReadBytesExt };
use sereal_common::constants::*;

use config::Config;
use varint;
use varint::VarintReaderExt;

#[derive(Debug)]
pub enum Error {
    InvalidMagic,
    InvalidVersion,
    InvalidType,
    SuffixTooLarge,
    IOError(io::Error),
}

impl Error {
    pub fn is_invalid_magic(&self) -> bool {
        match self {
            &Error::InvalidMagic => true,
            _ => false
        }
    }

    pub fn is_invalid_version(&self) -> bool {
        match self {
            &Error::InvalidVersion => true,
            _ => false,
        }
    }

    pub fn is_invalid_type(&self) -> bool {
        match self {
            &Error::InvalidType => true,
            _ => false,
        }
    }

    pub fn is_suffix_too_large(&self) -> bool {
        match self {
            &Error::SuffixTooLarge => true,
            _ => false,
        }
    }

    pub fn is_io_error(&self) -> bool {
        match self {
            &Error::IOError(_) => true,
            _ => false,
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IOError(e)
    }
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum DocumentType {
    Uncompressed,
    Snappy { compressed_size: u64 },
    ZLib { compressed_size: u64, uncompressed_size: u64 },
    ZStd { compressed_size: u64 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Header {
    doc_type: DocumentType,
    metadata: Option<Vec<u8>>,
}

impl Header {
    pub fn read<R: io::Read>(reader: &mut R, config: &Config) -> Result<Header> {
        let magic = match reader.read_u32::<LittleEndian>()? {
            m @ MAGIC_V1 => m,
            m @ MAGIC_V3 => m,
            _ => return Err(Error::InvalidMagic),
        };

        let version_type = reader.read_u8()?;

        let suffix_len = reader.read_varint()?;
        if suffix_len > config.max_suffix_len() {
            return Err(Error::SuffixTooLarge);
        }

        let meta = if suffix_len > 0 {
            let flags = reader.read_u8()?;
            let size = suffix_len - 1;

            // Assumes that sizeof(u64) >= sizeof(usize) for all platforms.
            if size > (usize::max_value() as u64) {
                return Err(Error::SuffixTooLarge);
            }

            let mut buffer = vec![0; size as usize];
            reader.read_exact(&mut buffer)?;

            if flags & OPT_USER_METADATA != 0 {
                Some(buffer)
            } else {
                None
            }
        } else {
            None
        };

        let proto = match version_type & 0xf {
            p @ PROTO_V2 if magic == MAGIC_V1 => p,
            p @ PROTO_V3 if magic == MAGIC_V3 => p,
            p @ PROTO_V4 if magic == MAGIC_V3 => p,
            _ => return Err(Error::InvalidVersion),
        };

        let doctype = match (version_type & 0xf0) >> 4 {
            TYPE_RAW => DocumentType::Uncompressed,

            TYPE_SNAPPY if proto >= PROTO_V2 => {
                DocumentType::Snappy {
                    compressed_size: reader.read_varint()?,
                }
            },

            TYPE_ZLIB if proto >= PROTO_V3 => {
                DocumentType::ZLib {
                    uncompressed_size: reader.read_varint()?,
                    compressed_size: reader.read_varint()?,
                }
            },

            TYPE_ZSTD if proto >= PROTO_V4 => {
                DocumentType::ZStd {
                    compressed_size: reader.read_varint()?
                }
            },

            _ => return Err(Error::InvalidType),
        };

        Ok(Header {
            doc_type: doctype,
            metadata: meta,
        })
    }

    pub fn document_type(&self) -> DocumentType {
        self.doc_type
    }

    pub fn user_metadata(&self) -> &Option<Vec<u8>> {
        &self.metadata
    }

    pub fn user_metadata_mut(&mut self) -> &mut Option<Vec<u8>> {
        &mut self.metadata
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;
    use config::Config;
    use super::{ Error, Result, Header };
    use super::DocumentType::*;

    fn r(s: &[u8]) -> Result<Header> {
        Header::read(&mut Cursor::new(s), &Config::default())
    }

    fn p(s: &[u8]) -> Header {
        r(s).unwrap()
    }

    fn e(s: &[u8]) -> Error {
        r(s).unwrap_err()
    }

    #[test]
    fn invalid() {
        // bad magic
        assert!(e(b"=mrl").is_invalid_magic());
        assert!(e(b"=srf").is_invalid_magic());

        // =srl allows version 2 only
        assert!(e(b"=srl\x00\x00").is_invalid_version());
        assert!(e(b"=srl\x01\x00").is_invalid_version());
        assert!(e(b"=srl\x03\x00").is_invalid_version());
        assert!(e(b"=srl\x04\x00").is_invalid_version());

        // =\xf3rl allows versions 3 and 4
        assert!(e(b"=\xf3rl\x00\x00").is_invalid_version());
        assert!(e(b"=\xf3rl\x01\x00").is_invalid_version());
        assert!(e(b"=\xf3rl\x02\x00").is_invalid_version());

        // doctype 1 is not allowed
        assert!(e(b"=srl\x12\x00").is_invalid_type());
        assert!(e(b"=\xf3rl\x13\x00").is_invalid_type());
        assert!(e(b"=\xf3rl\x14\x00").is_invalid_type());

        // doctype <= version
        assert!(e(b"=srl\x32\x00").is_invalid_type());
        assert!(e(b"=\xf3rl\x43\x00").is_invalid_type());
        assert!(e(b"=\xf3rl\x54\x00").is_invalid_type());
    }

    #[test]
    fn version2() {
        assert_eq!(p(b"=srl\x02\x00"), Header {
            doc_type: Uncompressed,
            metadata: None,
        });

        assert_eq!(p(b"=srl\x22\x02\x01\x00\x0a"), Header {
            doc_type: Snappy { compressed_size: 10 },
            metadata: Some(vec![ 0 ]),
        });
    }

    #[test]
    fn version3() {
        assert_eq!(p(b"=\xf3rl\x33\x02\x01\x00\x0a\x0b"), Header {
            doc_type: ZLib { uncompressed_size: 10, compressed_size: 11 },
            metadata: Some(vec![ 0 ]),
        });
    }

    #[test]
    fn version4() {
        assert_eq!(p(b"=\xf3rl\x04\x00"), Header {
            doc_type: Uncompressed,
            metadata: None,
        });

        assert_eq!(p(b"=\xf3rl\x44\x00\x0a"), Header {
            doc_type: ZStd { compressed_size: 10 },
            metadata: None,
        });
    }
}
