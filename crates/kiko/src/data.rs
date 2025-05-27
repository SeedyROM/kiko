//! Data structures used between the frontend and backend of the Kiko application.

use serde::{Deserialize, Serialize};

/// A simple data structure representing a "Hello World" message, used for now to test the API and integrations.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HelloWorld {
    pub message: String,
}
