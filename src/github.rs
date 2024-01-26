use crate::errors::PreviewerError;
use std::fs;
use std::path::Path;
use std::io::Cursor;
use awc::Client;

/// Fetch the `url` and return the response as a parsed JSON object.
async fn fetch_json(client: &Client,
                    url: &str) -> Result<serde_json::Value,
                                         PreviewerError> {
    let mut resp = client.get(url).send().await?;
    if resp.status() != 200 {
        return Err(PreviewerError::StatusError { url: url.to_owned(),
                                                 status_code: resp.status() });
    }
    Ok(resp.json::<serde_json::Value>().await?)
}

/// For a given pull request, return the reference (hash) or the last commit
async fn last_commit_from_pr(client: &Client,
                             base_api_url: &str,
                             pull_request_number: u64) -> Result<String,
                                                                 PreviewerError> {
    let url = format!("{base_api_url}pulls/{pull_request_number}/commits");
    let json_obj = fetch_json(client, &url).await?;

    match json_obj.as_array().and_then(|x| x.last()) {
        Some(last_commit) => {
            return last_commit["sha"].as_str()
                                     .map(|x| x.to_owned())
                                     .ok_or(
                PreviewerError::ResponseContentError(
                    "last commmit is not a string".to_owned(),
                    json_obj)
            );
        }
        None => { return Err(PreviewerError::ResponseContentError(
                             "No commits found".to_owned(),
                             json_obj)) }
    }
}

/// Extract the run id from the details_url returned by the checks run API
///
/// The url has the form:
/// `https://github.com/pandas-dev/pandas/actions/runs/{run_id}/job/{*}`
fn extract_run_id_from_detail_url(url: &str) -> Result<u64, PreviewerError> {
    let pattern = "/actions/runs/";

    if let Some(run_id_start_idx) = url.find(&pattern) {
        let url_end = &url[run_id_start_idx + pattern.len()..];
        if let Some(run_id_end_idx) = url_end.find("/") {
            if let Ok(run_id) = url_end[..run_id_end_idx].parse::<u64>() {
                return Ok(run_id);
            }
        }
    }
    Err(PreviewerError::PatternNotFound(format!("Run id not found in: {}", url)))
}

async fn run_id_from_commit(client: &Client,
                            base_api_url: &str,
                            commit_reference: &str) -> Result<u64,
                                                              PreviewerError> {
    let url = format!("{base_api_url}commits/{commit_reference}/check-runs");
    let json_obj = fetch_json(client, &url).await?;

    match json_obj["check_runs"]
        .as_array()
        .and_then(|jobs| jobs.iter().find(|job| job["name"] == "Doc Build and Upload")) {
            Some(job) => {
                if let Some(detail_url) = job["details_url"].as_str() {
                    extract_run_id_from_detail_url(&detail_url)
                } else {
                    Err(PreviewerError::ResponseContentError(
                        "details_url not found".to_owned(),
                        json_obj))
                    }
            }
            None => {
                Err(PreviewerError::ResponseContentError(
                    "no check runs found".to_owned(),
                    json_obj))
            }
    }
}

/// Obtain the url to download the artifact given a `run_id`
async fn artifact_url_from_run_id(client: &Client,
                                  base_api_url: &str,
                                  run_id: u64) -> Result<String,
                                                         PreviewerError> {
    let url = format!("{base_api_url}actions/runs/{run_id}/artifacts");
    let json_obj = fetch_json(client, &url).await?;

    match json_obj["artifacts"].as_array() {
        Some(artifacts) => {
            if artifacts.len() == 1 {
                if let Some(artifact_url) = artifacts[0]["archive_download_url"].as_str() {
                    return Ok(artifact_url.to_owned());
                } else {
                    return Err(PreviewerError::ResponseContentError(
                               "artifact url is not a string".to_owned(),
                               json_obj));
                }
            } else {
                return Err(PreviewerError::ResponseContentError(
                           format!("Expected 1 artifact, {} found", artifacts.len()),
                           json_obj));
            }
        }
        None => { return Err(PreviewerError::ResponseContentError(
                             "no artifacts found".to_owned(),
                             json_obj)) }
    }
}

/// Download the artifact given its `url` and extract in `target_dir`
async fn download_artifact(client: &Client,
                           url: &str,
                           target_dir: &Path,
                           max_artifact_size: usize) -> Result<(),
                                                               PreviewerError> {
    let mut resp = client.get(url).send().await?;
    let artifact_content = resp.body().limit(max_artifact_size).await?;
    let mut zip_archive = zip::ZipArchive::new(Cursor::new(artifact_content))?;
    zip_archive.extract(&target_dir)?;

    // Temporary creating a file inside the directory, so the directory mtime is updated
    // This will make sure the directory is not cleaned up when it has recent changes
    let temp_file = target_dir.join(".update_mtime");
    fs::File::create(&temp_file)?;
    fs::remove_file(&temp_file).ok();

    Ok(())
}

pub async fn publish_artifact(client: &Client,
                              base_api_url: &str,
                              pull_request_number: u64,
                              target_dir: &Path,
                              max_artifact_size: usize) -> Result<(),
                                                           PreviewerError> {

    log::info!("[PR {}] Preview requested", pull_request_number);

    let last_commit = last_commit_from_pr(&client,
                                          &base_api_url,
                                          pull_request_number).await?;
    log::info!("[PR {}] Last commit: {}", pull_request_number, last_commit);

    let run_id = run_id_from_commit(&client,
                                    &base_api_url,
                                    &last_commit).await?;
    log::info!("[PR {}] Run id: {}", pull_request_number, run_id);

    let artifact_url = artifact_url_from_run_id(&client,
                                                &base_api_url,
                                                run_id).await?;
    log::info!("[PR {}] Artifact url: {}", pull_request_number, artifact_url);


    log::info!("[PR {}] Starting artifact download", pull_request_number);
    download_artifact(&client, &artifact_url, &target_dir, max_artifact_size).await?;
    log::info!("[PR {}] Artifact downloaded to {:?}", pull_request_number, target_dir);

    Ok(())
}
