#![no_std]

extern crate byteorder;
use byteorder::{ByteOrder, LittleEndian};

mod error;
pub use error::{MbrError, ErrorCause};
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum PartitionType {
    Unused,
    Unknown(u8),
    Fat12(u8),
    Fat16(u8),
    Fat32(u8),
    LinuxExt(u8),
    HfsPlus(u8),
    ISO9660(u8),
    NtfsExfat(u8),
}

fn from_mbr_tag_byte(tag: u8) -> PartitionType {
    match tag {
        0x0 => PartitionType::Unused,
        0x0b | 0x0c | 0x1b | 0x1c => PartitionType::Fat32(tag),
        0x01 => PartitionType::Fat12(tag),
        0x04 | 0x06 | 0x0e => PartitionType::Fat16(tag),
        0x83 => PartitionType::LinuxExt(tag),
        0x07 => PartitionType::NtfsExfat(tag),
        0xaf => PartitionType::HfsPlus(tag),
        _ => PartitionType::Unknown(tag),
    }
}

fn to_mbr_tag_byte(ptype: PartitionType) -> u8 {
    match ptype {
        PartitionType::Unused => 0,
        PartitionType::Unknown(t) => t,
        PartitionType::Fat12(t) => t,
        PartitionType::Fat16(t) => t,
        PartitionType::Fat32(t) => t,
        PartitionType::LinuxExt(t) => t,
        PartitionType::HfsPlus(t) => t,
        PartitionType::ISO9660(t) => t,
        PartitionType::NtfsExfat(t) => t,
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct PartitionTableEntry {
    partition_type: PartitionType,
    logical_block_address: u32,
    sector_count: u32,
}

impl PartitionTableEntry {
    pub fn new(
        partition_type: PartitionType,
        logical_block_address: u32,
        sector_count: u32,
    ) -> PartitionTableEntry {
        PartitionTableEntry {
            partition_type,
            logical_block_address,
            sector_count,
        }
    }

    pub fn empty() -> PartitionTableEntry {
        PartitionTableEntry::new(PartitionType::Unused, 0, 0)
    }
}

pub trait PartitionTable {
    fn size(&self) -> usize;
    fn partition_table_entries(&self) -> &[PartitionTableEntry];
}

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
            let partition_type = from_mbr_tag_byte(buffer[offset + 4]);
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
            let parition_byte = to_mbr_tag_byte(entry.partition_type);
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
