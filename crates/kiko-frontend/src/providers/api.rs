use kiko::api::{ApiClient, ApiClientHttp, ApiError};
use kiko::data::{CreateSession, Session};

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

    pub async fn create_session(
        &self,
        create_session: &CreateSession,
    ) -> Result<Session, ApiError> {
        self.client.post("/session", create_session).await
    }

    pub async fn fetch_session(&self, session_id: &str) -> Result<Session, ApiError> {
        self.client.get(&format!("/session/{session_id}")).await
    }
}

/// Create a new instance of the API client with the default base URL.
pub fn create() -> Api {
    // For WASM, we need to construct the URL dynamically from the current location
    let base_url = if web_sys::window().is_some() {
        // Use relative path that will be proxied by nginx
        "/api/v1".to_string()
    } else {
        "http://localhost:3030/api/v1".to_string()
    };
    Api::new(&base_url)
}
