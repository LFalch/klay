use *;
use byteorder::{LE, BE};

pub enum AutoEndianReader<R> {
    Little(R),
    Big(R)
}

pub enum AutoEndianChars<R> {
    Little(Chars<LE, R>),
    Big(Chars<BE, R>)
}

pub enum AutoEndianShorts<R> {
    Little(Shorts<LE, R>),
    Big(Shorts<BE, R>)
}

pub enum AutoEndianLines<R> {
    Little(Lines<LE, R>),
    Big(Lines<BE, R>)
}

impl<R> AutoEndianReader<R> {
    pub fn new_little(inner: R) -> Self {
        AutoEndianReader::Little(inner)
    }
    pub fn new_big(inner: R) -> Self {
        AutoEndianReader::Big(inner)
    }
    pub fn is_little(&self) -> bool {
        match *self {
            AutoEndianReader::Little(_) => true,
            _ => false,
        }
    }
    pub fn is_big(&self) -> bool {
        match *self {
            AutoEndianReader::Big(_) => true,
            _ => false,
        }
    }
}

impl<R: Utf16ReadExt> AutoEndianReader<R> {
    /// Reads a utf16 to detect the endianness
    ///
    /// If value isn't a valid bom, an error is thrown
    pub fn new_auto_bom(mut inner: R) -> Result<Self, Error> {
        let bom = inner.read_u16::<LE>()?;
        println!("Bom: {:4x}", bom);
        match bom {
            0xfeff => Ok(AutoEndianReader::Little(inner)),
            0xfffe => Ok(AutoEndianReader::Big(inner)),
            _ => Ok(AutoEndianReader::Little(inner)),
            // _ => Err(Error::new(ErrorKind::InvalidData, "First character wasn't a bom"))
        }
    }
    pub fn read_u16(&mut self) -> Result<u16, Error> {
        match *self {
            AutoEndianReader::Little(ref mut r) => r.read_u16::<LE>(),
            AutoEndianReader::Big(ref mut r) => r.read_u16::<BE>(),
        }
    }
    pub fn shorts(self) -> AutoEndianShorts<R>
    where Self: Sized {
        match self {
            AutoEndianReader::Little(r) => AutoEndianShorts::Little(r.shorts::<LE>()),
            AutoEndianReader::Big(r) => AutoEndianShorts::Big(r.shorts::<BE>()),
        }
    }
    pub fn utf16_chars(self) -> AutoEndianChars<R>
    where Self: Sized {
        match self {
            AutoEndianReader::Little(r) => AutoEndianChars::Little(r.utf16_chars()),
            AutoEndianReader::Big(r) => AutoEndianChars::Big(r.utf16_chars()),
        }
    }
}

impl<R: Utf16ReadExt> AutoEndianReader<R> {
    pub fn read_utf16_line(&mut self, buf: &mut String) -> Result<usize, Error> {
        match *self {
            AutoEndianReader::Little(ref mut r) => r.read_utf16_line::<LE>(buf),
            AutoEndianReader::Big(ref mut r) => r.read_utf16_line::<BE>(buf),
        }
    }
    pub fn utf16_lines(self) -> AutoEndianLines<R>
    where Self: Sized {
        match self {
            AutoEndianReader::Little(r) => AutoEndianLines::Little(r.utf16_lines()),
            AutoEndianReader::Big(r) => AutoEndianLines::Big(r.utf16_lines()),
        }
    }
}

impl<R: Utf16ReadExt> Iterator for AutoEndianChars<R> {
    type Item = Result<char, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            AutoEndianChars::Little(ref mut r) => r.next(),
            AutoEndianChars::Big(ref mut r) => r.next(),
        }
    }
}

impl<R: Utf16ReadExt> Iterator for AutoEndianShorts<R> {
    type Item = Result<u16, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            AutoEndianShorts::Little(ref mut r) => r.next(),
            AutoEndianShorts::Big(ref mut r) => r.next(),
        }
    }
}

impl<R: Utf16ReadExt> Iterator for AutoEndianLines<R> {
    type Item = Result<String, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            AutoEndianLines::Little(ref mut r) => r.next(),
            AutoEndianLines::Big(ref mut r) => r.next(),
        }
    }
}
