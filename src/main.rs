mod error;
mod routes;
mod service;

use bollard::Docker;
use error::{AppError};
use gotcha::{axum::extract::FromRef, config::BasicConfig, ConfigWrapper, GotchaApp, GotchaContext, GotchaRouter};
use routes::{auth, services};
use serde::{Deserialize, Serialize};
use service::{Initializer, JwtManager, ServiceManager, UserManager};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Deserialize, Clone, Serialize, Default)]
pub struct Config {
    docker_sock: String,
    data_dir: String,
    jwt_secret: String,
}

#[derive(Debug, Clone)]
pub struct AppState {
    service_manager: Arc<Mutex<ServiceManager>>,
    user_manager: Arc<Mutex<UserManager>>,
    jwt_manager: Arc<Mutex<JwtManager>>,
}

impl FromRef<GotchaContext<AppState, Config>> for AppState {
    fn from_ref(context: &GotchaContext<AppState, Config>) -> Self {
        context.state.clone()
    }
}

pub struct App {
    // service_manager: ServiceManager,
    // user_manager: UserManager,
    // jwt_manager: JwtManager,
}

impl GotchaApp for App {
    type State = AppState;
    type Config = Config;

    fn config(&self) -> impl std::future::Future<Output = Result<ConfigWrapper<Self::Config>, Box<dyn std::error::Error>>> + Send {
        async move {
            let config = Config {
                docker_sock: "unix:///var/run/docker.sock".to_string(),
                data_dir: "./data".to_string(),
            jwt_secret: "your-secret-key".to_string(), // In production, use a secure secret
        };
        Ok(ConfigWrapper{basic: BasicConfig { host: "0.0.0.0".to_string(), port: 3000 }, application: config})
        }
    }
    fn routes(
        &self,
        router: GotchaRouter<GotchaContext<AppState, Config>>,
    ) -> GotchaRouter<GotchaContext<AppState, Config>> {
        router
            .post("/api/login", auth::login)
            .post("/api/change-password", auth::change_password)
            .get("/api/services", services::list_services)
            .post("/api/services", services::create_service)
            .get("/api/services/:id", services::get_service)
            .put("/api/services/:id", services::update_service)
            .delete("/api/services/:id", services::delete_service)
    }

    async fn state(
        &self,
        config: &ConfigWrapper<Self::Config>,
    ) -> Result<Self::State, Box<dyn std::error::Error>> {
        let docker = Docker::connect_with_unix(
            &config.application.docker_sock,
            120,
            bollard::API_DEFAULT_VERSION,
        )
        .unwrap();
        let service_manager = ServiceManager::new(
            docker,
            &format!("{}/services.json", config.application.data_dir),
        )
        .unwrap();
        let user_manager =
            UserManager::new(&format!("{}/users.json", config.application.data_dir)).unwrap();
        let jwt_manager = JwtManager::new(config.application.jwt_secret.as_bytes());

        let it: Result<Self::State, Box<dyn std::error::Error>> = Ok(AppState {
            service_manager: Arc::new(Mutex::new(service_manager)),
            user_manager: Arc::new(Mutex::new(user_manager)),
            jwt_manager: Arc::new(Mutex::new(jwt_manager)),
        });
        it
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
   
    // Initialize data directory and files
    let initializer = Initializer::new("./data");
    initializer.init()?;

    let app = App {};
    app.run().await?;

    Ok(())
}
