use bincode;
use core::slice::from_raw_parts;

pub type Elf64Half = u16;
pub type Elf64Word = u32;
pub type Elf64Xword = u64;
pub type Elf64Addr = u64;
// File offsets
pub type Elf64Off = u64;

#[derive(bincode::Decode, bincode::Encode, Debug)]
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
    pub fn deserialize(addr: u64) -> Result<Self, bincode::error::DecodeError> {
        let buf = unsafe { from_raw_parts(addr as *const u8, core::mem::size_of::<Self>()) };
        match bincode::decode_from_slice::<Self, _>(buf, bincode::config::standard()) {
            Ok((header, _)) => Ok(header),
            Err(e) => Err(e),
        }
    }
}

pub fn addr_range(buf: &[u8]) -> (u64, u64) {
    let header = Ehdr64::deserialize(buf.as_ptr() as u64).unwrap();
    let phdr_addr = (buf.as_ptr() as u64) + header.e_phoff;
    let phdrs = unsafe { from_raw_parts(phdr_addr as *const Phdr64, header.e_phnum.into()) };

    let mut first = u64::max_value();
    let mut last = u64::min_value();
    for phdr in phdrs {
        first = core::cmp::min(first, phdr.p_vaddr);
        last = core::cmp::max(last, phdr.p_vaddr + phdr.p_memsz);
    }
    (first, last)
}

#[derive(bincode::Decode, bincode::Encode, Debug)]
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
    pub fn deserialize(addr: u64) -> Result<Self, bincode::error::DecodeError> {
        let buf = unsafe { from_raw_parts(addr as *const u8, core::mem::size_of::<Self>()) };
        match bincode::decode_from_slice::<Self, _>(buf, bincode::config::standard()) {
            Ok((header, _)) => Ok(header),
            Err(e) => Err(e),
        }
    }
}
