use std::{
    collections::BTreeMap,
    env,
    fs::{self, DirEntry},
    io,
    path::{Path, PathBuf},
};

pub fn get_dirs_files() -> io::Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let path = env::current_dir()?;
    let entries = collect_entries(&path)?;

    let mut dirs = Vec::new();
    let mut files = Vec::new();

    for entry in entries {
        let meta = entry.metadata()?;
        if meta.is_dir() {
            dirs.push(entry.path());
        } else {
            files.push(entry.path());
        }
    }

    Ok((dirs, files))
}

pub fn build_char_map(paths: &[PathBuf]) -> BTreeMap<char, Vec<PathBuf>> {
    let mut map: BTreeMap<char, Vec<PathBuf>> = BTreeMap::new();

    for path in paths {
        if let Some(name) = path.file_name() {
            let c = name.to_string_lossy().chars().next().unwrap();
            map.entry(c).or_default().push(path.clone());
        }
    }

    map
}

pub fn starts_with(dirs: &[PathBuf], prefix: &str) -> Vec<PathBuf> {
    dirs.iter()
        .filter(|dir| {
            dir.file_name()
                .map(|os_str| os_str.to_string_lossy().starts_with(prefix))
                .unwrap_or(false)
        })
        .cloned()
        .collect()
}

fn collect_entries(path: &Path) -> io::Result<Vec<DirEntry>> {
    let mut entries: Vec<DirEntry> = fs::read_dir(path)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    Ok(entries)
}

pub fn is_directory(path: &Path) -> bool {
    path.metadata().map(|meta| meta.is_dir()).unwrap_or(false)
}

pub fn is_empty(path: &Path) -> bool {
    if !is_directory(path) {
        return false;
    }

    fs::read_dir(path)
        .map(|mut entries| entries.next().is_none())
        .unwrap_or(false)
}

pub fn get_extension(path: &Path) -> Option<String> {
    path.extension()
        .map(|ext| ext.to_string_lossy().to_string())
}

pub fn get_name(path: &Path) -> String {
    path.file_name()
        .map(|os| os.to_string_lossy().to_string())
        .unwrap_or_default()
}
