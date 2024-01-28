use std::path::Path;
use serde_derive::Deserialize;
use actix_web::{post, App, HttpResponse, HttpServer, Responder,
                web, rt, middleware::Logger};
use awc::Client;
use clap::Parser;

mod errors;
mod github;
mod cleanup;
mod config;
mod args;

#[derive(Deserialize)]
struct PreviewParams {
    job: String,
}

/// Handler of the HTTP server /preview/ endpoint.
///
/// This gets the parameters from the url and adds a job to the queue
/// to be processed by the worker.
#[post("/submit/{github_owner}/{github_repo}/{pull_request_number}/")]
async fn preview_handler(params: web::Path<(String, String, u64)>,
                         query_params: web::Query<PreviewParams>,
                         client: web::Data<Client>,
                         settings: web::Data<config::SettingsPerThread>) -> impl Responder {
    let github_owner = &params.0;
    let github_repo = &params.1;
    let pull_request_number = params.2;
    let job_name = &query_params.job;

    if !settings.github_allowed_owners.is_empty()
        && !settings.github_allowed_owners.contains(github_owner)  {
            log::info!("Organization {} made a request but is not authorized",
                       github_owner);
            return HttpResponse::InternalServerError().body(format!(
                "GitHub organization {} is not allowed to use this server",
                github_owner
            ));

    }

    let base_api_url = format!(
        "{}{github_owner}/{github_repo}/",
        settings.github_endpoint
    );

    let target_dir = Path::new(&settings.previews_path)
                        .join(github_owner)
                        .join(github_repo)
                        .join(pull_request_number.to_string());

    let publish_url = format!(
        "{}{github_owner}/{github_repo}/{pull_request_number}/",
        settings.server_url
    );

    match github::publish_artifact(&client,
                                   &base_api_url,
                                   pull_request_number,
                                   job_name,
                                   &target_dir,
                                   settings.max_artifact_size).await {
        Ok(_) => {
            rt::spawn(async move {
                match cleanup::clean_up_old_previews(Path::new(&settings.previews_path),
                                                     settings.retention_days).await {
                    Ok(num_deleted) => {
                        if num_deleted > 0 {
                            log::info!("[PR {}] Clean up processes deleted {} previews",
                                       pull_request_number,
                                       num_deleted);
                        }
                    }
                    Err(e) => {
                        log::error!("[PR {}] Error while cleaning up previews directory {:?}",
                                    pull_request_number,
                                    e);
                    }
                }
            });
            HttpResponse::Ok().body(
                format!("Website preview of this PR available at: {}", publish_url)
            )
        }
        Err(e) => {
            log::error!("[PR {}] {:?}", pull_request_number, e);
            HttpResponse::InternalServerError().body(format!("{:?}", e))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let parsed_args = args::Args::parse();
    let config_path = Path::new(&parsed_args.config_file);
    let settings = config::Settings::load(config_path);

    env_logger::init_from_env(
        env_logger::Env::default().default_filter_or(&settings.log_level)
    );

    HttpServer::new(move || {
        let client = Client::builder()
            .add_default_header(("Accept",
                                 "application/vnd.github+json"))
            .add_default_header(("Authorization",
                                 format!("Bearer {}", &settings.github_token)))
            .add_default_header(("X-GitHub-Api-Version",
                                 "2022-11-28"))
            .add_default_header(("User-Agent",
                                 "pandas-doc-previewer"))
            .finish();

        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(client.clone()))
            .app_data(web::Data::new(settings.per_thread.clone()))
            .service(preview_handler)
    })
    .bind((settings.server_address, settings.server_port))?
    .run()
    .await
}
