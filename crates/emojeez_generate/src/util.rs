use std::{any::type_name, env, fs, io, path::Path};

use reqwest::blocking::Client;
use sha2::{Digest, Sha256};

pub fn cached_download(url: &str) -> Result<String, io::Error> {
    let checksum = hex::encode(Sha256::digest(url.as_bytes()));
    let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/target/generate"))
        .join(checksum)
        .with_extension("txt");

    let cwd = env::current_dir()?;

    match fs::read_to_string(&path) {
        Ok(data) => {
            eprintln!(
                "using cached: {url}\n    at {}",
                path.strip_prefix(&cwd).unwrap_or(&path).display()
            );
            return Ok(data);
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
        Err(err) => return Err(err),
    }

    let data = download(url).map_err(|e| io::Error::other(format!("download failed: {e}")))?;

    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(&path, data.as_bytes())?;

    eprintln!(
        "downloaded: {url} to {}",
        path.strip_prefix(&cwd).unwrap_or(&path).display()
    );

    Ok(data)
}

pub fn download(url: &str) -> Result<String, io::Error> {
    let client = Client::new();

    let resp = client
        .get(url)
        .send()
        .map_err(|e| io::Error::other(format!("request failed: {e}")))?;

    if !resp.status().is_success() {
        return Err(io::Error::other(format!(
            "request failed with status: {}",
            resp.status()
        )));
    }

    let text = resp
        .text()
        .map_err(|e| io::Error::other(format!("read failed: {e}")))?;

    Ok(text)
}

pub fn struct_name<T>() -> String {
    let full = type_name::<T>();
    // Strip generic arguments first
    let before_generics = full.split('<').next().unwrap();
    // Take last segment after ::
    before_generics.rsplit("::").next().unwrap().to_owned()
}

pub fn struct_package<T>() -> &'static str {
    let full = type_name::<T>();
    let before_generics = full.split('<').next().unwrap();
    let mut parts = before_generics.rsplitn(2, "::");
    parts.next();
    parts.next().unwrap_or("")
}

#[cfg(test)]
mod tests {
    use unicode_types::Group;

    use super::*;

    #[test]
    fn test_struct_name_empty_case() {
        assert_eq!(struct_name::<i32>(), "i32");
        assert_eq!(struct_name::<u64>(), "u64");
        assert_eq!(struct_name::<bool>(), "bool");
        assert_eq!(struct_name::<Group>(), "Group");
    }

    #[test]
    fn test_struct_package_no_module() {
        assert_eq!(struct_package::<i32>(), "");
        assert_eq!(struct_package::<Group>(), "unicode_types");
    }
}
