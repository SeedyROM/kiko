use gloo_net::http::Request;
use kiko::data::HelloWorld;

pub async fn fetch_hello() -> Result<HelloWorld, gloo_net::Error> {
    let response = Request::get("http://localhost:3030/hello") // Back to full URL
        .send()
        .await?;

    let hello: HelloWorld = response.json().await?;
    Ok(hello)
}
