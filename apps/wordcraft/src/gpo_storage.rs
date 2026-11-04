//! Paths in a template pack are relative to its root, so moving a folder needs no path edits.
use std::path::{Path, PathBuf};

pub fn portable_dir(executable: &Path) -> Option<PathBuf> {
    let parent = executable.parent()?;
    parent.join("GPOWord.portable").is_file().then(|| parent.to_path_buf())
}

pub fn discover_root(executable: &Path, cwd: &Path, saved: Option<&Path>) -> PathBuf {
    let near_exe = executable.parent().unwrap_or(cwd).join("docsStorage");
    if portable_dir(executable).is_some() && near_exe.is_dir() {
        return near_exe;
    }
    if let Some(path) = saved {
        let path = if path.is_absolute() { path.to_path_buf() } else { executable.parent().unwrap_or(cwd).join(path) };
        if path.is_dir() {
            return path;
        }
    }
    if near_exe.is_dir() {
        return near_exe;
    }
    let local = cwd.join("docsStorage");
    if local.is_dir() {
        return local;
    }
    for ancestor in executable.ancestors().skip(1).take(4) {
        let candidate = ancestor.join("docsStorage");
        if candidate.is_dir() {
            return candidate;
        }
    }
    local
}

pub fn resolve_template(root: &Path, stored: &str) -> Option<PathBuf> {
    if stored.is_empty() {
        return None;
    }
    let path = Path::new(stored);
    if path.is_absolute() {
        return path.is_file().then(|| path.to_path_buf());
    }
    if path.components().any(|c| !matches!(c, std::path::Component::Normal(_))) {
        return None;
    }
    let path = root.join(path);
    let base = root.canonicalize().ok()?;
    let canonical = path.canonicalize().ok()?;
    (canonical.starts_with(base) && canonical.is_file()).then_some(path)
}

pub fn store_template(root: &Path, selected: &Path) -> String {
    selected.strip_prefix(root).unwrap_or(selected).to_string_lossy().replace('\\', "/")
}

pub fn store_root(executable: &Path, root: &Path) -> String {
    if let Some(parent) = executable.parent()
        && let Ok(relative) = root.strip_prefix(parent)
    {
        return relative.to_string_lossy().replace('\\', "/");
    }
    root.to_string_lossy().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("gpo-path-{}-{name}", std::process::id()));
        std::fs::create_dir_all(root.join("docsStorage/WI")).unwrap();
        root
    }
    #[test]
    fn moved_pack_resolves_the_same_relative_template() {
        let a = fixture("a");
        let b = fixture("b");
        std::fs::write(a.join("docsStorage/WI/master.doc"), b"source").unwrap();
        std::fs::write(b.join("docsStorage/WI/master.doc"), b"source").unwrap();
        let stored = store_template(&a.join("docsStorage"), &a.join("docsStorage/WI/master.doc"));
        assert_eq!(stored, "WI/master.doc");
        assert_eq!(resolve_template(&b.join("docsStorage"), &stored).unwrap(), b.join("docsStorage/WI/master.doc"));
    }
    #[test]
    fn portable_marker_overrides_saved_path_from_another_machine() {
        let a = fixture("portable");
        let other = fixture("old-machine");
        std::fs::write(a.join("GPOWord.portable"), b"1").unwrap();
        assert_eq!(discover_root(&a.join("GPOWord.exe"), &other, Some(&other.join("docsStorage"))), a.join("docsStorage"));
        assert_eq!(store_root(&a.join("GPOWord.exe"), &a.join("docsStorage")), "docsStorage");
    }
    #[test]
    fn stale_saved_path_falls_back_to_the_executable_folder() {
        let a = fixture("stale");
        assert_eq!(discover_root(&a.join("GPOWord.exe"), &a, Some(Path::new("missing-docs-folder"))), a.join("docsStorage"));
    }
    #[test]
    fn relative_traversal_is_refused() {
        let a = fixture("traversal");
        assert!(resolve_template(&a.join("docsStorage"), "../outside.doc").is_none());
    }
}
