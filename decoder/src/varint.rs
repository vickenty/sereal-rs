use std::io;
use std::result;
use byteorder::ReadBytesExt;

#[derive(Debug)]
pub enum Error {
    Overflow,
    IOError(io::Error),
}

impl Error {
    pub fn is_overflow(&self) -> bool {
        match self {
            &Error::Overflow => true,
            _ => false,
        }
    }

    pub fn is_io_error(&self) -> bool {
        match self {
            &Error::IOError(_) => true,
            _ => false,
        }
    }

    pub fn is_eof(&self) -> bool {
        match self {
            &Error::IOError(ref e) => e.kind() == io::ErrorKind::UnexpectedEof,
            _ => false,
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IOError(e)
    }
}

type Result<T> = result::Result<T, Error>;

pub trait VarintReaderExt {
    fn read_varint(&mut self) -> Result<u64>;
    fn read_zigzag(&mut self) -> Result<i64> {
        let v = self.read_varint()?;
        Ok((v >> 1) as i64 ^ -((v & 1) as i64))
    }
}

impl<R: io::Read> VarintReaderExt for R {
    fn read_varint(&mut self) -> Result<u64> {
        let mut a = 0;
        let mut o = 0;

        loop {
            let i = self.read_u8()?;
            a |= ((i & 0x7f) as u64) << o;

            if i & 0x80 == 0 {
                return Ok(a);
            }

            o += 7;
            if o >= 64 {
                return Err(Error::Overflow);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;
    use super::{Error, Result, VarintReaderExt};

    #[test]
    fn test_varint() {
        use std::u64::MAX;

        fn r(s: &[u8]) -> Result<u64> {
            Cursor::new(s).read_varint()
        }

        fn t(s: &[u8]) -> u64 {
            r(s).unwrap()
        }

        fn e(s: &[u8]) -> Error {
            r(s).unwrap_err()
        }

        assert_eq!(t(b"\x00"), 0);
        assert_eq!(t(b"\x01"), 1);
        assert_eq!(t(b"\x80\x01"), 128);
        assert_eq!(t(b"\x80\x80\x01"), 16384);
        assert_eq!(t(b"\x81\x01"), 129);
        assert_eq!(t(b"\x81\x81\x00"), 129);
        assert_eq!(t(b"\x80\x80\x80\x80\x80\x80\x80\x80\x80\x00"), 0);
        assert_eq!(t(b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\x7f"), MAX);

        assert!(e(b"\x80").is_eof());
        assert!(e(b"\x80\x80\x80\x80\x80\x80\x80\x80\x80\x80\x00").is_overflow());
    }

    #[test]
    fn test_zigzag() {
        use std::i64::{MIN, MAX};

        fn r(s: &[u8]) -> Result<i64> {
            Cursor::new(s).read_zigzag()
        }

        fn t(s: &[u8]) -> i64 {
            r(s).unwrap()
        }

        fn e(s: &[u8]) -> Error {
            r(s).unwrap_err()
        }

        assert_eq!(t(b"\x00"), 0);
        assert_eq!(t(b"\x01"), -1);
        assert_eq!(t(b"\x02"), 1);
        assert_eq!(t(b"\x03"), -2);
        assert_eq!(t(b"\x04"), 2);
        assert_eq!(t(b"\x80\x01"), 64);
        assert_eq!(t(b"\x80\x80\x01"), 8192);
        assert_eq!(t(b"\x81\x01"), -65);
        assert_eq!(t(b"\x81\x81\x00"), -65);
        assert_eq!(t(b"\x80\x80\x80\x80\x80\x80\x80\x80\x80\x00"), 0);
        assert_eq!(t(b"\xfe\xff\xff\xff\xff\xff\xff\xff\xff\x7f"), MAX);
        assert_eq!(t(b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\x7f"), MIN);

        assert!(e(b"\x80").is_eof());
        assert!(e(b"\x80\x80\x80\x80\x80\x80\x80\x80\x80\x80\x00").is_overflow());
    }
}
