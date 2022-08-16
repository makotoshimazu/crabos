#![no_std]
#![no_main]
#![feature(abi_efiapi)]
use core::fmt::Write;
use core::mem;
use core::slice::{from_raw_parts, from_raw_parts_mut};
use uefi::data_types::Align;
use uefi::prelude::*;
use uefi::proto::media::file::{File, FileAttribute, FileInfo, FileMode};
use uefi::table::boot::{AllocateType, MemoryDescriptor, MemoryType};
use uefi::CStr16;

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

        // Read `bin/kernel.rs`.
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
        let page_size = (kernel_file_size + PAGE_UNIT_SIZE - 0x1) / PAGE_UNIT_SIZE;

        writeln!(
            system_table.stdout(),
            "kernel file size: {}",
            kernel_file_size
        )
        .unwrap();

        let addr = fs_sytem_table
            .boot_services()
            .allocate_pages(
                AllocateType::Address(0x100000),
                MemoryType::LOADER_DATA,
                page_size,
            )
            .unwrap();
        let buffer = from_raw_parts_mut(addr as *mut u8, page_size * PAGE_UNIT_SIZE);
        file.read(buffer).unwrap();

        // Reference: Mikan book p.79
        // TODO: Understand this magic...
        let ptr = addr + 0x1e0;
        writeln!(
            system_table.stdout(),
            "entry point addr: {:?}",
            ptr as *const u64
        )
        .unwrap();

        // ↑↑↑↑↑ mikan book list3.2

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

        let f = core::mem::transmute::<_, extern "C" fn() -> core::ffi::c_void>(ptr);
        f();

        Status::SUCCESS
    }
}
