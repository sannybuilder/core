#![feature(io_error_more)]
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::atomic::{AtomicI64, Ordering},
};
use cached::proc_macro::cached;
use ctor::ctor;
use serde::Deserialize;
use simplelog::*;
mod ffi;
mod http_client;
use const_format::concatcp;

use crate::utils::version::compare_versions;

const GITHUB_ORG_NAME: &str = "sannybuilder";
const GITHUB_REPO_NAME: &str = "dev";
const GITHUB_BRANCH_NAME: &str = "updates";
const UPDATE_DIR_NAME: &str = "updates";
const DELETE_EXTENSION: &str = "deleteonlaunch";

const UPDATE_REPO: &str = concatcp!(
    "https://api.github.com/repos/",
    GITHUB_ORG_NAME,
    "/",
    GITHUB_REPO_NAME,
    "/commits/",
    GITHUB_BRANCH_NAME
);
const CONTENT_URL: &str = concatcp!(
    "https://raw.githubusercontent.com/",
    GITHUB_ORG_NAME,
    "/",
    GITHUB_REPO_NAME,
    "/",
    GITHUB_BRANCH_NAME
);

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub enum Channel {
    Stable,
    Beta,
    Nightly,
}

impl From<u8> for Channel {
    fn from(val: u8) -> Self {
        match val {
            1 => Channel::Beta,
            2 => Channel::Nightly,
            _ => Channel::Stable,
        }
    }
}

#[derive(Deserialize, Debug, Default)]
struct UpdateInfo {
    min: String,
    url: String,
}

#[derive(Deserialize, Debug, Default)]
struct Manifest {
    version: String,

    #[serde(rename = "_")]
    info: Vec<UpdateInfo>,

    release_notes: String,
}

impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Channel::Stable => write!(f, "stable"),
            Channel::Beta => write!(f, "beta"),
            Channel::Nightly => write!(f, "nightly"),
        }
    }
}

fn get_exe_folder() -> std::io::Result<PathBuf> {
    let path = std::env::current_exe()?;
    match path.parent() {
        Some(p) => Ok(p.to_path_buf()),
        None => Err(std::io::Error::from(std::io::ErrorKind::Other)),
    }
}

fn get_update_dir() -> std::io::Result<PathBuf> {
    Ok(get_exe_folder()?.join(UPDATE_DIR_NAME))
}

// fn store_update_check_timestamp() {
//     let ts = chrono::Utc::now().timestamp();

//     match get_exe_folder() {
//         Ok(folder) => {
//             let ini_file = folder.join(SETTINGS_FILE);
//             ini::Ini::load_from_file_noescape(&ini_file).and_then(|mut map| {
//                 map.set_to(Some("Main"), UPDATE_TIME_KEY.into(), ts.to_string());
//                 map.write_to_file_opt(
//                     ini_file,
//                     ini::WriteOption {
//                         escape_policy: ini::EscapePolicy::Nothing, // don't escape backslashes
//                         ..Default::default()
//                     },
//                 )?;
//                 log::debug!("update check timestamp updated to {}", ts);
//                 Ok(())
//             });
//         }
//         _ => {
//             log::error!("failed to get exe folder");
//         }
//     }
// }

// fn get_update_check_timestamp() -> i64 {
//     match get_exe_folder() {
//         Ok(folder) => {
//             let ini_file = folder.join(SETTINGS_FILE);
//             match ini::Ini::load_from_file_noescape(&ini_file) {
//                 Ok(map) => match map.get_from(Some("Main"), UPDATE_TIME_KEY.into()) {
//                     Some(t) => t.parse::<i64>().unwrap_or_else(|e| {
//                         log::error!("parse error while reading settings.ini {e}");
//                         0
//                     }),
//                     _ => {
//                         log::debug!(
//                             "LastUpdateChecklast check value is not found in ini file {}",
//                             ini_file.display()
//                         );
//                         0
//                     }
//                 },
//                 Err(e) => {
//                     log::debug!("can't load ini file {}: {e}", ini_file.display());
//                     0
//                 }
//             }
//         }
//         _ => {
//             log::error!("failed to get exe folder");
//             0
//         }
//     }
// }

// pub fn has_passed_check_cooldown() -> bool {
//     let cooldown = chrono::Utc::now() - chrono::Duration::hours(COOLDOWN_HOURS);
//     let last = get_update_check_timestamp();
//     cooldown.timestamp() >= last
// }

#[cached(time=60)]
fn get_from_channel(channel: Channel) -> Option<serde_json::Value> {
    http_client::get_json(&format!("{}/{}", CONTENT_URL, channel))
}

fn get_latest_version_from_github(channel: Channel) -> Option<String> {
    get_from_channel(channel).and_then(|json| match serde_json::from_value::<Manifest>(json) {
        Ok(manifest) => Some(manifest.version),
        Err(e) => {
            log::error!("{e}");
            None
        }
    })
}

fn get_release_notes_from_github(channel: Channel) -> Option<String> {
    get_from_channel(channel).and_then(|json| match serde_json::from_value::<Manifest>(json) {
        Ok(manifest) => Some(manifest.release_notes),
        Err(e) => {
            log::error!("{e}");
            None
        }
    })
}

