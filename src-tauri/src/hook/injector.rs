use std::ffi::CString;
use std::path::PathBuf;
use sysinfo::System;
use windows::core::PCSTR;
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::*;
use windows::Win32::System::Memory::*;
use windows::Win32::System::Threading::*;

pub struct DllInjector {
    dll_path: PathBuf,
}

impl DllInjector {
    pub fn new(dll_path: PathBuf) -> Self {
        Self { dll_path }
    }

    /// Find League of Legends process
    pub fn find_league_process() -> Option<u32> {
        let mut system = System::new_all();
        system.refresh_all();

        system
            .processes()
            .iter()
            .find(|(_, p)| {
                let name = p.name().to_string();
                name == "LeagueOfLegends.exe" || name == "League of Legends.exe"
            })
            .map(|(pid, _)| pid.as_u32())
    }

    /// Inject DLL into target process
    pub fn inject(&self, pid: u32) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            // Open target process
            let h_process = OpenProcess(PROCESS_ALL_ACCESS, false, pid)?;
            if h_process.is_invalid() {
                return Err("Failed to open process".into());
            }

            // Get full DLL path
            let dll_path_str = self
                .dll_path
                .to_str()
                .ok_or("Invalid DLL path")?;
            let dll_path_cstr = CString::new(dll_path_str)?;
            let dll_path_bytes = dll_path_cstr.as_bytes_with_nul();

            // Allocate memory in target process
            let remote_mem = VirtualAllocEx(
                h_process,
                None,
                dll_path_bytes.len(),
                MEM_COMMIT | MEM_RESERVE,
                PAGE_READWRITE,
            );

            if remote_mem.is_null() {
                CloseHandle(h_process);
                return Err("Failed to allocate memory in target process".into());
            }

            // Write DLL path to remote process
            let mut bytes_written = 0;
            let write_result = windows::Win32::System::Diagnostics::Debug::WriteProcessMemory(
                h_process,
                remote_mem,
                dll_path_bytes.as_ptr() as *const _,
                dll_path_bytes.len(),
                Some(&mut bytes_written),
            );

            if write_result.is_err() {
                VirtualFreeEx(h_process, remote_mem, 0, MEM_RELEASE);
                CloseHandle(h_process);
                return Err("Failed to write DLL path to process".into());
            }

            // Get LoadLibraryA address from kernel32.dll
            let kernel32 = GetModuleHandleA(PCSTR(b"kernel32.dll\0" as *const u8))?;
            let load_library_addr = GetProcAddress(kernel32, PCSTR(b"LoadLibraryA\0" as *const u8))
                .ok_or("Failed to get LoadLibraryA address")?;

            // Create remote thread to call LoadLibraryA
            let h_thread = CreateRemoteThread(
                h_process,
                None,
                0,
                Some(std::mem::transmute(load_library_addr)),
                Some(remote_mem),
                0,
                None,
            )?;

            if h_thread.is_invalid() {
                VirtualFreeEx(h_process, remote_mem, 0, MEM_RELEASE);
                CloseHandle(h_process);
                return Err("Failed to create remote thread".into());
            }

            // Wait for thread to complete
            WaitForSingleObject(h_thread, 5000);

            // Cleanup
            CloseHandle(h_thread);
            VirtualFreeEx(h_process, remote_mem, 0, MEM_RELEASE);
            CloseHandle(h_process);

            println!("DLL injected successfully into PID {}", pid);
            Ok(())
        }
    }
}

/// Inject DLL into League of Legends
pub fn inject_into_league(
    dll_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let pid = DllInjector::find_league_process()
        .ok_or("League of Legends process not found")?;

    let injector = DllInjector::new(dll_path);
    injector.inject(pid)?;

    Ok(())
}
