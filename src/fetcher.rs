use std::{
    collections::BTreeMap,
    env,
    fs::{self, DirEntry},
    io,
    path::{Path, PathBuf},
};

type DirMap = io::Result<(BTreeMap<char, Vec<PathBuf>>, Vec<PathBuf>)>;

pub fn get_dirs() -> DirMap {
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

    let char_map = build_char_map(&dirs);

    Ok((char_map, files))
}

fn build_char_map(paths: &[PathBuf]) -> BTreeMap<char, Vec<PathBuf>> {
    let mut map: BTreeMap<char, Vec<PathBuf>> = BTreeMap::new();

    for path in paths {
        if let Some(name) = path.file_name() {
            let c = name.to_string_lossy().chars().next().unwrap();
            map.entry(c).or_default().push(path.clone());
        }
    }

    map
}

fn collect_entries(path: &Path) -> io::Result<Vec<DirEntry>> {
    let mut entries: Vec<DirEntry> = fs::read_dir(path)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    Ok(entries)
}
