//! Windows process elevation through the normal UAC consent flow.
use std::io;

#[derive(Debug, PartialEq)]
pub enum RestartOutcome {
    Launched,
    Cancelled,
}

pub fn is_elevated() -> io::Result<bool> {
    #[cfg(windows)]
    {
        use windows_sys::Win32::{
            Foundation::{CloseHandle, GetLastError},
            Security::{GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation},
            System::Threading::{GetCurrentProcess, OpenProcessToken},
        };
        let mut token = std::ptr::null_mut();
        let mut elevation = TOKEN_ELEVATION::default();
        let mut length = 0;
        // The process token describes actual elevation, including filtered UAC tokens.
        unsafe {
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
                return Err(io::Error::last_os_error());
            }
            let success = GetTokenInformation(
                token,
                TokenElevation,
                (&mut elevation as *mut TOKEN_ELEVATION).cast(),
                std::mem::size_of_val(&elevation) as u32,
                &mut length,
            );
            let error = GetLastError();
            CloseHandle(token);
            if success == 0 {
                return Err(io::Error::from_raw_os_error(error as i32));
            }
        }
        Ok(elevation.TokenIsElevated != 0)
    }
    #[cfg(not(windows))]
    {
        Ok(true)
    }
}

pub fn restart_as_administrator() -> io::Result<RestartOutcome> {
    #[cfg(windows)]
    {
        // ShellExecuteEx may use STA shell extensions; use a fresh COM thread.
        std::thread::spawn(restart_windows)
            .join()
            .map_err(|_| io::Error::other("Administrator restart thread stopped"))?
    }
    #[cfg(not(windows))]
    {
        Ok(RestartOutcome::Cancelled)
    }
}

#[cfg(windows)]
fn restart_windows() -> io::Result<RestartOutcome> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::{
        Foundation::{CloseHandle, ERROR_CANCELLED, GetLastError},
        System::Com::{
            COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoInitializeEx, CoUninitialize,
        },
        UI::{
            Shell::{
                SEE_MASK_FLAG_NO_UI, SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW,
                ShellExecuteExW,
            },
            WindowsAndMessaging::SW_SHOWNORMAL,
        },
    };
    let executable = std::env::current_exe()?;
    let directory = executable
        .parent()
        .ok_or_else(|| io::Error::other("Missing executable directory"))?;
    let file: Vec<u16> = executable
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    let directory: Vec<u16> = directory.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut request = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOASYNC | SEE_MASK_NOCLOSEPROCESS | SEE_MASK_FLAG_NO_UI,
        lpVerb: windows_sys::core::w!("runas"),
        lpFile: file.as_ptr(),
        lpDirectory: directory.as_ptr(),
        nShow: SW_SHOWNORMAL,
        ..Default::default()
    };
    // The UTF-16 buffers remain alive until ShellExecuteEx has finished.
    unsafe {
        let hr = CoInitializeEx(
            std::ptr::null(),
            (COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) as u32,
        );
        if hr < 0 {
            return Err(io::Error::other(format!(
                "COM initialization failed: {hr:#x}"
            )));
        }
        let success = ShellExecuteExW(&mut request);
        let error = GetLastError();
        if !request.hProcess.is_null() {
            CloseHandle(request.hProcess);
        }
        CoUninitialize();
        if success != 0 {
            Ok(RestartOutcome::Launched)
        } else if error == ERROR_CANCELLED {
            Ok(RestartOutcome::Cancelled)
        } else {
            Err(io::Error::from_raw_os_error(error as i32))
        }
    }
}

#[cfg(all(test, windows))]
mod tests {
    #[test]
    fn process_elevation_can_be_read_without_requesting_uac() {
        super::is_elevated().expect("current process token should be readable");
    }
}
