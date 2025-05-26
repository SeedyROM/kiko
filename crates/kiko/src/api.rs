use gloo_net::http::Response;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Not Found: {0}")]
    NotFound(String),
    #[error("Bad Request: {0}")]
    BadRequest(String),
    #[error("Internal Server Error")]
    InternalServerError,
    #[error("Unauthorized Access")]
    UnauthorizedAccess,
    #[error("Forbidden Access")]
    ForbiddenAccess,
    #[error("Network error: {0}")]
    NetworkError(gloo_net::Error),
    #[error("Parse error: {0}")]
    ParseError(gloo_net::Error),
    #[error("Serialize error: {0}")]
    SerializeError(gloo_net::Error),
    #[error("Unexpected response status code: {0}")]
    UnexpectedStatusCode(u16),
}

type ApiResult<T> = Result<T, ApiError>;

#[derive(Clone, Default)]
pub struct ApiHeaders(HashMap<String, String>);

impl ApiHeaders {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert(&mut self, key: String, value: String) {
        self.0.insert(key, value);
    }

    pub fn delete(&mut self, key: &str) {
        self.0.remove(key);
    }
}

impl From<ApiHeaders> for gloo_net::http::Headers {
    fn from(val: ApiHeaders) -> Self {
        let headers = gloo_net::http::Headers::new();
        for (key, value) in val.0 {
            headers.set(&key, &value);
        }
        headers
    }
}

// Reusable response handling functions
async fn handle_response_status(response: Response, endpoint: &str) -> ApiResult<Response> {
    match response.status() {
        200..=299 => Ok(response),
        400 => Err(ApiError::BadRequest(format!("Bad request to {}", endpoint))),
        401 => Err(ApiError::UnauthorizedAccess),
        403 => Err(ApiError::ForbiddenAccess),
        404 => Err(ApiError::NotFound(format!("{} not found", endpoint))),
        500..=599 => Err(ApiError::InternalServerError),
        status => Err(ApiError::UnexpectedStatusCode(status)),
    }
}

async fn parse_json_response<T>(response: Response) -> ApiResult<T>
where
    T: serde::de::DeserializeOwned,
{
    response.json::<T>().await.map_err(ApiError::ParseError)
}

// Combined function for the common pattern
async fn handle_json_response<T>(response: Response, endpoint: &str) -> ApiResult<T>
where
    T: serde::de::DeserializeOwned,
{
    let validated_response = handle_response_status(response, endpoint).await?;
    parse_json_response(validated_response).await
}

#[async_trait::async_trait(?Send)]
pub trait ApiClient {
    // Core request methods
    async fn make_request(&self, method: HttpMethod, endpoint: &str) -> ApiResult<Response>;

    async fn make_request_with_body<B>(
        &self,
        method: HttpMethod,
        endpoint: &str,
        body: &B,
    ) -> ApiResult<Response>
    where
        B: serde::Serialize;

    // HTTP method implementations
    async fn get<T>(&self, endpoint: &str) -> ApiResult<T>
    where
        T: serde::de::DeserializeOwned;

    async fn post<T, B>(&self, endpoint: &str, body: &B) -> ApiResult<T>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize;

    async fn put<T, B>(&self, endpoint: &str, body: &B) -> ApiResult<T>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize;

    async fn patch<T, B>(&self, endpoint: &str, body: &B) -> ApiResult<T>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize;

    async fn delete<T>(&self, endpoint: &str) -> ApiResult<T>
    where
        T: serde::de::DeserializeOwned;
}

pub struct HttpApiClient {
    root_url: String,
    headers: ApiHeaders,
}

impl HttpApiClient {
    pub fn new(root_url: impl Into<String>) -> Self {
        Self {
            root_url: root_url.into(),
            headers: ApiHeaders::new(),
        }
    }

    pub fn set_header(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }

    pub fn set_headers(&mut self, headers: Vec<(String, String)>) {
        for (key, value) in headers {
            self.headers.insert(key, value);
        }
    }
}

#[async_trait::async_trait(?Send)]
impl ApiClient for HttpApiClient {
    async fn make_request(&self, method: HttpMethod, endpoint: &str) -> ApiResult<Response> {
        let url = format!("{}{}", self.root_url, endpoint);

        let request = match method {
            HttpMethod::Get => gloo_net::http::Request::get(&url),
            HttpMethod::Delete => gloo_net::http::Request::delete(&url),
            _ => return Err(ApiError::UnexpectedStatusCode(405)), // Method not allowed for this function
        };

        request
            .headers(self.headers.clone().into())
            .send()
            .await
            .map_err(ApiError::NetworkError)
    }

    async fn make_request_with_body<B>(
        &self,
        method: HttpMethod,
        endpoint: &str,
        body: &B,
    ) -> ApiResult<Response>
    where
        B: serde::Serialize,
    {
        let url = format!("{}{}", self.root_url, endpoint);

        let request = match method {
            HttpMethod::Post => gloo_net::http::Request::post(&url),
            HttpMethod::Put => gloo_net::http::Request::put(&url),
            HttpMethod::Patch => gloo_net::http::Request::patch(&url),
            _ => return Err(ApiError::UnexpectedStatusCode(405)), // Method not allowed for this function
        };

        request
            .headers(self.headers.clone().into())
            .json(body)
            .map_err(ApiError::SerializeError)?
            .send()
            .await
            .map_err(ApiError::NetworkError)
    }

    async fn get<T>(&self, endpoint: &str) -> ApiResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let response = self.make_request(HttpMethod::Get, endpoint).await?;
        handle_json_response(response, endpoint).await
    }

    async fn post<T, B>(&self, endpoint: &str, body: &B) -> ApiResult<T>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        let response = self
            .make_request_with_body(HttpMethod::Post, endpoint, body)
            .await?;
        handle_json_response(response, endpoint).await
    }

    async fn put<T, B>(&self, endpoint: &str, body: &B) -> ApiResult<T>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        let response = self
            .make_request_with_body(HttpMethod::Put, endpoint, body)
            .await?;
        handle_json_response(response, endpoint).await
    }

    async fn patch<T, B>(&self, endpoint: &str, body: &B) -> ApiResult<T>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        let response = self
            .make_request_with_body(HttpMethod::Patch, endpoint, body)
            .await?;
        handle_json_response(response, endpoint).await
    }

    async fn delete<T>(&self, endpoint: &str) -> ApiResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let response = self.make_request(HttpMethod::Delete, endpoint).await?;
        handle_json_response(response, endpoint).await
    }
}
