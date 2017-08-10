use std::io;
use std::result;
use byteorder::ReadBytesExt;

#[derive(Debug)]
pub enum Error {
    Overflow,
    UnexpectedEof,
}

impl Error {
    pub fn is_overflow(&self) -> bool {
        match self {
            &Error::Overflow => true,
            _ => false,
        }
    }

    pub fn is_eof(&self) -> bool {
        match self {
            &Error::UnexpectedEof => true,
            _ => false,
        }
    }
}

type Result<T> = result::Result<T, Error>;

fn straighten(v: u64) -> i64 {
    (v >> 1) as i64 ^ -((v & 1) as i64)
}

pub trait VarintReaderExt {
    fn read_varint(&mut self) -> io::Result<u64>;
    fn read_zigzag(&mut self) -> io::Result<i64> {
        let v = self.read_varint()?;
        Ok(straighten(v))
    }
}

impl<R: io::Read> VarintReaderExt for R {
    fn read_varint(&mut self) -> io::Result<u64> {
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
                return Err(io::Error::new(io::ErrorKind::Other, "varint overflow"));
            }
        }
    }
}

pub fn parse_varint(buf: &[u8]) -> Result<(u64, usize)> {
    let mut a = 0;
    let mut o = 0;

    for i in buf {
        a |= ((i & 0x7f) as u64) << (o * 7);
        o += 1;

        if i & 0x80 == 0 {
            return Ok((a, o));
        }

        if o >= 10 {
            return Err(Error::Overflow);
        }
    }

    Err(Error::UnexpectedEof)
}

pub fn parse_zigzag(buf: &[u8]) -> Result<(i64, usize)> {
    let (val, len) = parse_varint(buf)?;
    Ok((straighten(val), len))
}

#[cfg(test)]
mod test {
    use std::io;
    use std::io::Cursor;
    use super::{Error, Result, VarintReaderExt};

    #[test]
    fn test_varint() {
        use std::u64::MAX;

        fn r(s: &[u8]) -> io::Result<u64> {
            Cursor::new(s).read_varint()
        }

        fn t(s: &[u8]) -> u64 {
            r(s).unwrap()
        }

        fn e(s: &[u8]) -> io::Error {
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

        assert_eq!(e(b"\x80").kind(), io::ErrorKind::UnexpectedEof);
        assert_eq!(format!("{}", e(b"\x80\x80\x80\x80\x80\x80\x80\x80\x80\x80\x00")), "varint overflow");
    }

    #[test]
    fn test_zigzag() {
        use std::i64::{MIN, MAX};

        fn r(s: &[u8]) -> io::Result<i64> {
            Cursor::new(s).read_zigzag()
        }

        fn t(s: &[u8]) -> i64 {
            r(s).unwrap()
        }

        fn e(s: &[u8]) -> io::Error {
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

        assert_eq!(e(b"\x80").kind(), io::ErrorKind::UnexpectedEof);
        assert_eq!(format!("{}", e(b"\x80\x80\x80\x80\x80\x80\x80\x80\x80\x80\x00")), "varint overflow");
    }
}
