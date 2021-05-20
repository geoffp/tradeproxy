use serde::Deserialize;
use std::convert::Infallible;
use std::result::{Result};
use warp::{http::StatusCode, Filter, Rejection, Reply, reply::WithStatus};

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

fn ok_result(_: ()) -> WithStatus<&'static str> {
    warp::reply::with_status("Success!", StatusCode::OK)
}

#[tokio::main]
async fn main() {
    println!("Tradeproxy starting up!");

    let api = get_json()
        .map(ok_result)
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
            .body(r#"{"action": "foo", "contracts": "bar"}"#)
            .matches(&filter)
            .await
    );
}

#[tokio::test]
async fn it_rejects_bad_json() {
    let filter = get_json();

    fn mock_request() -> warp::test::RequestBuilder {
        warp::test::request()
            .path("/trade")
            .method("POST")
    }

    assert!(
        !mock_request()
            .body("blah blah blah")
            .matches(&filter)
            .await
    );

    assert!(
        !mock_request()
            .body(r#"{"wrong": "json"}"#)
            .matches(&filter)
            .await
    );
}

#[tokio::test]
async fn it_returns_correct_status() {

    fn mock_request() -> warp::test::RequestBuilder {
        warp::test::request()
            .path("/trade")
            .method("POST")
    }

    assert_eq!(
        mock_request()
            .body("blah blah blah")
            .filter(&get_json().map(ok_result).recover(handle_error))
            .await
            .unwrap()
            .into_response()
            .status(),
        StatusCode::BAD_REQUEST
    );

    assert_eq!(
        mock_request()
            .body(r#"{"action": "foo", "contracts": "bar"}"#)
            .filter(&get_json().map(ok_result).recover(handle_error))
            .await
            .unwrap()
            .into_response()
            .status(),
        StatusCode::OK
    );
}
