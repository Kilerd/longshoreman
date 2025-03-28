use bollard::container::{Config, CreateContainerOptions};
use bollard::Docker;
use std::fs;
use std::path::Path;

use crate::error::AppError;
use crate::service::models::{CreateServiceRequest, Service};

type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug)]
pub struct ServiceManager {
    services: Vec<Service>,
    file_path: String,
    docker: Docker,
}

impl ServiceManager {
    pub fn new(docker: Docker, file_path: &str) -> Result<Self> {
        let services = if Path::new(file_path).exists() {
            let contents = fs::read_to_string(file_path)?;
            serde_json::from_str(&contents)?
        } else {
            Vec::new()
        };

        Ok(Self {
            services,
            file_path: file_path.to_string(),
            docker,
        })
    }

    fn save(&self) -> Result<()> {
        let contents = serde_json::to_string_pretty(&self.services)?;
        fs::write(&self.file_path, contents)?;
        Ok(())
    }

    pub async fn list_services(&self) -> Result<Vec<Service>> {
        Ok(self.services.clone())
    }

    pub async fn create_service(&mut self, request: CreateServiceRequest) -> Result<Service> {
        if self.services.iter().any(|s| s.name == request.name) {
            return Err(AppError::Service(format!("Service with name {} already exists", request.name)));
        }

        let mut config = Config {
            image: Some(request.image.clone()),
            cmd: request.command.clone(),
            env: request.env.clone(),
            ..Default::default()
        };

        // if let Some(ports) = &request.ports {
        //     let mut port_bindings = std::collections::HashMap::new();
        //     for port in ports {
        //         port_bindings.insert(
        //             format!("{}/tcp", port.container_port),
        //             Some(vec![bollard::models::PortBinding {
        //                 host_ip: Some("0.0.0.0".to_string()),
        //                 host_port: Some(port.host_port.to_string()),
        //             }]),
        //         );
        //     }
        //     config.exposed_ports = Some(port_bindings);
        // }

        let container = self
            .docker
            .create_container(
                Some(CreateContainerOptions {
                    name: request.name.clone(),
                    ..Default::default()
                }),
                config,
            )
            .await?;

        let service = Service {
            id: container.id.clone(),
            name: request.name,
            image: request.image,
            status: "created".to_string(),
            command: request.command,
            env: request.env,
            ports: request.ports,
        };

        self.services.push(service.clone());
        self.save()?;

        Ok(service)
    }

    pub async fn get_service(&self, id: &str) -> Result<Service> {
        self.services
            .iter()
            .find(|s| s.id == id)
            .cloned()
            .ok_or_else(|| AppError::Service("Service not found".to_string()))
    }

    pub async fn update_service(&mut self, id: &str, request: CreateServiceRequest) -> Result<Service> {
        let index = self
            .services
            .iter()
            .position(|s| s.id == id)
            .ok_or_else(|| AppError::Service("Service not found".to_string()))?;

        // Delete the old container
        self.docker
            .remove_container(
                id,
                Some(bollard::container::RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await?;

        // Create new container
        let mut config = Config {
            image: Some(request.image.clone()),
            cmd: request.command.clone(),
            env: request.env.clone(),
            ..Default::default()
        };

        // if let Some(ports) = &request.ports {
        //     let mut port_bindings = std::collections::HashMap::new();
        //     for port in ports {
        //         port_bindings.insert(
        //             format!("{}/tcp", port.container_port),
        //             Some(vec![bollard::container::PortBinding {
        //                 host_ip: Some("0.0.0.0".to_string()),
        //                 host_port: Some(port.host_port.to_string()),
        //             }]),
        //         );
        //     }
        //     config.exposed_ports = Some(port_bindings);
        // }

        let container = self
            .docker
            .create_container(
                Some(CreateContainerOptions {
                    name: request.name.clone(),
                    ..Default::default()
                }),
                config,
            )
            .await?;

        let service = Service {
            id: container.id.clone(),
            name: request.name,
            image: request.image,
            status: "created".to_string(),
            command: request.command,
            env: request.env,
            ports: request.ports,
        };

        self.services[index] = service.clone();
        self.save()?;

        Ok(service)
    }

    pub async fn delete_service(&mut self, id: &str) -> Result<()> {
        let index = self
            .services
            .iter()
            .position(|s| s.id == id).expect("can't find service");

        self.docker
            .remove_container(
                id,
                Some(bollard::container::RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await?;

        self.services.remove(index);
        self.save()?;
        Ok(())
    }
} 