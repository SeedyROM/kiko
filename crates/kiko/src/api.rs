//! A flexible REST API client built on top of `gloo-net` for WebAssembly applications.
//!
//! This module provides a trait-based approach to making HTTP requests with automatic
//! JSON serialization/deserialization, comprehensive error handling, and customizable headers.
//!
//! # Example
//! ```compile_fail
//! use your_crate::api::{ApiClientHttp, ApiClient};
//!
//! let mut client = ApiClientHttp::new("https://api.example.com");
//! client.set_header("Authorization".to_string(), "Bearer token123".to_string());
//!
//! // GET request
//! let user: User = client.get("/users/123").await?;
//!
//! // POST request with body
//! let new_user = CreateUserRequest { name: "John".to_string() };
//! let created_user: User = client.post("/users", &new_user).await?;
//! ```

use gloo_net::http::Response;
use std::collections::HashMap;

/// HTTP methods supported by the API client.
#[derive(Debug, Clone, Copy)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

/// Comprehensive error types for API operations.
///
/// These errors cover common HTTP status codes as well as network and parsing errors
/// that can occur during API interactions.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// Resource not found (404)
    #[error("Not Found: {0}")]
    NotFound(String),
    /// Bad request (400)
    #[error("Bad Request: {0}")]
    BadRequest(String),
    /// Internal server error (5xx)
    #[error("Internal Server Error")]
    InternalServerError,
    /// Unauthorized access (401)
    #[error("Unauthorized Access")]
    UnauthorizedAccess,
    /// Forbidden access (403)
    #[error("Forbidden Access")]
    ForbiddenAccess,
    /// Network-related errors during request
    #[error("Network error: {0}")]
    NetworkError(gloo_net::Error),
    /// JSON parsing errors
    #[error("Parse error: {0}")]
    ParseError(gloo_net::Error),
    /// JSON serialization errors
    #[error("Serialize error: {0}")]
    SerializeError(gloo_net::Error),
    /// Unexpected HTTP status codes
    #[error("Unexpected response status code: {0}")]
    UnexpectedStatusCode(u16),
}

/// Result type alias for API operations.
type ApiResult<T> = Result<T, ApiError>;

/// A wrapper around HTTP headers for type safety and convenience.
///
/// This struct provides a simplified interface for managing HTTP headers
/// while maintaining compatibility with `gloo-net`'s header system.
#[derive(Clone, Default)]
pub struct ApiHeaders(HashMap<String, String>);

impl ApiHeaders {
    /// Creates a new empty set of headers.
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Inserts a header key-value pair.
    ///
    /// If the key already exists, the value will be replaced.
    pub fn insert(&mut self, key: String, value: String) {
        self.0.insert(key, value);
    }

    /// Removes a header by key.
    pub fn delete(&mut self, key: &str) {
        self.0.remove(key);
    }
}

/// Converts ApiHeaders to gloo-net's Headers format.
impl From<ApiHeaders> for gloo_net::http::Headers {
    fn from(val: ApiHeaders) -> Self {
        let headers = gloo_net::http::Headers::new();
        for (key, value) in val.0 {
            headers.set(&key, &value);
        }
        headers
    }
}

/// Validates HTTP response status codes and converts them to appropriate errors.
///
/// # Arguments
/// * `response` - The HTTP response to validate
/// * `endpoint` - The endpoint that was called (used for error messages)
///
/// # Returns
/// The original response if successful, or an ApiError for non-success status codes
async fn handle_response_status(response: Response, endpoint: &str) -> ApiResult<Response> {
    match response.status() {
        200..=299 => Ok(response),
        400 => Err(ApiError::BadRequest(format!("Bad request to {endpoint}"))),
        401 => Err(ApiError::UnauthorizedAccess),
        403 => Err(ApiError::ForbiddenAccess),
        404 => Err(ApiError::NotFound(format!("{endpoint} not found"))),
        500..=599 => Err(ApiError::InternalServerError),
        status => Err(ApiError::UnexpectedStatusCode(status)),
    }
}

/// Parses a JSON response into the specified type.
///
/// # Type Parameters
/// * `T` - The type to deserialize the JSON response into
///
/// # Arguments
/// * `response` - The HTTP response containing JSON data
///
/// # Returns
/// The deserialized object or a ParseError if deserialization fails
async fn parse_json_response<T>(response: Response) -> ApiResult<T>
where
    T: serde::de::DeserializeOwned,
{
    response.json::<T>().await.map_err(ApiError::ParseError)
}

/// Combines response validation and JSON parsing into a single operation.
///
/// This is a convenience function that handles the common pattern of validating
/// an HTTP response and then parsing it as JSON.
///
/// # Type Parameters
/// * `T` - The type to deserialize the JSON response into
///
/// # Arguments
/// * `response` - The HTTP response to process
/// * `endpoint` - The endpoint that was called (used for error messages)
async fn handle_json_response<T>(response: Response, endpoint: &str) -> ApiResult<T>
where
    T: serde::de::DeserializeOwned,
{
    let validated_response = handle_response_status(response, endpoint).await?;
    parse_json_response(validated_response).await
}

