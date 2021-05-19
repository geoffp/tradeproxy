use serde::Deserialize;
use std::convert::Infallible;
use std::result::Result;
use warp::{http::StatusCode, Filter, Rejection, Reply};

#[derive(Deserialize, Debug)]
pub struct Request {
    pub action: String,
    pub contracts: String,
}

fn get_json() -> impl Filter<Extract = ((),), Error = warp::Rejection> + Copy {
    warp::path!("trade")
        .and(warp::post())
        .and(warp::body::json())
        .map(|req: Request| {
            println!("Successful request: {:?}", req);
        })
}

/*Ok(warp::reply::with_status(
    "Success!",
    StatusCode::OK,
)) */

#[tokio::main]
async fn main() {
    println!("Tradeproxy starting up!");

    let api = get_json()
        .map(|()| Ok(warp::reply::with_status("Success!", StatusCode::OK)))
        .recover(handle_error);

    // .map(|reply: warp::reply::WithStatus<, bytes: Bytes| -> _ {
    //     let bad_bytes = format!("Bad request content: {:?}", bytes);
    //     println!("{}", bad_bytes);
    // });

    // .or(warp::body::bytes())
    // .map(|reply: Response<Reply>, bytes: Bytes| -> _ {
    //     let bad_bytes = format!("Bad request content: {:?}", bytes);
    //     println!("{}", bad_bytes);
    // });

    warp::serve(api).run(([0, 0, 0, 0], 3137)).await;
}

async fn handle_error(err: Rejection) -> Result<impl Reply, Infallible> {
    let err_text = format!("Whoa, bad JSON: {:?}", err);

    eprintln!("{}", err_text);

    Ok(warp::reply::with_status(err_text, StatusCode::BAD_REQUEST))
}


#[tokio::test]
async fn it_accepts_good_json() {
    let filter = get_json();
    assert!(
        warp::test::request()
            .path("/trade")
            .method("POST")
            .body("{\"action\": \"foo\", \"contracts\": \"bar\"}")
            .matches(&filter)
            .await
    );
}

#[tokio::test]
async fn it_rejects_bad_json() {
    let filter = get_json();
    assert!(
        !warp::test::request()
            .path("/trade")
            .method("POST")
            .body("blah blah blah")
            .matches(&filter)
            .await
    );
}
