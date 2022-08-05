use super::ffi::StatusChangeCallback;
use base64::decode;
use std::error::Error;
use version_compare::Version;

const API_URL: &str = "https://api.github.com/repos/sannybuilder/library/contents";
const RAW_CONTENT: &str = "https://raw.githubusercontent.com/sannybuilder/library/master";

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
            let decoded = get_file_gracefully(game, "version.txt").unwrap_or("".to_string());
            (status_change)(pchar!(decoded));
        });
        Some(())
    }

    pub fn auto_update(&self, params: &str) -> Option<()> {
        let mut v = Vec::new();
        let mut x = params.split_ascii_whitespace().map(|x| x.to_string());
        while let Some(game) = x.next() {
            if let Some(version) = x.next() {
                v.push((game, version, x.next()));
            } else {
                return None;
            }
        }
        let status_change = self.status_change;
        std::thread::spawn(move || {
            let result = auto_update(v).unwrap_or("".to_string());
            (status_change)(pchar!(result.as_str()));
        });
        Some(())
    }

    pub fn download(&self, params: &str) -> Option<()> {
        let mut x = params.split_ascii_whitespace().map(|x| x.to_string());
        if let Some(game) = x.next() {
            let status_change = self.status_change;
            let destination = x.next();
            std::thread::spawn(move || {
                let result = download(game, destination).unwrap_or("".to_string());
                (status_change)(pchar!(result.as_str()));
            });
        } else {
            return None;
        }
        Some(())
    }
}

fn get_file_gracefully(game: String, file_name: &str) -> Result<String, Box<dyn Error>> {
    let url = format!("{}/{}/{}", RAW_CONTENT, game, file_name);
    match super::http_client::get_string(url.as_str()) {
        Some(s) => Ok(s),
        None => get_file_with_api(game, file_name),
    }
}

fn get_file_with_api(game: String, file_name: &str) -> Result<String, Box<dyn Error>> {
    let path = &format!("{}/{}", game, file_name);

    let contents = super::http_client::get_json(format!("{}/{}", API_URL, game).as_str())
        .ok_or("Request failed")?;
    let contents = contents.as_array().ok_or("Response is not an array")?;

    let git_url = contents
        .iter()
        .find_map(|x| {
            let x = x.as_object()?;
            if x.get("path")?.eq(path) {
                return x.get("git_url")?.as_str();
            }
            None
        })
        .ok_or(format!("Can't find git_url for {}", path))?;

    let body = super::http_client::get_json(git_url).ok_or("Request failed")?;

    let content = body
        .as_object()
        .ok_or("Response is not an object")?
        .get("content")
        .ok_or("Key 'content' is not found")?
        .as_str()
        .ok_or("Content is not a string")?;

    let content = content.replace('\n', "");
    let decoded = decode(content)?;

    String::from_utf8(decoded).or_else(|_| Err("Can't decode string".into()))
}

fn get_file_name<'a>(game: &str) -> Option<&'a str> {
    match game {
        "gta3" => Some("gta3.json"),
        "vc" => Some("vc.json"),
        "sa" => Some("sa.json"),
        "sa_mobile" => Some("sa_mobile.json"),
        "vc_mobile" => Some("vc_mobile.json"),
        _ => None,
    }
}

fn download(game: String, destination: Option<String>) -> Result<String, Box<dyn Error>> {
    let file_name = get_file_name(&game).ok_or("Unsupported game")?;
    let destination = destination.unwrap_or(file_name.to_owned());
    let decoded = get_file_gracefully(game, file_name)?;
    std::fs::write(destination, &decoded)?;
    let lib = serde_json::from_str::<crate::namespaces::Library>(decoded.as_str())?;
    Ok(lib.meta.version)
}

fn auto_update(options: Vec<(String, String, Option<String>)>) -> Result<String, Box<dyn Error>> {
    let mut versions: String = String::new();
    for (game, version, destination) in options {
        let v = match get_file_gracefully(game.clone(), "version.txt") {
            Ok(x) => x,
            Err(_) => continue,
        };

        let file_name = get_file_name(&game).ok_or("Unsupported game")?;
        let destination = destination.unwrap_or(file_name.to_owned());
        let remote_version = Version::from(v.as_str());
        let local_version = Version::from(version.as_str());

        if remote_version > local_version {
            let decoded = get_file_gracefully(game.clone(), file_name)?;
            std::fs::write(destination, &decoded)?;
            let lib = serde_json::from_str::<crate::namespaces::Library>(decoded.as_str())?;

            versions.push_str(format!("{} {}", game, lib.meta.version).as_str());
        }
    }
    Ok(versions)
}
