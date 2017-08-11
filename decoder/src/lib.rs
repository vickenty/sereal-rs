extern crate byteorder;
extern crate typed_arena;
extern crate sereal_common;
extern crate serde;

#[cfg(test)]
#[macro_use]
extern crate serde_derive;

#[cfg(feature = "comp-snappy")]
extern crate snap;
#[cfg(feature = "comp-zlib")]
extern crate flate2;
#[cfg(feature = "comp-zstd")]
extern crate zstd;

pub mod config;
pub mod header;
pub mod parser;
pub mod reader;

pub mod arc;
pub mod arena;
pub mod de;

mod varint;

use std::io;
use std::io::Read;

use config::Config;
use header::Header;
use header::DocumentType;
use parser::Parser;
use parser::Builder;

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    Header(header::Error),
    Parser(parser::Error),
    BodyTooLarge { size: u64, limit: u64 },
    UnsupportedType(DocumentType),
}

impl From<header::Error> for Error {
    fn from(e: header::Error) -> Error {
        Error::Header(e)
    }
}

impl From<parser::Error> for Error {
    fn from(e: parser::Error) -> Error {
        Error::Parser(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IO(e)
    }
}

#[cfg(feature = "comp-snappy")]
fn read_snappy_body<R: io::Read>(
    mut reader: R,
    comp_size: u64,
    buf: &mut Vec<u8>,
) -> io::Result<()> {
    let mut input = vec![0; comp_size as usize];
    reader.read_exact(&mut input)?;
    buf.resize(snap::decompress_len(&input)?, 0);
    let mut dec = snap::Decoder::new();
    dec.decompress(&input, buf)?;
    Ok(())
}

#[cfg(feature = "comp-zlib")]
fn read_zlib_body<R: io::Read>(
    reader: R,
    comp_size: u64,
    full_size: u64,
    buf: &mut Vec<u8>,
) -> io::Result<()> {
    let mut rdr = flate2::read::ZlibDecoder::new(reader.take(comp_size));
    buf.resize(full_size as usize, 0);
    rdr.read_exact(buf)?;
    Ok(())
}

#[cfg(feature = "comp-zstd")]
fn read_zstd_body<R: io::Read>(reader: R, comp_size: u64, buf: &mut Vec<u8>) -> io::Result<()> {
    let mut rdr = zstd::stream::Decoder::new(reader.take(comp_size))?;
    rdr.read_to_end(buf)?;
    Ok(())
}

pub fn parse<'buf, R, B>(
    mut reader: R,
    builder: B,
    buffer: &'buf mut Vec<u8>,
) -> Result<B::Value, Error>
where
    R: io::Read + io::Seek,
    B: Builder<'buf>,
{
    let config = Config::default();
    let header = Header::read(&mut reader, &config)?;

    #[allow(unreachable_patterns)]
    match header.document_type() {
        DocumentType::Uncompressed => {
            reader.read_to_end(buffer)?;
        }

        #[cfg(feature = "comp-snappy")]
        DocumentType::Snappy { compressed_size } => {
            if compressed_size > config.max_compressed_size() {
                return Err(Error::BodyTooLarge {
                    size: compressed_size,
                    limit: config.max_compressed_size(),
                });
            }
            read_snappy_body(reader, compressed_size, buffer)?
        }

        #[cfg(feature = "comp-zlib")]
        DocumentType::ZLib {
            compressed_size,
            uncompressed_size,
        } => {
            if compressed_size > config.max_compressed_size() {
                return Err(Error::BodyTooLarge {
                    size: compressed_size,
                    limit: config.max_compressed_size(),
                });
            }

            if uncompressed_size > config.max_uncompressed_size() {
                return Err(Error::BodyTooLarge {
                    size: uncompressed_size,
                    limit: config.max_uncompressed_size(),
                });
            }

            read_zlib_body(reader, compressed_size, uncompressed_size, buffer)?
        }

        #[cfg(feature = "comp-zstd")]
        DocumentType::ZStd { compressed_size } => {
            if compressed_size > config.max_compressed_size() {
                return Err(Error::BodyTooLarge {
                    size: compressed_size,
                    limit: config.max_compressed_size(),
                });
            }

            read_zstd_body(reader, compressed_size, buffer)?
        }

        ty => return Err(Error::UnsupportedType(ty)),
    };

    let mut parser = Parser::new(builder, &config, buffer);
    Ok(parser.parse()?)
}

#[cfg(test)]
mod test {
    use std::io::Cursor;
    use arc::ArcBuilder;
    use arc::Value;
    use arc::Inner;
    use parse;

    #[cfg(feature = "comp-snappy")]
    #[test]
    fn simple_snappy() {
        let raw = b"\
            \x3d\xf3\x72\x6c\x23\x00\xb8\x00\x84\x08\x10\x28\x2b\x80\x08\x00\
            \xfe\x01\x00\xfe\x01\x00\xfe\x01\x00\xfe\x01\x00\xfe\x01\x00\xfe\
            \x01\x00\xfe\x01\x00\xfe\x01\x00\xfe\x01\x00\xfe\x01\x00\xfe\x01\
            \x00\xfe\x01\x00\xfe\x01\x00\xfe\x01\x00\xfe\x01\x00\xfa\x01\x00\
        ";
        let val = parse(Cursor::new(&raw[..]), ArcBuilder, &mut Vec::new()).unwrap();
        assert_eq!(
            val,
            Value::new(Inner::Ref(Value::new(
                Inner::Array(vec![Value::new(Inner::U64(0)); 1024]),
            )))
        );
    }

    #[cfg(feature = "comp-zlib")]
    #[test]
    fn simple_zlib() {
        let raw = b"\
            \x3d\xf3\x72\x6c\x33\x00\x84\x08\x9d\x00\x78\x01\xed\xc0\x31\x0d\
            \x00\x00\x0c\x02\xc1\x8e\x95\x42\x82\x49\xa4\x23\x84\x3f\x39\x7f\
            \x00\x66\x15\x72\x5a\x00\xdc\
        ";
        let val = parse(Cursor::new(&raw[..]), ArcBuilder, &mut Vec::new()).unwrap();
        assert_eq!(
            val,
            Value::new(Inner::Ref(Value::new(
                Inner::Array(vec![Value::new(Inner::U64(0)); 1024]),
            )))
        );
    }
}
