use kiko::api::{ApiClient, ApiError, HttpApiClient};
use kiko::data::HelloWorld;

/// The main API client for the Kiko application, providing methods to interact with the backend API.
pub struct Api {
    client: HttpApiClient,
}

impl Api {
    pub fn new(base_url: &str) -> Self {
        Api {
            client: HttpApiClient::new(base_url),
        }
    }

    pub async fn fetch_hello(&self) -> Result<HelloWorld, ApiError> {
        self.client.get("/hello").await
    }
}

/// Create a new instance of the API client with the default base URL.
pub fn create() -> Api {
    Api::new("http://localhost:3030/api/v1") // or your base URL
}
