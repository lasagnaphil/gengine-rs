use std::path::PathBuf;
use find_folder;

lazy_static! {
    static ref ASSETS: PathBuf = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();
}

lazy_static! {
    static ref STORAGE: PathBuf = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("storage").unwrap();
}

pub fn asset_path(path: &str) -> String {
    ASSETS.join(path).to_str().unwrap().to_string()
}

pub fn storage_path(path: &str) -> String {
    STORAGE.join(path).to_str().unwrap().to_string()
}

