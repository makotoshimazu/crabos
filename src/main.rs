#![no_std]
#![no_main]
#![feature(abi_efiapi)]

use core::fmt::Write;
use core::mem::{self, MaybeUninit};
use core::ptr::write_bytes;
use core::slice::{from_raw_parts, from_raw_parts_mut};
use uefi::data_types::Align;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::proto::console::serial::Serial;
use uefi::proto::console::text::Output;
use uefi::proto::media::file::{File, FileAttribute, FileInfo, FileMode};
use uefi::table::boot::{AllocateType, MemoryDescriptor, MemoryType, ScopedProtocol};
use uefi::CStr16;
use uefi::{prelude::*, Identify};
use uefi_services::system_table;

#[entry]
fn main(handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();

    writeln!(system_table.stdout(), "hello from bootloader :)").unwrap();

    unsafe {
        // System table: https://docs.rs/uefi/0.8.0/uefi/table/struct.SystemTable.html
        // `system_table` is cloned here to use both volume & system_table.stdout().
        let fs_sytem_table = system_table.unsafe_clone();
        let fs = fs_sytem_table
            .boot_services()
            .get_image_file_system(handle)
            .unwrap();
        let mut volume = (*fs.interface.get()).open_volume().unwrap();

        // CStr16: https://docs.rs/uefi/0.3.0/uefi/struct.CStr16.html
        // utf-8 -> utf-16
        let mut kernel_filename_buffer = [0; 7];
        let kernel_filename =
            CStr16::from_str_with_buf("kernel", &mut kernel_filename_buffer).unwrap();

        // Read `kernel`.
        let handle_result = volume.open(kernel_filename, FileMode::Read, FileAttribute::READ_ONLY);
        let file_handle = match handle_result {
            Ok(handle) => handle,
            Err(error) if error.status() == uefi::Status::NOT_FOUND => {
                writeln!(system_table.stdout(), "file not found.").unwrap();
                panic!("dame");
            }
            Err(_) => {
                writeln!(system_table.stdout(), "other error.").unwrap();
                panic!("dame");
            }
        };
        let mut file = file_handle.into_regular_file().unwrap();
        // TODO: allocate enough memory
        let mut file_info_buffer = [0; 1000];
        let file_info = file.get_info::<FileInfo>(&mut file_info_buffer).unwrap();

        const PAGE_UNIT_SIZE: usize = 0x1000;
        let kernel_file_size = file_info.file_size() as usize;
        let kernel_file_size_in_pages = size_in_pages(kernel_file_size);

        writeln!(
            system_table.stdout(),
            "kernel file size: {}",
            kernel_file_size
        )
        .unwrap();

        // 1. まず一回雑に全部読む
        let kernel_buffer = fs_sytem_table
            .boot_services()
            .allocate_pool(MemoryType::LOADER_DATA, kernel_file_size_in_pages)
            .unwrap();

        let buffer = from_raw_parts_mut(kernel_buffer, kernel_file_size_in_pages * PAGE_UNIT_SIZE);
        file.read(buffer).unwrap();

        // 2. ヘッダーをいい感じに読む
        let ehdr =
            core::mem::transmute::<*const u64, *const Elf64_Ehdr>(kernel_buffer as *const u64);

        let (kernel_first_addr, kernel_last_addr) = calc_load_address_range(ehdr).unwrap();
        writeln!(
            system_table.stdout(),
            "kernel first addr: {:?}\nkernel last addr: {:?}",
            kernel_first_addr as *const u64,
            kernel_last_addr as *const u64,
        )
        .unwrap();

        // 3. セクションごとに適切なアドレスにバッファを確保してコピーする
        let aligned_kernel_first_addr = align_page(kernel_first_addr as usize);
        let kernel_addr_size_in_pages =
            size_in_pages(kernel_last_addr as usize - aligned_kernel_first_addr);

        let addr = fs_sytem_table
            .boot_services()
            .allocate_pages(
                AllocateType::Address(aligned_kernel_first_addr),
                MemoryType::LOADER_DATA,
                kernel_addr_size_in_pages,
            )
            .unwrap();
        let buffer =
            from_raw_parts_mut(addr as *mut u8, kernel_addr_size_in_pages * PAGE_UNIT_SIZE);
        copy_load_segments(ehdr, buffer);

        // Reference: Mikan book p.79
        // TODO: Understand this magic...
        let ptr = (*ehdr).e_entry;
        writeln!(
            system_table.stdout(),
            "entry point addr: {:?}",
            ptr as *const u64
        )
        .unwrap();

        // ↑↑↑↑↑ mikan book list3.2

        let major = system_table.uefi_revision().major();
        let minor = system_table.uefi_revision().minor();
        writeln!(system_table.stdout(), "uefi revision: {}.{}", major, minor).unwrap();

        let (frame_buffer_ptr, frame_buffer_size) = {
            let handle = system_table
                .boot_services()
                .get_handle_for_protocol::<GraphicsOutput>()
                .unwrap();
            let mut scoped_protocol = system_table
                .boot_services()
                .open_protocol_exclusive::<GraphicsOutput>(handle)
                .unwrap();
            let ptr = scoped_protocol.frame_buffer().as_mut_ptr();
            let size = scoped_protocol.frame_buffer().size();
            (ptr, size)
        };
        writeln!(
            system_table.stdout(),
            "frame buffer size: {}",
            frame_buffer_size
        )
        .unwrap();

        // let frame_buffer = from_raw_parts_mut(frame_buffer_ptr, frame_buffer_size);
        // for (i, v) in frame_buffer.iter_mut().enumerate() {
        //     *v = (i % 256) as u8;
        // }

        let memory_map_size = system_table.boot_services().memory_map_size().map_size;
        writeln!(
            system_table.stdout(),
            "memory map size: {}",
            memory_map_size
        )
        .unwrap();
        let mut mmap_buf = [0u8; 10000];
        let mut mmap_buf_aligned = MemoryDescriptor::align_buf(&mut mmap_buf).unwrap();

        writeln!(
            system_table.stdout(),
            "MemoryDescriptor::alignment(): {:?}",
            MemoryDescriptor::alignment()
        )
        .unwrap();
        writeln!(
            system_table.stdout(),
            "mmap_buf_aligned: {:?}",
            mmap_buf_aligned.as_ptr()
        )
        .unwrap();

        system_table
            .exit_boot_services(handle, &mut mmap_buf_aligned)
            .unwrap();
        mem::forget(mmap_buf_aligned);

        let f = core::mem::transmute::<_, extern "C" fn(u64) -> core::ffi::c_void>(ptr);
        // f(frame_buffer_ptr, frame_buffer_size);
        f(10000);

        Status::SUCCESS
    }
}

