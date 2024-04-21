use std::collections::BTreeMap;
use tentacli_packet::Segment;
use std::io::{BufRead, Error, ErrorKind, Read, Write};
use std::mem;
use bitflags::bitflags;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

#[derive(Segment)]
struct Test {
    a: u32,
    b: u32,
    #[depends_on(a, b)]
    c: u32,
    #[depends_on(a, b, c)]
    d: u32,
    e: u32,
    f: u32,
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

impl BinaryConverter for u32 {
    fn write_into(&self, buffer: &mut Vec<u8>) -> Result<(), Error> {
        buffer.write_u32::<LittleEndian>(*self)
    }

    fn read_from<R: BufRead>(mut reader: R, dependencies: Vec<u8>) -> Result<Self, Error> {
        println!("u32 DEPS: {:?}", dependencies);
        reader.read_u32::<LittleEndian>()
    }
}

trait ToBytes {
    fn to_bytes(&self) -> Vec<u8>;
}

impl ToBytes for i32 {
    fn to_bytes(&self) -> Vec<u8> {
        unsafe {
            let size = mem::size_of::<i32>();
            let ptr = self as *const i32 as *const u8;
            let mut bytes = Vec::with_capacity(size);
            for i in 0..size {
                bytes.push(*ptr.add(i));
            }
            bytes
        }
    }
}

impl ToBytes for u32 {
    fn to_bytes(&self) -> Vec<u8> {
        unsafe {
            let size = mem::size_of::<u32>();
            let ptr = self as *const u32 as *const u8;
            let mut bytes = Vec::with_capacity(size);
            for i in 0..size {
                bytes.push(*ptr.add(i));
            }
            bytes
        }
    }
}

impl ToBytes for f64 {
    fn to_bytes(&self) -> Vec<u8> {
        unsafe {
            let size = mem::size_of::<f64>();
            let ptr = self as *const f64 as *const u8;
            let mut bytes = Vec::with_capacity(size);
            for i in 0..size {
                bytes.push(*ptr.add(i));
            }
            bytes
        }
    }
}

impl ToBytes for bool {
    fn to_bytes(&self) -> Vec<u8> {
        if *self {
            vec![1u8]
        } else {
            vec![0u8]
        }
    }
}

impl<T: ToBytes> ToBytes for Vec<T> {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for item in self {
            bytes.extend_from_slice(&item.to_bytes());
        }
        bytes
    }
}

impl<K: ToBytes + Ord, V: ToBytes> ToBytes for BTreeMap<K, V> {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for (key, value) in self {
            bytes.extend_from_slice(&key.to_bytes());
            bytes.extend_from_slice(&value.to_bytes());
        }
        bytes
    }
}

bitflags! {
    struct Flags: u32 {
        const FLAG_A = 0b00000001;
        const FLAG_B = 0b00000010;
        const FLAG_C = 0b00000100;
    }
}

impl ToBytes for Flags {
    fn to_bytes(&self) -> Vec<u8> {
        self.bits().to_bytes()
    }
}

fn main() {
    let data: Vec<u32> = vec![1, 2, 3, 4, 5, 6];
    let data_u8: Vec<u8> = data.iter().flat_map(|&num| num.to_le_bytes().to_vec()).collect();

    Test::from_binary(data_u8).test();
}
