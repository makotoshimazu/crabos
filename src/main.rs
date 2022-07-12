#![no_std]
#![no_main]
#![feature(abi_efiapi)]
use uefi::prelude::*;
use uefi::proto::media::file::{File, FileAttribute, FileMode};
use uefi::CStr16;

#[entry]
fn main(handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();

    let fs = system_table
        .boot_services()
        .get_image_file_system(handle)
        .unwrap();
    unsafe {
        let mut volume = (*fs.interface.get()).open_volume().unwrap();

        let mut kernel_filename_buffer = [0; 7];
        let kernel_filename =
            CStr16::from_str_with_buf("kernel", &mut kernel_filename_buffer).unwrap();

        let handle = volume
            .open(kernel_filename, FileMode::Read, FileAttribute::READ_ONLY)
            .unwrap();
        let mut file = handle.into_regular_file().unwrap();
        // TODO: allocate enough memory
        let mut buffer = [0; 10000];
        let _nbytes = file.read(&mut buffer).unwrap();

        // TODO: Move the instruction pointer to the address stored at (buffer + 24).
    }

    Status::SUCCESS
}
