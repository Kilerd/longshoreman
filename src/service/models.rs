use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct CreateServiceRequest {
    pub name: String,
    pub image: String,
    pub command: Option<Vec<String>>,
    pub env: Option<Vec<String>>,
    pub ports: Option<Vec<PortMapping>>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct PortMapping {
    pub host_port: u16,
    pub container_port: u16,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Service {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub command: Option<Vec<String>>,
    pub env: Option<Vec<String>>,
    pub ports: Option<Vec<PortMapping>>,
} 