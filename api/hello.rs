use serde_json::json;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

#[allow(dead_code)]
#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {
	let body = serde_json::to_vec(&json!({ "message": "Hello world" }))?;
	Ok(Response::builder()
		.status(StatusCode::OK)
		.header("Content-Type", "application/json")
		.body(Body::from(body))?)
}
