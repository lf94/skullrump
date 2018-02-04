//!
//! `skullrump` is a crate to quickly write UNIX-like binary head and tail routines.
//!
//! In order to make use of it, users implement the [`BinaryEntry`] trait for their types.
//!
//! File watching is not built in, but can be simulated with the `watch` program or similar.
//!
//! [`BinaryEntry`]: trait.BinaryEntry.html
//!
//! # Examples
//!
//! ```
//! use std::fs::File;
//! use std::io::{ Result };
//! use self::skullrump::byteorder::{ WriteBytesExt, ReadBytesExt };
//! use self::skullrump::{ BinaryEntry, BinaryChunkStream };
//!
//! struct ASingleByte(u8);
//!
//! impl BinaryEntry for ASingleByte {
//!  fn entry_write(data_in: Self, buffer_out: &mut Vec<u8>) -> Result<()> {
//!    buffer_out.write_u8(data_in.0)
//!  }
//!
//!  fn entry_read(file: &mut File) -> Result<Self> {
//!    file
//!      .read_u8()
//!      .and_then(|data| Ok(ASingleByte(data)))
//!  }
//!
//!  fn entry_size() -> i64 {
//!    1
//!  }
//! }
//!
//! fn foo(file: &mut File) {
//!   let mut buff:Vec<u8> = vec![];
//!
//!   file.entry_write(&mut buff, ASingleByte(1));
//!   match file.tail::<ASingleByte>(1) {
//!     Ok(_entries) => {}
//!     Err(_)      => {}
//!   };
//! }
//! ```
//!

mod skullrump;
pub extern crate byteorder;

pub use skullrump::{
  BinaryEntry,
  BinaryChunkStream,
  StreamFlow
};

#[cfg(test)]
mod tests {
  extern crate byteorder;

  use std::fs::{ 
    File,
    OpenOptions
  };

  use self::byteorder::{
    LittleEndian,
    ReadBytesExt
  };

  use std::io::{
    Result,
    Write
  };

  use skullrump::{
    BinaryChunkStream,
    BinaryEntry
  };

  struct CustomType(i64, f32);

  impl BinaryEntry for CustomType {
    fn entry_write(_data_in: Self, _buffer_out: &mut Vec<u8>) -> Result<()> {
      Ok(())
    }

    fn entry_read(file: &mut File) -> Result<Self> {
      let p1 = file.read_i64::<LittleEndian>().or::<Result<i64>>(Ok(0)).unwrap();
      let p2 = file.read_f32::<LittleEndian>().or::<Result<f32>>(Ok(0.0)).unwrap();

      return Ok(CustomType(p1, p2));
    }

    fn entry_size() -> i64 {
      (::std::mem::size_of::<i64>() + ::std::mem::size_of::<f32>()) as i64
    }
  }

  #[test]
  fn read_no_entries_forward() {
    let mut file = OpenOptions
      ::new()
      .write(true)
      .read(true)
      .create(true)
      .truncate(true)
      .open("test.bin")
      .unwrap();

    file.write_all(&[2u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8]).unwrap();
    file.flush().unwrap();
    assert_eq!(true, file.head::<i64>(0).unwrap().is_empty());
  }

  #[test]
  fn read_negative_n_entries_forward_return_empty() {
    let mut file = OpenOptions
      ::new()
      .write(true)
      .read(true)
      .create(true)
      .open("test.bin")
      .unwrap();

    file.write_all(&[1u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,2u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8]).unwrap();
    file.flush().unwrap();
    
    assert_eq!(true, file.head::<i64>(-2).unwrap().is_empty());
  }

  #[test]
  fn read_n_entries_forward() {
    let mut file = OpenOptions
      ::new()
      .write(true)
      .read(true)
      .create(true)
      .open("test.bin")
      .unwrap();

    file.write_all(&[1u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8]).unwrap();
    file.write_all(&[2u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8]).unwrap();
    file.flush().unwrap();
    
    let result = file.head::<i64>(2).unwrap();
    assert_eq!(1i64, *(result.get(0).unwrap()));
    assert_eq!(2i64, *(result.get(1).unwrap()));
  }

  #[test]
  fn read_one_entry_forward_past_the_end_returns_data() {
    let mut file = OpenOptions
      ::new()
      .write(true)
      .read(true)
      .create(true)
      .open("test.bin")
      .unwrap();

    file.write_all(&[1u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8]).unwrap();
    file.write_all(&[2u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8]).unwrap();
    file.flush().unwrap();
    
    let result = file.head::<i64>(3).unwrap();
    assert_eq!(1i64, *(result.get(0).unwrap()));
    assert_eq!(2i64, *(result.get(1).unwrap()));
  }

  #[test]
  fn read_no_entries_backward() {
    let mut file = OpenOptions
      ::new()
      .write(true)
      .read(true)
      .create(true)
      .open("test.bin")
      .unwrap();

    file.write_all(&[1u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,2u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8]).unwrap();
    file.flush().unwrap();
    
    assert_eq!(true, file.tail::<i64>(0).unwrap().is_empty());
  }

  #[test]
  fn read_n_entries_backward() {
    let mut file = OpenOptions
      ::new()
      .write(true)
      .read(true)
      .create(true)
      .open("test.bin")
      .unwrap();

    file.write_all(&[1u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,2u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8]).unwrap();
    file.flush().unwrap();
    
    let result = file.tail::<i64>(2).unwrap();
    assert_eq!(1i64, *(result.get(0).unwrap()));
    assert_eq!(2i64, *(result.get(1).unwrap()));
  }

  #[test]
  fn read_negative_n_entries_backward_return_empty() {
    let mut file = OpenOptions
      ::new()
      .write(true)
      .read(true)
      .create(true)
      .open("test.bin")
      .unwrap();

    file.write_all(&[1u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,2u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8]).unwrap();
    file.flush().unwrap();
    
    assert_eq!(true, file.tail::<i64>(-2).unwrap().is_empty());
  }

  #[test]
  fn read_1_entry_backward_past_the_end_returns_data() {
    let mut file = OpenOptions
      ::new()
      .write(true)
      .read(true)
      .create(true)
      .open("test.bin")
      .unwrap();

    file.write_all(&[1u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8,2u8,0u8,0u8,0u8,0u8,0u8,0u8,0u8]).unwrap();
    file.flush().unwrap();
    
    let result = file.tail::<i64>(3).unwrap();
    assert_eq!(1i64, *(result.get(0).unwrap()));
    assert_eq!(2i64, *(result.get(1).unwrap()));
  }
}
