//! Commit prepared resources without deleting the last installation before the rename succeeds.
use std::{fs, path::Path};

/// APK updates may remove/rename original Java sources. Do not leave those in
/// the bundle key, but retain the preparer's independent `.local` work cache.
pub(super) fn reset_kit_sources(dir: &Path) -> Result<(), String> {
    for name in ["mods", "crates", "exporters"] {
        let path = dir.join(name);
        match fs::symlink_metadata(&path) {
            Ok(metadata) => {
                let result = if metadata.is_dir() && !metadata.is_symlink() {
                    fs::remove_dir_all(&path)
                } else {
                    fs::remove_file(&path)
                };
                result.map_err(|e| format!("Cannot refresh preparer sources: {e}"))?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(())
}

pub(super) fn activate(made: &Path, destination: &Path) -> Result<(), String> {
    if made == destination {
        return Ok(());
    }
    let parent = destination
        .parent()
        .ok_or("Resource folder has no parent")?;
    let name = destination
        .file_name()
        .ok_or("Resource folder has no name")?
        .to_string_lossy();
    let backup = parent.join(format!(".{name}.starling-previous"));
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    // Recover a stop between moving the old folder and publishing the new one.
    if backup.exists() && !destination.exists() {
        fs::rename(&backup, destination)
            .map_err(|e| format!("Cannot restore previous resources: {e}"))?;
    }
    if backup.exists() {
        remove_tree(&backup, "Removing the previous game files")
            .map_err(|e| format!("Cannot retire previous resource backup: {e}"))?;
    }
    let previous = destination.exists();
    if previous {
        fs::rename(destination, &backup)
            .map_err(|e| format!("Cannot preserve previous resources: {e}"))?;
    }
    if let Err(error) = fs::rename(made, destination) {
        // The kit is in the app's files and the resources may be in shared storage: a rename
        // across filesystems fails with EXDEV. Copy instead (then the old made folder goes).
        if let Err(copy) = copy_tree(made, destination) {
            let _ = fs::remove_dir_all(destination);
            if previous && let Err(restore) = fs::rename(&backup, destination) {
                return Err(format!(
                    "Cannot activate resources: {error}; copying them failed: {copy}; previous \
                     resources remain at {} (restore failed: {restore})",
                    backup.display()
                ));
            }
            return Err(format!(
                "Cannot activate resources: {error}; copying them failed: {copy}; previous \
                 resources preserved"
            ));
        }
        crate::preparer::report("Cleaning up the work files".into());
        let _ = remove_tree(made, "Cleaning up the work files");
    }
    if previous {
        if let Err(error) = remove_tree(&backup, "Removing the previous game files") {
            log::warn!("Resources installed; old resource backup could not be removed: {error}");
        }
    }
    Ok(())
}

/// `from`'s folders and files under `to` (a destination that does not exist yet), for the
/// rename across filesystems shared storage forces. The count goes to the setup's stage as it
/// goes: copying the whole prepared set takes minutes on a phone, and with no word on screen it
/// looked hung on the last line the preparation printed.
fn copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    crate::preparer::report("Moving the game files to the TML folder".into());
    let total = count_files(from)?;
    let mut done = 0;
    copy_with_progress(from, to, total, &mut done)
}

/// A folder removed with its count on screen: taking tens of thousands of files out of shared
/// storage takes minutes, and silence there read as a hang on the line before it.
fn remove_tree(path: &Path, label: &str) -> Result<(), String> {
    // A word before the count: walking a shared-storage tree to count it is not instant either.
    crate::preparer::report(label.to_string());
    let total = count_files(path)?;
    let mut done = 0;
    remove_with_progress(path, label, total, &mut done)
}

fn remove_with_progress(
    path: &Path,
    label: &str,
    total: u64,
    done: &mut u64,
) -> Result<(), String> {
    let meta = fs::symlink_metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if meta.is_dir() {
        for entry in fs::read_dir(path).map_err(|e| format!("{}: {e}", path.display()))? {
            remove_with_progress(&entry.map_err(|e| e.to_string())?.path(), label, total, done)?;
        }
        fs::remove_dir(path).map_err(|e| format!("{}: {e}", path.display()))
    } else {
        *done += 1;
        if *done == 1 || *done % 100 == 0 || *done == total {
            crate::preparer::report(format!("{label}: {done}/{total}"));
        }
        fs::remove_file(path).map_err(|e| format!("{}: {e}", path.display()))
    }
}

/// The files below `from` (the copy's whole length, for the progress line).
fn count_files(path: &Path) -> Result<u64, String> {
    let meta = fs::symlink_metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if meta.is_dir() {
        let mut total = 0;
        for entry in fs::read_dir(path).map_err(|e| format!("{}: {e}", path.display()))? {
            total += count_files(&entry.map_err(|e| e.to_string())?.path())?;
        }
        Ok(total)
    } else if meta.is_file() {
        Ok(1)
    } else {
        Ok(0)
    }
}

fn copy_with_progress(from: &Path, to: &Path, total: u64, done: &mut u64) -> Result<(), String> {
    let meta = fs::symlink_metadata(from).map_err(|e| format!("{}: {e}", from.display()))?;
    if meta.is_dir() {
        fs::create_dir_all(to).map_err(|e| format!("{}: {e}", to.display()))?;
        for entry in fs::read_dir(from).map_err(|e| format!("{}: {e}", from.display()))? {
            let entry = entry.map_err(|e| e.to_string())?;
            copy_with_progress(&entry.path(), &to.join(entry.file_name()), total, done)?;
        }
    } else if meta.is_file() {
        *done += 1;
        if *done == 1 || *done % 100 == 0 || *done == total {
            crate::preparer::report(format!(
                "Moving the game files to the TML folder: {done}/{total}"
            ));
        }
        fs::copy(from, to).map_err(|e| format!("{}: {e}", to.display()))?;
        if fs::metadata(to).map_err(|e| e.to_string())?.len() != meta.len() {
            return Err(format!("{} came out short", to.display()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn root() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "starling-resource-swap-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }
    #[test]
    fn apk_source_refresh_removes_old_inputs_but_keeps_work_cache() {
        let dir = root();
        for name in ["mods", "crates", "exporters", ".local"] {
            fs::create_dir(dir.join(name)).unwrap();
            fs::write(dir.join(name).join("old"), name).unwrap();
        }
        reset_kit_sources(&dir).unwrap();
        assert_eq!(fs::read(dir.join(".local/old")).unwrap(), b".local");
        for name in ["mods", "crates", "exporters"] {
            assert!(!dir.join(name).exists());
        }
        reset_kit_sources(&dir).unwrap();
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn failed_publish_restores_old_resources_and_a_retry_replaces_them() {
        let dir = root();
        let target = dir.join("resources");
        let made = dir.join("prepared");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("kept"), "old resources").unwrap();
        assert!(activate(&made, &target).is_err());
        assert_eq!(
            fs::read_to_string(target.join("kept")).unwrap(),
            "old resources"
        );
        fs::create_dir(&made).unwrap();
        fs::write(made.join("ready"), "new resources").unwrap();
        activate(&made, &target).unwrap();
        assert_eq!(
            fs::read_to_string(target.join("ready")).unwrap(),
            "new resources"
        );
        assert!(!made.exists());
        assert!(!dir.join(".resources.starling-previous").exists());
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn interrupted_swap_restores_before_a_failed_retry() {
        let dir = root();
        let target = dir.join("resources");
        let backup = dir.join(".resources.starling-previous");
        fs::create_dir(&backup).unwrap();
        fs::write(backup.join("kept"), "last working resources").unwrap();
        assert!(activate(&dir.join("missing"), &target).is_err());
        assert_eq!(
            fs::read_to_string(target.join("kept")).unwrap(),
            "last working resources"
        );
        fs::remove_dir_all(dir).unwrap();
    }
}
