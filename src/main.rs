use warp::Filter;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use woothee::parser::Parser;
mod pages;

#[tokio::main]
async fn main() {
    let index = warp::path::end().map(|| {
        warp::reply::html(pages::INDEX.clone())
    });
    let add_user_agent = warp::path("add-user-agent")
        .and(warp::body::content_length_limit(4096))
        .and(warp::body::json())
        .and_then(|user_agent: UserAgent| async move {
            if Parser::new().parse(&user_agent.user_agent).is_none() {
                return Ok(warp::reply::with_status("Yes".to_owned(), warp::http::StatusCode::OK)) as Result<_, warp::Rejection>;
            }
            println!("{:?}", user_agent.user_agent);
            let app_id = match std::env::var("BACK4APP_APP_ID") {
                Ok(app_id) => app_id,
                Err(err) => return Ok(warp::reply::with_status(format!("500 Internal Server Error: {}", err), warp::http::StatusCode::INTERNAL_SERVER_ERROR)),
            };
            let api_key = match std::env::var("BACK4APP_API_KEY") {
                Ok(api_key) => api_key,
                Err(err) => return Ok(warp::reply::with_status(format!("500 Internal Server Error: {}", err), warp::http::StatusCode::INTERNAL_SERVER_ERROR)),
            };
            let user_agent_json = json!({
                "user_agent": user_agent.user_agent
            });
            let client = reqwest::Client::new();
            let response_sent = client
                .get("https://parseapi.back4app.com/classes/user_agents")
                .header("X-Parse-Application-Id", app_id.clone())
                .header("X-Parse-REST-API-Key", api_key.clone())
                .query(&[("where", user_agent_json.to_string())])
                .send()
                .await;
            let mut response_json = match response_sent {
                Ok(response) => response.json::<HashMap<String, Vec<HashMap<String, String>>>>().await,
                Err(err) => return Ok(warp::reply::with_status(format!("502 Internal Server Error: {}", err), warp::http::StatusCode::INTERNAL_SERVER_ERROR)),
            };
            let response = match &mut response_json {
                Ok(json) => json.get("results").unwrap(),
                Err(err) => return Ok(warp::reply::with_status(format!("502 Internal Server Error: {}", err), warp::http::StatusCode::INTERNAL_SERVER_ERROR)),
            };
            if response.len() == 0 {
                println!("unique");
                let user_agent_json = json!({
                    "user_agent": user_agent.user_agent
                });
                let response_sent = client
                    .post("https://parseapi.back4app.com/classes/user_agents")
                    .header("X-Parse-Application-Id", app_id)
                    .header("X-Parse-REST-API-Key", api_key)
                    .header("Content-Type", "application/json")
                    .body(user_agent_json.to_string())
                    .send()
                    .await;
                let response_json = match response_sent {
                    Ok(response) => response.json::<HashMap<String, String>>().await,
                    Err(err) => return Ok(warp::reply::with_status(format!("502 Internal Server Error: {}", err), warp::http::StatusCode::INTERNAL_SERVER_ERROR)),
                };
                let response = match response_json {
                    Ok(json) => json,
                    Err(err) => return Ok(warp::reply::with_status(format!("502 Internal Server Error: {}", err), warp::http::StatusCode::INTERNAL_SERVER_ERROR)),
                };
                println!("{:?}", response);
                Ok(warp::reply::with_status("Yes".to_owned(), warp::http::StatusCode::OK))
            } else {
                println!("not unique");
                Ok(warp::reply::with_status("No".to_owned(), warp::http::StatusCode::OK))
            }
        });
    let port = std::env::var("PORT")
        .expect("PORT Environment Variable not set")
        .parse()
        .expect("PORT is not a valid port number");
    let routes = index.or(add_user_agent);
    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
}

#[derive(Deserialize, Serialize)]
struct UserAgent {
    user_agent: String,
}
