use tentacli_packet::Segment;
use std::io::{BufRead, Error, ErrorKind, Read, Write};
use byteorder::{ReadBytesExt, WriteBytesExt};

#[derive(Segment)]
struct Test {
    a: u8,
    b: u8,
    #[depends_on(a, b)]
    c: u8,
    #[depends_on(a, b, c)]
    d: u8,
    e: u8,
    f: u8,
}

pub trait BinaryConverter {
    fn write_into(&self, buffer: &mut Vec<u8>) -> Result<(), Error>;
    fn read_from<R: BufRead>(reader: R, dependencies: Vec<u8>) -> Result<Self, Error>
        where
            Self: Sized;
}

impl BinaryConverter for u8 {
    fn write_into(&self, buffer: &mut Vec<u8>) -> Result<(), Error> {
        buffer.write_u8(*self)
    }

    fn read_from<R: BufRead>(mut reader: R, dependencies: Vec<u8>) -> Result<Self, Error> {
        println!("DEPS: {:?}", dependencies);
        reader.read_u8()
    }
}

fn main() {
    let data = vec![1, 2, 3, 4, 5, 6];
    Test::from_binary(data).test();
}
