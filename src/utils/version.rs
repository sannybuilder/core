use std::cmp::Ordering;
use version_info::get_file_version;

pub fn extract_version(file_name: &str) -> Option<String> {
    let (a1, a2, a3, a4) = get_file_version(file_name)?;
    Some(format!("{}.{}.{}.{}", a1, a2, a3, a4))
}

pub fn compare_versions(file1: &str, file2: &str) -> Option<Ordering> {
    let (a1, a2, a3, a4) = get_file_version(file1)?;
    let (b1, b2, b3, b4) = get_file_version(file2)?;

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
