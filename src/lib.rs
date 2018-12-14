#![no_std]

extern crate byteorder;
use byteorder::{ByteOrder, LittleEndian};

mod error;
pub use error::{MbrError, ErrorCause};

mod partitions;
pub use partitions::*;


pub struct MasterBootRecord {
    entries: [PartitionTableEntry; MAX_ENTRIES],
}

const BUFFER_SIZE: usize = 512;
const TABLE_OFFSET: usize = 446;
const ENTRY_SIZE: usize = 16;
const SUFFIX_BYTES: [u8; 2] = [0x55, 0xaa];
const MAX_ENTRIES: usize = (BUFFER_SIZE - TABLE_OFFSET - 2) / ENTRY_SIZE;

impl MasterBootRecord {
    pub fn from_bytes<T: AsRef<[u8]>>(bytes: &T) -> Result<MasterBootRecord, MbrError> {
        let buffer: &[u8] = bytes.as_ref();
        if buffer.len() < BUFFER_SIZE {
            return Err(MbrError::from_cause(ErrorCause::BufferWrongSizeError{expected : BUFFER_SIZE, actual : buffer.len()}));
        } else if buffer[BUFFER_SIZE - SUFFIX_BYTES.len()..BUFFER_SIZE] != SUFFIX_BYTES[..] {
            return Err(MbrError::from_cause(ErrorCause::InvalidMBRSuffix{actual : [buffer[BUFFER_SIZE - 2], buffer[BUFFER_SIZE -1]]}));
        }
        let mut entries = [PartitionTableEntry::empty(); MAX_ENTRIES];
        for idx in 0..MAX_ENTRIES {
            let offset = TABLE_OFFSET + idx * ENTRY_SIZE;
            let partition_type = PartitionType::from_mbr_tag_byte(buffer[offset + 4]);
            if let PartitionType::Unknown(c) = partition_type {
                return Err(MbrError::from_cause(ErrorCause::UnsupportedPartitionError { tag : c}));
            }
            let lba = LittleEndian::read_u32(&buffer[offset + 8..]);
            let len = LittleEndian::read_u32(&buffer[offset + 12..]);
            entries[idx] = PartitionTableEntry::new(partition_type, lba, len);
        }
        Ok(MasterBootRecord { entries })
    }

    pub fn serialize<T: AsMut<[u8]>>(&self, buffer: &mut T) -> Result<usize, MbrError> {
        let buffer: &mut [u8] = buffer.as_mut();
        if buffer.len() < BUFFER_SIZE {
            return Err(MbrError::from_cause(ErrorCause::BufferWrongSizeError{expected : BUFFER_SIZE, actual : buffer.len()}));
        }
        {
            let suffix: &mut [u8] = &mut buffer[BUFFER_SIZE - SUFFIX_BYTES.len()..BUFFER_SIZE];
            suffix.copy_from_slice(&SUFFIX_BYTES);
        }
        for idx in 0..MAX_ENTRIES {
            let offset = TABLE_OFFSET + idx * ENTRY_SIZE;
            let entry = self.entries[idx];
            let parition_byte = entry.partition_type.to_mbr_tag_byte();
            let lba = entry.logical_block_address;
            let len = entry.sector_count;

            buffer[offset + 4] = parition_byte;
            {
                let lba_slice: &mut [u8] = &mut buffer[offset + 8..offset + 12];
                LittleEndian::write_u32(lba_slice, lba);
            }
            {
                let len_slice: &mut [u8] = &mut buffer[offset + 12..offset + 16];
                LittleEndian::write_u32(len_slice, len);
            }
        }
        Ok(BUFFER_SIZE)
    }
}

impl PartitionTable for MasterBootRecord {
    fn size(&self) -> usize {
        BUFFER_SIZE
    }

    fn partition_table_entries(&self) -> &[PartitionTableEntry] {
        &self.entries[..]
    }
}