const PAGE_UNIT_SIZE: usize = 0x1000;

fn align_page(addr: usize) -> usize {
    addr / PAGE_UNIT_SIZE * PAGE_UNIT_SIZE
}

fn size_in_pages(size_in_bytes: usize) -> usize {
    (size_in_bytes + PAGE_UNIT_SIZE - 1) / PAGE_UNIT_SIZE
}

#[allow(unused)]
/* Type for a 16-bit quantity.  */
/// Type for a 16-bit quantity (in ELF64)
pub type Elf64Half = u16;

/* Types for signed and unsigned 32-bit quantities.  */
/// Type for an unsigned 32-bit quantity (in ELF64)
pub type Elf64Word = u32;
/// Type for a signed 32-bit quantity (in ELF64)
pub type Elf64Sword = i32;

/* Types for signed and unsigned 64-bit quantities.  */
/// Type for an unsigned 64-bit quantity (in ELF32)
pub type Elf64Xword = u64;
/// Type for a signed 64-bit quantity (in ELF32)
pub type Elf64Sxword = i64;

/* Type of addresses.  */
/// Type of an address (in ELF64)
pub type Elf64Addr = u64;

/* Type of file offsets.  */
/// Type of a file offsets (in ELF64)
pub type Elf64Off = u64;

/* Type for section indices, which are 16-bit quantities.  */
/// Type of a file offsets (in ELF64)
pub type Elf64Section = u16;

/* Type for version symbol information.  */
/// Type of a version symbol information (in ELF64)
pub type Elf64Versym = Elf64Half;

#[repr(C)]
#[derive(Debug)]
pub struct Elf64_Ehdr {
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

#[repr(C)]
#[derive(Debug)]
pub struct Elf64_Phdr {
    pub p_type: Elf64Word,
    pub p_flags: Elf64Word,
    pub p_offset: Elf64Off,
    pub p_vaddr: Elf64Addr,
    pub p_paddr: Elf64Addr,
    pub p_filesz: Elf64Xword,
    pub p_memsz: Elf64Xword,
    pub p_align: Elf64Xword,
}

const PT_LOAD: Elf64Word = 1;

unsafe fn phdrs<'a>(ehdr: *const Elf64_Ehdr) -> &'a [Elf64_Phdr] {
    from_raw_parts(
        (ehdr as *const u8).add((*ehdr).e_phoff as usize) as *const Elf64_Phdr,
        (*ehdr).e_phnum as _,
    )
}

unsafe fn calc_load_address_range(ehdr: *const Elf64_Ehdr) -> Option<(u64, u64)> {
    let loads = phdrs(ehdr).iter().filter(|phdr| phdr.p_type == PT_LOAD);
    let first = loads.clone().map(|phdr| phdr.p_vaddr).min()?;
    let last = loads
        .clone()
        .map(|phdr| phdr.p_vaddr + phdr.p_memsz)
        .max()?;
    Some((first, last))
}

unsafe fn copy_load_segments(ehdr: *const Elf64_Ehdr, buffer: &mut [u8]) {
    for phdr in phdrs(ehdr) {
        if phdr.p_type != PT_LOAD {
            continue;
        }

        let src_addr = (ehdr as *const u8).add(phdr.p_offset as _);
        let section_start_index_in_buffer = ((*phdr).p_vaddr - buffer.as_ptr() as u64) as usize;
        buffer[section_start_index_in_buffer..]
            .as_mut_ptr()
            .copy_from(src_addr, phdr.p_filesz as usize);

        let remain_bytes = (phdr.p_memsz - phdr.p_filesz) as usize;
        let remain_first_addr =
            buffer[section_start_index_in_buffer + phdr.p_filesz as usize..].as_mut_ptr();
        write_bytes(remain_first_addr, 0, remain_bytes);
    }
}