/// A trait defining the interface for making HTTP API requests.
///
/// This trait provides both low-level request methods and high-level convenience
/// methods for common HTTP operations with automatic JSON handling.
#[async_trait::async_trait(?Send)]
pub trait ApiClient {
    /// Makes a basic HTTP request without a body.
    ///
    /// Suitable for GET and DELETE requests.
    ///
    /// # Arguments
    /// * `method` - The HTTP method to use
    /// * `endpoint` - The API endpoint to call
    async fn make_request(&self, method: HttpMethod, endpoint: &str) -> ApiResult<Response>;

    /// Makes an HTTP request with a JSON body.
    ///
    /// Suitable for POST, PUT, and PATCH requests.
    ///
    /// # Type Parameters
    /// * `B` - The type of the request body (must be serializable)
    ///
    /// # Arguments
    /// * `method` - The HTTP method to use
    /// * `endpoint` - The API endpoint to call
    /// * `body` - The request body to serialize as JSON
    async fn make_request_with_body<B>(
        &self,
        method: HttpMethod,
        endpoint: &str,
        body: &B,
    ) -> ApiResult<Response>
    where
        B: serde::Serialize;

    /// Performs a GET request and deserializes the JSON response.
    ///
    /// # Type Parameters
    /// * `T` - The expected response type
    ///
    /// # Arguments
    /// * `endpoint` - The API endpoint to call
    async fn get<T>(&self, endpoint: &str) -> ApiResult<T>
    where
        T: serde::de::DeserializeOwned;

    /// Performs a POST request with a JSON body and deserializes the response.
    ///
    /// # Type Parameters
    /// * `T` - The expected response type
    /// * `B` - The request body type
    ///
    /// # Arguments
    /// * `endpoint` - The API endpoint to call
    /// * `body` - The request body to send
    async fn post<T, B>(&self, endpoint: &str, body: &B) -> ApiResult<T>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize;

    /// Performs a PUT request with a JSON body and deserializes the response.
    ///
    /// # Type Parameters
    /// * `T` - The expected response type
    /// * `B` - The request body type
    ///
    /// # Arguments
    /// * `endpoint` - The API endpoint to call
    /// * `body` - The request body to send
    async fn put<T, B>(&self, endpoint: &str, body: &B) -> ApiResult<T>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize;

    /// Performs a PATCH request with a JSON body and deserializes the response.
    ///
    /// # Type Parameters
    /// * `T` - The expected response type
    /// * `B` - The request body type
    ///
    /// # Arguments
    /// * `endpoint` - The API endpoint to call
    /// * `body` - The request body to send
    async fn patch<T, B>(&self, endpoint: &str, body: &B) -> ApiResult<T>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize;

    /// Performs a DELETE request and deserializes the JSON response.
    ///
    /// # Type Parameters
    /// * `T` - The expected response type
    ///
    /// # Arguments
    /// * `endpoint` - The API endpoint to call
    async fn delete<T>(&self, endpoint: &str) -> ApiResult<T>
    where
        T: serde::de::DeserializeOwned;
}

/// A concrete implementation of the ApiClient trait using gloo-net for HTTP requests.
///
/// This client is designed for WebAssembly applications and provides a simple
/// interface for making REST API calls with customizable headers.
///
/// # Example
/// ```compile_fail
/// let mut client = ApiClientHttp::new("https://api.example.com");
/// client.set_header("Content-Type".to_string(), "application/json".to_string());
/// client.set_header("Authorization".to_string(), "Bearer token123".to_string());
///
/// let response: MyResponse = client.get("/api/data").await?;
/// ```
pub struct ApiClientHttp {
    /// The base URL for all API requests
    root_url: String,
    /// Headers to include with every request
    headers: ApiHeaders,
}

impl ApiClientHttp {
    /// Creates a new HTTP API client with the specified base URL.
    ///
    /// # Arguments
    /// * `root_url` - The base URL for all API requests (e.g., "https://api.example.com")
    ///
    /// # Example
    /// ```compile_fail
    /// let client = ApiClientHttp::new("https://api.example.com");
    /// ```
    pub fn new(root_url: impl Into<String>) -> Self {
        Self {
            root_url: root_url.into(),
            headers: ApiHeaders::new(),
        }
    }

    /// Sets a single header that will be included with all requests.
    ///
    /// # Arguments
    /// * `key` - The header name
    /// * `value` - The header value
    ///
    /// # Example
    /// ```compile_fail
    /// client.set_header("Authorization".to_string(), "Bearer token123".to_string());
    /// ```
    pub fn set_header(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }

    /// Sets multiple headers at once.
    ///
    /// # Arguments
    /// * `headers` - A vector of (key, value) tuples representing headers
    ///
    /// # Example
    /// ```compile_fail
    /// client.set_headers(vec![
    ///     ("Content-Type".to_string(), "application/json".to_string()),
    ///     ("Authorization".to_string(), "Bearer token123".to_string()),
    /// ]);
    /// ```
    pub fn set_headers(&mut self, headers: Vec<(String, String)>) {
        for (key, value) in headers {
            self.headers.insert(key, value);
        }
    }
}

#[async_trait::async_trait(?Send)]
impl ApiClient for ApiClientHttp {
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
