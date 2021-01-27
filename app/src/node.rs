use crate::prelude::*;
use crate::api;
use crate::config;
use crate::repo;
use crate::models;

use chrono::prelude::*;
use actix_web::{get, web, App, HttpServer, HttpResponse, Responder};

#[derive(Clone)]
pub struct Node {
    conf: config::Settings,
    repo: repo::Repo,
    ip: String,
}

const METADATA_URL: &str = "http://169.254.169.254/latest/meta-data/local-ipv4";

impl Node {
    pub async fn new(conf: config::Settings) -> Result<Self> {
        let repo = repo::Repo::new(&conf).await?;
        let ip = Self::local_ip().await;

        Ok(Node{ conf, repo, ip })
    }

    pub async fn run(&self) -> Result<()> {
        let clone = self.clone();
        actix_rt::spawn(async move {
            clone.run_heartbeats().await
        });

        self.run_service().await
    }

    async fn local_ip() -> String {
        match Self::discover_ip().await {
            Ok(ip) => {
                log::info!("Discovered local ip: {}", ip);
                ip
            },
            Err(error) => {
                let fake = fakedata_generator::gen_ipv4();
                log::warn!("Failed to discover local ip: {}, going to use fake ip {}", error, fake);
                fake
            }
        }
    }

    async fn discover_ip() -> Result<String> {
        log::debug!("Going to send ip discovery request");

        let client = reqwest::ClientBuilder::new()
            .timeout(std::time::Duration::from_millis(1000))
            .build()?;

        let ip = client.get(METADATA_URL)
            .send()
            .await?
            .text()
            .await?;

        log::debug!("Successfully discovered ip {}", ip);

        Ok(ip)
    }

    async fn run_heartbeats(&self) -> ! {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(1000));
            self.send_heartbeat().await;
        }
    }

    async fn send_heartbeat(&self) {
        match self.repo.update_status(models::NodeStatus::new(self.ip.clone())).await {
            Ok(()) => log::info!("Successfully sent heartbeat"),
            Err(err) => log::warn!("Failed to send heartbeat: {}", err),
        };
    }

    async fn run_service(&self) -> Result<()> {
        log::info!("Starting node service at {}", &self.conf.bind_address);

        let clone = self.clone();
        HttpServer::new(move || App::new().data(clone.clone()).service(healthcheck))
            .bind(self.conf.bind_address)?
            .run()
            .await?;

        Ok(())
    }

    async fn healthcheck(&self) -> impl Responder {
        match self.process_healthcheck().await {
            Ok(status) => HttpResponse::Ok().json(status),
            Err(e) => {
                log::error!("Healthcheck failed: {:?}", e);
                HttpResponse::ServiceUnavailable().json(api::Error{ error: e.to_string() })
            },
        }
    }

    async fn process_healthcheck(&self) -> Result<api::Status> {
        let first_timepoint = Utc::now() - chrono::Duration::seconds(10);

        let res = self
            .repo
            .list_statuses()
            .await?
            .into_iter()
            .filter(|status| status.timestamp > first_timepoint)
            .map(|status| api::Service{
                ip: status.ip,
                status: status.status,
            })
            .collect::<Vec<_>>();

        Ok(api::Status{
            ip: self.ip.clone(),
            services: res,
        })
    }
}

#[get("/healthcheck")]
async fn healthcheck(node: web::Data<Node>) -> impl Responder {
    node.healthcheck().await
}
