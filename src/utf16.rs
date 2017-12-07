use byteorder::{ReadBytesExt, LE};

pub trait ReadShort: ReadBytesExt {
    fn shorts(&mut self) -> Shorts<Self> {
        Shorts(self)
    }
}

impl<T: ::std::io::Read> ReadShort for T {
}

pub struct Shorts<'a, R: 'a + ReadBytesExt + ?Sized>(&'a mut R);

impl<'a, R: 'a + ReadBytesExt> Iterator for Shorts<'a, R> {
    type Item = u16;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.read_u16::<LE>().ok()
    }
}
