use serde::Deserialize;
use std::convert::Infallible;
use std::result::Result;
use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};

#[derive(Deserialize, Debug)]
pub struct Request {
    pub action: String,
    pub contracts: String,
}

#[tokio::main]
async fn main() {
    println!("Tradeproxy starting up!");

    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("trade")
        .and(warp::post())
        // if it's JSON...
        .and(warp::body::json())
        .map(|req: Request| {
            println!("Got a request! {:?}", req);
            warp::reply()
        })
        .recover(handle_error);

    warp::serve(hello).run(([0, 0, 0, 0], 3137)).await;
}

async fn handle_error(err: Rejection) -> Result<impl Reply, Infallible> {
    let err_text = format!(
        "Whoa! There was a problem with parsing the JSON, or something: {:?}",
        err
    );
    println!("{}", err_text);
    Ok(warp::reply::with_status(
        err_text,
        StatusCode::BAD_REQUEST,
    ))
}
