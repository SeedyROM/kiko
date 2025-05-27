use kiko::api::{ApiClient, ApiError, HttpApiClient};
use kiko::data::HelloWorld;

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

pub fn create() -> Api {
    Api::new("http://localhost:3030/api/v1") // or your base URL
}
