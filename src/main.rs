#![no_std]
#![no_main]
#![feature(abi_efiapi)]
use core::fmt::Write;
use core::slice::from_raw_parts_mut;
use uefi::prelude::*;
use uefi::proto::media::file::{File, FileAttribute, FileInfo, FileMode};
use uefi::table::boot::{AllocateType, MemoryType};
use uefi::CStr16;

#[entry]
fn main(handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();

    writeln!(system_table.stdout(), "hello from bootloader").unwrap();

    unsafe {
        let fs_sytem_table = system_table.unsafe_clone();
        let fs = fs_sytem_table
            .boot_services()
            .get_image_file_system(handle)
            .unwrap();
        let mut volume = (*fs.interface.get()).open_volume().unwrap();

        let mut kernel_filename_buffer = [0; 7];
        let kernel_filename =
            CStr16::from_str_with_buf("kernel", &mut kernel_filename_buffer).unwrap();

        let handle_result = volume.open(kernel_filename, FileMode::Read, FileAttribute::READ_ONLY);
        let handle = match handle_result {
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
        let mut file = handle.into_regular_file().unwrap();
        // TODO: allocate enough memory
        let mut file_info_buffer = [0; 1000];
        let file_info = file.get_info::<FileInfo>(&mut file_info_buffer).unwrap();

        const PAGE_UNIT_SIZE: usize = 0x1000;
        let kernel_file_size = file_info.file_size() as usize;
        let page_size = (kernel_file_size + PAGE_UNIT_SIZE - 0x1) / PAGE_UNIT_SIZE;

        writeln!(system_table.stdout(), "{}", kernel_file_size).unwrap();

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

        let ptr = u64::from_ne_bytes(buffer[24..32].try_into().unwrap());
        writeln!(system_table.stdout(), "{:?}", ptr as *const u64).unwrap();

        // ↑↑↑↑↑ mikan book list3.2
        // TODO: Move the instruction pointer to the address stored at (buffer + 24).
    }

    Status::SUCCESS
}
