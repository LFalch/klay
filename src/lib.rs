pub extern crate byteorder;

use std::io::{BufRead, Read, Error, ErrorKind};

use byteorder::{ByteOrder, LE};

pub trait Utf16Read: Read {
    fn read_u16(&mut self) -> Result<u16, Error> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(LE::read_u16(&buf))
    }
    fn shorts(self) -> Shorts<Self>
    where Self: Sized {
        Shorts(self)
    }
    fn utf16_chars(self) -> Chars<Self>
    where Self: Sized {
        Chars(self)
    }
}

pub trait Utf16BufRead: BufRead {
    fn read_utf16_line(&mut self, buf: &mut String) -> Result<usize, Error> {
        let mut len = 0;
        for c in self.utf16_chars() {
            match c {
                Ok(c) => {
                    buf.push(c);
                    len += 1;
                    if c == '\n' {
                        break
                    }
                }
                Err(e) => match e.kind() {
                    ErrorKind::Interrupted => continue,
                    _ => return Err(e),
                }
            }
        }
        Ok(len)
    }
    fn utf16_lines(self) -> Lines<Self>
    where Self: Sized {
        Lines(self)
    }
}

impl<T: Read> Utf16Read for T {}
impl<T: BufRead> Utf16BufRead for T {}

#[derive(Debug)]
pub struct Shorts<R>(R);
#[derive(Debug)]
pub struct Chars<R>(R);

impl<R: Utf16Read> Iterator for Shorts<R> {
    type Item = Result<u16, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.0.read_u16() {
                Ok(u) => break Some(Ok(u)),
                Err(e) => match e.kind() {
                    ErrorKind::Interrupted => (),
                    ErrorKind::UnexpectedEof => break None,
                    _ => break Some(Err(e)),
                }
            }
        }
    }
}

use std::char::decode_utf16;

impl<R: Utf16Read> Iterator for Chars<R> {
    type Item = Result<char, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        let first = match self.0.read_u16() {
            Ok(f) => f,
            Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => return None,
            Err(e) => return Some(Err(e))
        };
        match decode_utf16(Some(first)).next().unwrap() {
            Ok(c) => Some(Ok(c)),
            Err(_) => {
                let snd = match self.0.read_u16() {
                    Ok(f) => f,
                    Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => return None,
                    Err(e) => return Some(Err(e))
                };
                Some(decode_utf16(Some(first).into_iter().chain(Some(snd))).next().unwrap()
                    .map_err(|e| Error::new(ErrorKind::InvalidData, e)))
            }
        }
    }
}

#[derive(Debug)]
pub struct Lines<B>(B);

impl<B: Utf16BufRead> Iterator for Lines<B> {
    type Item = Result<String, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = String::new();
        match self.0.read_utf16_line(&mut buf) {
            Ok(0) => None,
            Ok(_n) => {
                if buf.ends_with("\n") {
                    buf.pop();
                    if buf.ends_with("\r") {
                        buf.pop();
                    }
                }
                Some(Ok(buf))
            }
            Err(e) => Some(Err(e))
        }
    }
}