fn get_download_link(channel: Channel, local_version: &str) -> Option<String> {
    get_from_channel(channel).and_then(|json| {
        match serde_json::from_value::<Manifest>(json) {
            Ok(manifest) => {
                for i in manifest.info {
                    // fallback url
                    if (i.min.eq("*")) {
                        return Some(i.url);
                    }
                    let min = version_compare::Version::from(&i.min);
                    let current = version_compare::Version::from(local_version);

                    if current >= min {
                        return Some(i.url);
                    }
                }
                log::error!("no suitable update found for {local_version}");
                None
            }
            Err(e) => {
                log::error!("{e}");
                None
            }
        }
    })
}

fn zip_unpack(reader: impl std::io::Read + std::io::Seek, base: &Path) -> std::io::Result<()> {
    let mut archive = zip::ZipArchive::new(reader)?;
    archive.extract(base)?;
    Ok(())
}

fn move_files(src_dir: impl AsRef<Path>, dst_dir: impl AsRef<Path>) -> std::io::Result<()> {
    use std::fs;
    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst = dst_dir.as_ref().join(entry.file_name());
        if ty.is_dir() {
            // entry is a folder in source
            move_files(entry.path(), dst)?;
        } else {
            // entry is a file in source
            log::debug!("copying {} to {}", entry.path().display(), dst.display());
            match fs::rename(entry.path(), &dst) {
                Ok(_) => {}
                Err(e) => {
                    // try renaming original file
                    match dst.extension().and_then(std::ffi::OsStr::to_str) {
                        Some(ext)
                            if ["exe", "dll"].contains(&ext.to_ascii_lowercase().as_str()) =>
                        {
                            //
                            match fs::rename(
                                &dst,
                                &dst.with_extension(format!("{ext}.{DELETE_EXTENSION}")),
                            ) {
                                Ok(_) => {
                                    // try again
                                    if let Ok(_) = fs::rename(entry.path(), dst) {
                                        continue;
                                    }
                                }
                                Err(e) => {
                                    log::error!("Failed on renaming used file {}", dst.display());
                                }
                            }
                        }
                        _ => {}
                    };

                    return Err(e);
                }
            }
        }
    }
    Ok(())
}

fn delete_files(src_dir: impl AsRef<Path>, extension: &str) -> std::io::Result<()> {
    use std::fs;
    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            // entry is a folder in source
            delete_files(entry.path(), extension)?;
        } else {
            // entry is a file in source
            let file = entry.path();

            match file.extension().and_then(std::ffi::OsStr::to_str) {
                Some(ext) if ext.eq_ignore_ascii_case(extension) => {
                    log::debug!("deleting {}", file.display());
                    match fs::remove_file(file) {
                        Ok(_) => {}
                        Err(e) => {
                            log::error!("{e}");
                        }
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

pub fn download_update(channel: Channel, local_version: &str) -> bool {
    log::info!("downloading update from channel {channel}");

    let Some(url) = get_download_link(channel, local_version) else {
        log::error!("failed to get download link");
        return false;
    };

    log::debug!("downloading archive from {}", url);

    let Some(buf) = http_client::get_binary(&url) else {
        log::error!("failed to download archive");
        return false;
    };
    // save to local file
    let Ok(cwd) = get_exe_folder() else {
        log::error!("failed to get exe folder");
        return false;
    };

    let Ok(temp) = get_update_dir() else {
        log::error!("failed to get update dir");
        return false;
    };
    log::info!("unpacking update to {}", temp.display());

    match zip_unpack(&mut std::io::Cursor::new(buf), &temp) {
        Ok(_) => {}
        Err(e) => {
            log::error!("unpack failed: {e}");
            return false;
        }
    }

    // // copy all files from pending to cwd
    log::info!("copying files to {}", cwd.display());
    match move_files(temp, cwd) {
        Ok(_) => {
            log::debug!("copied all files");
        }
        Err(e) => {
            log::error!("copy failed: {e}");
            return false;
        }
    }

    log::info!("update complete.");

    true
}

pub fn has_update(channel: Channel, local_version: &str) -> bool {
    use version_compare::Version;

    // if (!has_passed_check_cooldown()) {
    //     log::info!("update check cooldown has not passed yet.");
    //     return false;
    // }
    // store_update_check_timestamp();

    log::info!("checking for updates from channel {channel}");
    let Some(remote_version) = get_latest_version_from_github(channel) else {
        log::error!("failed to get remote snapshot");
        return false;
    };
    let remote_version = Version::from(&remote_version);
    log::debug!("remote version: {:?}", remote_version);

    let local_version = Version::from(local_version);
    log::debug!("local version: {:?}", local_version);

    if local_version >= remote_version {
        log::info!("already on latest version. no update needed");
        return false;
    }

    log::info!("new version available: {:?}", remote_version);

    return true;
}

pub fn cleanup() {
    log::info!("Performing cleanup on current directory");
    let Ok(cwd) = get_exe_folder() else {
        log::error!("failed to get exe folder");
        return;
    };

    delete_files(cwd, DELETE_EXTENSION);
}
