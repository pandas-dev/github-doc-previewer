use crate::errors;
use std::fs;
use std::path::Path;

pub async fn clean_up_old_previews(target_dir: &Path,
                                   retention_days: f64) -> Result<u64, errors::PreviewerError> {
    let mut deletion_counter = 0;
    for owner in fs::read_dir(target_dir)? {
        let owner_path = owner?.path();
        let owner_metadata = fs::metadata(&owner_path)?;
        if !owner_metadata.is_dir() { continue };
        for repo in fs::read_dir(&owner_path)? {
            let repo_path = repo?.path();
            let repo_metadata = fs::metadata(&repo_path)?;
            if !repo_metadata.is_dir() { continue };
            for preview in fs::read_dir(&repo_path)? {
                let preview_path = preview?.path();
                let preview_metadata = fs::metadata(&preview_path)?;
                if !preview_metadata.is_dir() { continue };
                if let Some(_) = preview_path.file_name()
                                             .and_then(|x| x.to_str())
                                             .and_then(|x| x.parse::<u64>().ok()) {
                    let last_modified = preview_metadata.modified()?
                                                        .elapsed()?
                                                        .as_secs() as f64
                                                        / 60. / 60. / 24.;
                    if last_modified > retention_days {
                        log::info!("Deleting directory {:?} ({:.2}d > {:.2}d)",
                                   &preview_path,
                                   last_modified,
                                   retention_days);
                        fs::remove_dir_all(&preview_path)?;
                        deletion_counter += 1;
                    }
                }
            }
        }
    }
    Ok(deletion_counter)
}
