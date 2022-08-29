use bincode;

pub type Elf64Half = u16;
pub type Elf64Word = u32;
pub type Elf64Xword = u64;
pub type Elf64Addr = u64;
// File offsets
pub type Elf64Off = u64;

#[derive(bincode::Decode, bincode::Encode)]
#[repr(C)]
pub struct Ehdr64 {
    pub e_ident: [u8; 16],
    pub e_type: Elf64Half,
    pub e_machine: Elf64Half,
    pub e_version: Elf64Word,
    pub e_entry: Elf64Addr,
    pub e_phoff: Elf64Off,
    pub e_shoff: Elf64Off,
    pub e_flags: Elf64Word,
    pub e_ehsize: Elf64Half,
    pub e_phentsize: Elf64Half,
    pub e_phnum: Elf64Half,
    pub e_shentsize: Elf64Half,
    pub e_shnum: Elf64Half,
    pub e_shstrndx: Elf64Half,
}

impl Ehdr64 {
    pub fn deserialize(buf: &[u8], start: usize) -> Result<Self, bincode::error::DecodeError> {
        match bincode::decode_from_slice::<Self, _>(&buf[start..], bincode::config::standard()) {
            Ok((header, _)) => Ok(header),
            Err(e) => Err(e),
        }
    }

    pub fn addr_range(&self) -> (u64, u64) {
        // TODO: 実装する
        (0, 0)
    }
}

#[derive(bincode::Decode, bincode::Encode)]
pub struct Phdr64 {
    /// Segment type
    pub p_type: Elf64Word,

    /// Segment flags
    pub p_flags: Elf64Word,

    /// Segment file offset
    pub p_offset: Elf64Off,

    /// Segment virtual address
    pub p_vaddr: Elf64Addr,

    /// Segment physical address
    pub p_paddr: Elf64Addr,

    /// Segment size in file
    pub p_filesz: Elf64Xword,

    /// Segment size in memory
    pub p_memsz: Elf64Xword,

    /// Segment alignment
    pub p_align: Elf64Xword,
}

impl Phdr64 {
    pub fn deserialize(buf: &[u8], start: usize) -> Result<Self, bincode::error::DecodeError> {
        match bincode::decode_from_slice::<Self, _>(&buf[start..], bincode::config::standard()) {
            Ok((header, _)) => Ok(header),
            Err(e) => Err(e),
        }
    }
}
