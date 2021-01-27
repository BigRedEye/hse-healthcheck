use crate::prelude::*;
use crate::config;
use crate::models;

use sqlx::postgres::PgPoolOptions;

#[derive(Clone)]
pub struct Repo {
    pool: sqlx::Pool<sqlx::Postgres>,
}

impl Repo {
    pub async fn new(conf: &config::Settings) -> Result<Repo> {
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .connect(&conf.database_url)
            .await?;

        let repo = Repo { pool };

        repo.migrate().await?;

        Ok(repo)
    }

    async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("src/migrations")
            .run(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn list_statuses(&self) -> Result<Vec<models::NodeStatus>> {
        let res: Vec<models::NodeStatus> = sqlx::query_as("SELECT * FROM statuses")
            .fetch_all(&self.pool)
            .await?;

        Ok(res)
    }

    pub async fn update_status(&self, status: models::NodeStatus) -> Result<()> {
        sqlx::query("INSERT INTO statuses(ip, status, timestamp) VALUES ($1, $2, $3) ON CONFLICT (ip) DO UPDATE SET timestamp = $3")
            .bind(status.ip)
            .bind(status.status)
            .bind(status.timestamp)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
