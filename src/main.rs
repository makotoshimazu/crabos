#![no_std]
#![no_main]
#![feature(abi_efiapi)]
use core::clone;
use core::fmt::Write;
use uefi::prelude::*;
use uefi::proto::media::file::{File, FileAttribute, FileMode};
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
        let mut buffer = [0; 10000];
        let nbytes = file.read(&mut buffer).unwrap();

        writeln!(system_table.stdout(), "{}", nbytes).unwrap();

        // TODO: Move the instruction pointer to the address stored at (buffer + 24).
    }

    Status::SUCCESS
}
