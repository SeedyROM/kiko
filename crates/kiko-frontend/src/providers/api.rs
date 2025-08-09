use kiko::api::{ApiClient, ApiClientHttp, ApiError};
use kiko::data::{self, HelloWorld};

/// The main API client for the Kiko application, providing methods to interact with the backend API.
pub struct Api {
    client: ApiClientHttp,
}

impl Api {
    pub fn new(base_url: &str) -> Self {
        Api {
            client: ApiClientHttp::new(base_url),
        }
    }

    pub async fn fetch_hello(&self) -> Result<HelloWorld, ApiError> {
        self.client.get("/hello").await
    }

    pub async fn create_session(
        &self,
        create_session: &data::CreateSession,
    ) -> Result<data::Session, ApiError> {
        self.client.post("/session", create_session).await
    }

    pub async fn fetch_session(&self, session_id: &str) -> Result<data::Session, ApiError> {
        self.client.get(&format!("/session/{session_id}")).await
    }
}

/// Create a new instance of the API client with the default base URL.
pub fn create() -> Api {
    Api::new("http://localhost:3030/api/v1") // or your base URL
}
