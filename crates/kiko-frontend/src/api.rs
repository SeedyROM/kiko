use gloo_net::http::Request;
use kiko::data::HelloWorld;
use kiko::log;

pub async fn fetch_hello() -> Result<HelloWorld, gloo_net::Error> {
    log::info!("Fetching hello data from the backend");
    let response = Request::get("http://localhost:3030/hello") // Back to full URL
        .send()
        .await?;

    let hello: HelloWorld = response.json().await?;
    log::info!("Received hello data: {:?}", hello);

    Ok(hello)
}
