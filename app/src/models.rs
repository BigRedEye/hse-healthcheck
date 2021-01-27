use chrono::prelude::*;

#[derive(sqlx::FromRow)]
pub struct NodeStatus {
    pub ip: String,
    pub status: String,
    pub timestamp: DateTime<Utc>,
}

impl NodeStatus  {
    pub fn new(ip: String) -> NodeStatus {
        NodeStatus {
            ip,
            status: "AVAILABLE".into(),
            timestamp: Utc::now(),
        }
    }
}
