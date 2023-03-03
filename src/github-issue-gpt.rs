use github_flows::{get_octo, listen_to_event, EventPayload};
use http_req::{
    request::{Method, Request},
    uri::Uri,
};
use serde::{Deserialize, Serialize};
use slack_flows::send_message_to_channel;
use std::convert::TryFrom;
use std::env;

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() -> anyhow::Result<()> {
    listen_to_event(
        "jaykchen",
        "vitesse-lite",
        vec!["issue_comment", "issues", "created"],
        handler,
    )
    .await;

    Ok(())
}

pub async fn get_answer(query: String) -> String {
    let api_token: String = env::var("OPENAI_API_TOKEN").expect("token not found");

    // let prompt = r#"Please reply to me with the answer "You have abused Q&A api""#;

    let params = serde_json::json!({
                "model": "gpt-3.5-turbo",
        "messages": [{"role": "user", "content": query}],
        "temperature": 0.7,
        "top_p": 1,
        "n": 1,
        "stream": false,
        "max_tokens": 512,
        "presence_penalty": 0,
        "frequency_penalty": 0,
    });

    let s: &str = Deserialize::deserialize(params).unwrap();
    let body = s.as_bytes();

    let addr = Uri::try_from("https://api.openai.com/v1/chat/completions").unwrap();
    let mut writer = Vec::new();

    let bearer_token = format!("Bearer {}", api_token);

    send_message_to_channel("ik8", "general", api_token.to_string());

    let _ = Request::new(&addr)
        .method(Method::POST)
        .header("Content-Type", "application/json")
        .header("Authorization", &bearer_token)
        .header("Content-Length", &body.len())
        .body(&body)
        .send(&mut writer)
        .unwrap();

    let text = String::from_utf8(writer).unwrap_or("failed to parse response".to_string());
    let raw: ChatResponse = serde_json::from_str(&text).unwrap();
    raw.choices[0].message.content.to_string()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatResponse {
    created: i64,
    choices: Vec<Choice>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Choice {
    message: Message,
    index: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    role: String,
    content: String,
}

async fn handler(payload: EventPayload) {
    let owner = "jaykchen";
    let repo = "vitesse-lite";

    if let EventPayload::IssueCommentEvent(e) = payload {
        // let octocrab = get_octo(None);

        let comment_obj = e.comment;
        let comment_id = comment_obj.id;

        let comment = comment_obj.body.expect("possibly empty comment");

        send_message_to_channel("ik8", "general", comment.clone());

        let gpt_answer = get_answer(comment).await;

        send_message_to_channel("ik8", "general", gpt_answer.clone());

        // let id = comment_id.to_string().parse::<u64>().unwrap_or(0);
        // octocrab
        //     .issues(owner, repo)
        //     .create_comment(id, gpt_answer)
        //     .await;

        // let mut writer = Vec::new();
        // let query_str =
        //     format!("repos/{owner}/{repo}/issues/{issue_number}/comments/{comment_id}/replies");
        // let addr = Uri::try_from(query_str).unwrap();

        // let body = serde_json::json!({ "body": gpt_answer });

        // let _ = Request::new(&addr)
        //     .method(Method::POST)
        //     .header("Content-Type", "application/vnd.github.v3+json")
        //     .header("Authorization", &bearer_token)
        //     .header("Content-Length", &body.len())
        //     .body(&body)
        //     .send(&mut writer)
        //     .unwrap();
    }
}

// async fn reply_comment(reply: String) {
//     let mut writer = Vec::new();
//     let query_str =
//         format!("repos/{owner}/{repo}/issues/{issue_number}/comments/{comment_id}/replies");
//     let addr = Uri::try_from(query_str).unwrap();

//     let body = serde_json!({ "body": reply });

//     let _ = Request::new(&addr)
//         .method(Method::POST)
//         .header("Content-Type", "application/vnd.github.v3+json")
//         .header("Authorization", &bearer_token)
//         .header("Content-Length", &body.len())
//         .body(&body)
//         .send(&mut writer)
//         .unwrap();
// }
