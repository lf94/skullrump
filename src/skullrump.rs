extern crate byteorder;

use std::fs::{
  File
};
use std::io::{
  Write,
  Result,
  Seek,
  SeekFrom
};
use byteorder::{
  LittleEndian,
  ReadBytesExt,
  WriteBytesExt
};

#[allow(dead_code)]
/// The direction the stream is reading in.
pub enum StreamFlow {
  Forward,
  Backward
}

/// A single entry to be written to a binary file.
pub trait BinaryEntry: Sized {
  /// Write an entry to a reusable output buffer.
  fn entry_write(data_in: Self, buffer_out: &mut Vec<u8>) -> Result<()>;

  /// Read an entry from a file.
  fn entry_read(file: &mut File) -> Result<Self>;

  /// Get the size of an entry.
  ///
  /// Entries can vary in complexity, so it's necessary to implement this rather than magically calculate it.
  fn entry_size() -> i64;
}

/// A "stream" of incoming binary entries.
pub trait BinaryChunkStream: Write {
  /// Write a new binary entry from an output buffer to a file.
  fn entry_write<T: BinaryEntry>(&mut self, buffer_out: &mut Vec<u8>, data_in: T) -> Result<()> {
    T::entry_write(data_in, buffer_out)
      .and(self.write_all(&buffer_out))
      .and(self.flush())
      .and(Ok(buffer_out.clear()))
  }

  /// Specify the reading direction and number of entries to read, and return the list of entries.
  fn stream_in<T: BinaryEntry>(&mut self, direction: StreamFlow, until_entry: i64) -> Result<Vec<T>>;

  /// Read from the end of the file `until_entry` is reached.
  fn tail<T: BinaryEntry>(&mut self, until_entry: i64) -> Result<Vec<T>> {
    self.stream_in(StreamFlow::Backward, until_entry)
  }

  /// Read from the start of the file `until_entry` is reached.
  fn head<T: BinaryEntry>(&mut self, until_entry: i64) -> Result<Vec<T>> {
    self.stream_in(StreamFlow::Forward, until_entry)
  }
}

impl BinaryEntry for i64 {
  fn entry_write(data_in: Self, buffer_out: &mut Vec<u8>) -> Result<()> { buffer_out.write_i64::<LittleEndian>(data_in) }
  fn entry_read(file: &mut File) -> Result<Self> { file.read_i64::<LittleEndian>() }
  fn entry_size() -> i64 { ::std::mem::size_of::<Self>() as i64 }
}

impl BinaryEntry for f32 {
  fn entry_write(data_in: Self, buffer_out: &mut Vec<u8>) -> Result<()> { buffer_out.write_f32::<LittleEndian>(data_in) }
  fn entry_read(file: &mut File) -> Result<Self> { file.read_f32::<LittleEndian>() }
  fn entry_size() -> i64 { ::std::mem::size_of::<Self>() as i64 }
}

impl BinaryChunkStream for File {
  fn stream_in<T: BinaryEntry>(&mut self, direction: StreamFlow, until_entry: i64) -> Result<Vec<T>> {
    let mut entries: Vec<T> = vec![];
    let mut entry_index:i64 = 0;

    if until_entry <= 0 {
      return Ok(entries);
    }

    let data_size = T::entry_size();
    let tail_position = data_size * until_entry;

    let internal_direction = match direction {
      StreamFlow::Forward => { StreamFlow::Forward }
      StreamFlow::Backward => {
        if let Err(_) = self.seek(SeekFrom::End(-tail_position)) {
          StreamFlow::Forward
        } else {
          StreamFlow::Backward
        }
      }
    };

    loop {
      let position = data_size * entry_index;
      match internal_direction {
        StreamFlow::Forward  => {
          self.seek(SeekFrom::Start(position as u64))
        }
        StreamFlow::Backward => {
          self.seek(SeekFrom::End(-(tail_position - position)))
        }
      }.unwrap();

      match T::entry_read(self) {
        Ok(entry) => {
          entries.push(entry);
          entry_index += 1;
        }
        Err(_) => {
          break;
        }
      }

      if entry_index >= until_entry { break; }
    }

    return Ok(entries);
  }
}

