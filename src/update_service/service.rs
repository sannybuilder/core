use super::ffi::StatusChangeCallback;
use base64::decode;
use std::error::Error;
use version_compare::Version;

const API_URL: &str = "https://api.github.com/repos/sannybuilder/library/contents";
const REPO: &str = "https://raw.githubusercontent.com/sannybuilder/library";

pub struct UpdateService {
    pub status_change: StatusChangeCallback,
}

impl UpdateService {
    pub fn new(cb: StatusChangeCallback) -> Self {
        log::debug!("Update service created");
        UpdateService { status_change: cb }
    }

    pub fn check_version(&self, params: &str) -> Option<()> {
        let status_change = self.status_change;
        let game = params.to_string();
        std::thread::spawn(move || {
            let decoded = match download_file_gracefully(&game, "version.txt", "master") {
                Ok(v) => std::ffi::CString::new(v).unwrap(),
                Err(e) => {
                    log::error!("{}", e);
                    std::ffi::CString::new("").unwrap()
                }
            };
            (status_change)(decoded.as_ptr());
        });
        Some(())
    }

    pub fn auto_update(&self, params: &str) -> Option<()> {
        let status_change = self.status_change;
        let params = params.to_owned();
        std::thread::spawn(move || {
            let result = match auto_update(params) {
                Ok(v) => std::ffi::CString::new(v).unwrap(),
                Err(e) => {
                    log::error!("{}", e);
                    std::ffi::CString::new("").unwrap()
                }
            };
            (status_change)(result.as_ptr());
        });
        Some(())
    }

    pub fn download(&self, params: &str) -> Option<()> {
        static EMPTY_STRING: i8 = '\0' as i8;

        let params = params.to_owned();
        let status_change = self.status_change;
        std::thread::spawn(move || {
            let mut x = params.split_terminator('|'); //.map(|x| x.to_string());
            let game = match x.next() {
                Some(x) if !x.is_empty() => x,
                _ => {
                    log::error!("Error during update: no game specified");
                    (status_change)(&EMPTY_STRING as _);
                    return;
                }
            };
            let library_path = match x.next() {
                Some(x) if !x.is_empty() => x,
                _ => {
                    log::error!("Error during update: no library path specified");
                    (status_change)(&EMPTY_STRING as _);
                    return;
                }
            };

            let classes_path = x.next();
            let enums_path = x.next();
            let examples_path = x.next();

            let result = match download_library(game, library_path) {
                Ok(v) => {
                    download_config_files(classes_path, enums_path, examples_path, game);
                    std::ffi::CString::new(v).unwrap()
                }
                Err(e) => {
                    log::error!("{}", e);
                    std::ffi::CString::new("").unwrap()
                }
            };

            (status_change)(result.as_ptr());
        });

        Some(())
    }
}

fn download_file_gracefully(
    folder: &str,
    file_name: &str,
    branch: &str,
) -> Result<String, Box<dyn Error>> {
    let url = format!("{}/{}/{}/{}", REPO, branch, folder, file_name);
    log::info!("Downloading {}", url.as_str());
    match super::http_client::get_string(url.as_str()) {
        Some(s) => Ok(s),
        None => download_file_using_api(folder, file_name, branch),
    }
}

fn download_file_using_api(
    folder: &str,
    file_name: &str,
    branch: &str,
) -> Result<String, Box<dyn Error>> {
    let path = &format!("{}/{}", folder, file_name);

    let contents =
        super::http_client::get_json(format!("{}/{}?ref={}", API_URL, folder, branch).as_str())
            .ok_or("Request failed")?;
    let contents = contents.as_array().ok_or("Response is not an array")?;

    let git_url = contents
        .iter()
        .find_map(|x| {
            if x["path"].eq(path) {
                return x["git_url"].as_str();
            }
            None
        })
        .ok_or(format!("Can't find git_url for {}", path))?;

    let body = super::http_client::get_json(git_url).ok_or("Request failed")?;
    let content = body["content"].as_str().ok_or("Content is not a string")?;

    let content = content.replace('\n', "");
    let decoded = decode(content)?;

    String::from_utf8(decoded).or_else(|_| Err("Can't decode string".into()))
}

fn get_library_json_name<'a>(game: &str) -> Option<&'a str> {
    match game {
        "gta3" => Some("gta3.json"),
        "vc" => Some("vc.json"),
        "sa" => Some("sa.json"),
        "sa_mobile" => Some("sa_mobile.json"),
        "vc_mobile" => Some("vc_mobile.json"),
        // "lcs" => Some("lcs.json"),
        // "vcs" => Some("vcs.json"),
        _ => None,
    }
}

fn download_library(game: &str, destination: &str) -> Result<String, Box<dyn Error>> {
    let file_name = get_library_json_name(&game).ok_or("Unsupported game")?;
    let decoded = download_file_gracefully(&game, file_name, "master")?;
    log::info!("Saving new file {}", destination);
    std::fs::write(destination, &decoded)?;
    let lib = serde_json::from_str::<crate::namespaces::Library>(decoded.as_str())?;
    Ok(lib.meta.version)
}

fn download_text_file(
    folder: &str,
    file_name: &str,
    destination: &str,
) -> Result<(), Box<dyn Error>> {
    if folder.is_empty() || destination.is_empty() {
        return Ok(());
    }
    let decoded = download_file_gracefully(&folder, file_name, "gh-pages/assets")?;
    log::info!("Saving new file {}", destination);
    std::fs::write(destination, &decoded)?;
    Ok(())
}

fn auto_update(options: String) -> Result<String, Box<dyn Error>> {
    let mut versions: String = String::new();
    let mut x = options.split_terminator('|');
    while let Some(game) = x.next() {
        let version = x.next();
        let library_path = x.next();
        let classes_path = x.next();
        let enums_path = x.next();
        let examples_path = x.next();

        let Some(file_name) = get_library_json_name(&game) else {
            log::error!("Unsupported game {game}");
            continue;
        };
        let Ok(v) = download_file_gracefully(&game, "version.txt", "master") else {
            log::error!("Can't get version for {game}");
            continue;
        };

        let Some(version) = version else {
            log::error!("No version for {game}");
            continue;
        };

        let destination = library_path.unwrap_or(file_name);
        let remote_version = Version::from(v.as_str());
        let local_version = Version::from(version);

        log::debug!(
            "Remote version: {:?}, Local version: {:?}",
            remote_version,
            local_version
        );

        if remote_version > local_version {
            let decoded = download_file_gracefully(&game, file_name, "master")?;
            log::info!("Saving new file {}", destination);
            std::fs::write(destination, &decoded)?;

            download_config_files(classes_path, enums_path, examples_path, game);
            let lib = serde_json::from_str::<crate::namespaces::Library>(decoded.as_str())?;

            versions.push_str(format!("{} {}", game, lib.meta.version).as_str());
        }
    }
    Ok(versions)
}

fn download_config_files(
    classes_path: Option<&str>,
    enums_path: Option<&str>,
    examples_path: Option<&str>,
    game: &str,
) {
    ["classes.db", "enums.txt", "opcodes.txt"]
        .iter()
        .zip([classes_path, enums_path, examples_path].iter())
        .for_each(|(file, path)| match path {
            Some(path) if !path.is_empty() => {
                if let Err(e) = download_text_file(game, file, path) {
                    log::error!("{}", e)
                }
            }
            _ => (),
        });
}
