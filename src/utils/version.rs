use std::cmp::Ordering;

pub fn extract_version(file_name: &str) -> Option<String> {
    let (a1, a2, a3, a4) = get_file_version(std::path::PathBuf::from(file_name))?;
    Some(format!("{}.{}.{}.{}", a1, a2, a3, a4))
}

pub fn compare_versions(file1: &str, file2: &str) -> Option<Ordering> {
    let (a1, a2, a3, a4) = get_file_version(std::path::PathBuf::from(file1))?;
    let (b1, b2, b3, b4) = get_file_version(std::path::PathBuf::from(file2))?;

    if a1 < b1 {
        return Some(Ordering::Less);
    }
    if a1 > b1 {
        return Some(Ordering::Greater);
    }
    if a2 < b2 {
        return Some(Ordering::Less);
    }
    if a2 > b2 {
        return Some(Ordering::Greater);
    }
    if a3 < b3 {
        return Some(Ordering::Less);
    }
    if a3 > b3 {
        return Some(Ordering::Greater);
    }
    if a4 < b4 {
        return Some(Ordering::Less);
    }
    if a4 > b4 {
        return Some(Ordering::Greater);
    }
    Some(Ordering::Equal)
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct VS_FIXEDFILEINFO {
    pub dwSignature: u32,
    pub dwStrucVersion: u32,
    pub dwFileVersionMS: u32,
    pub dwFileVersionLS: u32,
    pub dwProductVersionMS: u32,
    pub dwProductVersionLS: u32,
    pub dwFileFlagsMask: u32,
    pub dwFileFlags: u32,
    pub dwFileOS: i32,
    pub dwFileType: i32,
    pub dwFileSubtype: i32,
    pub dwFileDateMS: u32,
    pub dwFileDateLS: u32,
}

pub fn get_file_version(file_name: std::path::PathBuf) -> Option<(u32, u32, u32, u32)> {
    use winapi::um::winver::{GetFileVersionInfoA, GetFileVersionInfoSizeA, VerQueryValueA};

    unsafe {
        let filename = std::ffi::CString::new(file_name.to_str()?).unwrap();
        let mut handle = 0;
        let size = GetFileVersionInfoSizeA(filename.as_ptr(), &mut handle);

        if size == 0 {
            return None;
        }

        let mut buf = vec![0u8; size as usize];
        let pbuf = buf.as_mut_ptr() as *mut _;

        if GetFileVersionInfoA(filename.as_ptr(), 0, size, pbuf) == 0 {
            return None;
        }

        let mut pinfo: winapi::um::winnt::PVOID = std::ptr::null_mut();
        let mut length = 0;
        let path = std::ffi::CString::new("\\").unwrap();

        if VerQueryValueA(pbuf, path.as_ptr(), &mut pinfo, &mut length) == 0 {
            return None;
        }

        let info = &*(pinfo as *const VS_FIXEDFILEINFO);

        let v1 = info.dwFileVersionMS >> 16 & 0xFFFF;
        let v2 = info.dwFileVersionMS >> 0 & 0xFFFF;
        let v3 = info.dwFileVersionLS >> 16 & 0xFFFF;
        let v4 = info.dwFileVersionLS >> 0 & 0xFFFF;
        return Some((v1, v2, v3, v4));
    }
}
