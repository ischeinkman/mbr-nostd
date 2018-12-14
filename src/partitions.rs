
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

impl PartitionType {
    pub fn from_mbr_tag_byte(tag: u8) -> PartitionType {
        match tag {
            0x0 => PartitionType::Unused,
            0x01 => PartitionType::Fat12(tag),
            0x04 | 0x06 | 0x0e => PartitionType::Fat16(tag),
            0x0b | 0x0c | 0x1b | 0x1c => PartitionType::Fat32(tag),
            0x83 => PartitionType::LinuxExt(tag),
            0x07 => PartitionType::NtfsExfat(tag),
            0xaf => PartitionType::HfsPlus(tag),
            _ => PartitionType::Unknown(tag),
        }
    }

    pub fn to_mbr_tag_byte(&self) -> u8 {
        match *self {
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
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct PartitionTableEntry {
    pub partition_type: PartitionType,
    pub logical_block_address: u32,
    pub sector_count: u32,
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