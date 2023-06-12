use std::ffi::c_void;
use std::mem::transmute_copy;

pub mod data;
use windows::{
    core::PCSTR,
    imp::{GetLastError, GetProcAddress, LoadLibraryA},
    Win32::Foundation::{HMODULE, NTSTATUS},
};

pub struct PCSTRWrapper {
    text: PCSTR,
    #[allow(unused)]
    _container: Vec<u8>,
}

impl std::ops::Deref for PCSTRWrapper {
    type Target = PCSTR;

    fn deref(&self) -> &Self::Target {
        &self.text
    }
}

pub trait ToPCSTRWrapper {
    fn to_pcstr(&self) -> PCSTRWrapper;
}

impl ToPCSTRWrapper for &str {
    fn to_pcstr(&self) -> PCSTRWrapper {
        // https://stackoverflow.com/questions/47980023/how-to-convert-from-u8-to-vecu8
        let mut text = self.as_bytes().iter().cloned().collect::<Vec<u8>>();
        text.push(0); // add null

        PCSTRWrapper {
            text: PCSTR(text.as_ptr()),
            _container: text, // data lifetime management
        }
    }
}

impl ToPCSTRWrapper for PCSTR {
    fn to_pcstr(&self) -> PCSTRWrapper {
        PCSTRWrapper {
            text: *self,
            _container: Vec::new(),
        }
    }
}

pub const LMEM_ZEROINIT: u8 = 0;

pub fn wrap_load_library_a<T>(name: T) -> Result<HMODULE, ()>
where
    T: ToPCSTRWrapper,
{
    let name = *name.to_pcstr();
    unsafe {
        match LoadLibraryA(name) {
            0 => Err(eprintln!(
                "Failed LoadLibraryA\nGetLastError(): {:?}",
                GetLastError()
            )),
            v => Ok(HMODULE(v)),
        }
    }
}

pub fn wrap_get_proc_address<T>(library: HMODULE, name: T) -> Result<*const c_void, ()>
where
    T: ToPCSTRWrapper,
{
    let name = *name.to_pcstr();
    unsafe {
        let ret = GetProcAddress(library.0, name);
        if ret.is_null() {
            Err(eprintln!(
                "Failed GetProcAddress\nGetLastError(): {:?}",
                GetLastError()
            ))
        } else {
            Ok(ret)
        }
    }
}

pub type DWORD = u32;
pub type PDWORD = *mut u32;
pub type PBYTE = *mut u8;
pub type ULONG = u32;
pub type BOOLEAN = u8;
pub type PBOOLEAN = *mut u8;
pub type ErrNTSTATUS = u32;

pub type RtlAdjustPrivilegeFn = extern "system" fn(
    privileges: ULONG,
    enable: BOOLEAN,
    current_thread: BOOLEAN,
    enabled: PBOOLEAN,
) -> NTSTATUS;

pub type NtRaiseHardErrorFn = extern "system" fn(
    error_status: NTSTATUS,
    number_of_parameters: DWORD,
    unicode_string_parameter_mask: DWORD,
    parameters: DWORD,
    valid_response_option: DWORD,
    response: PDWORD,
) -> NTSTATUS;

pub enum NtdllError {
    NtdllError,
}

pub struct Library {
    handle: HMODULE,
}

impl Library {
    pub fn new(name: &str) -> Self {
        let res = wrap_load_library_a(name).expect("Failed LoadLibraryA()\nUnknown Error");
        Self { handle: res }
    }

    pub fn get_proc<T>(&self, name: &str) -> Option<T> {
        let res = wrap_get_proc_address(self.handle, name)
            .expect("Failed GetProcAddress()\nUnknown Error");
        unsafe { transmute_copy(&res) }
    }
}
