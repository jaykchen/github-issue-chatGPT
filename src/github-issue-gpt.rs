use dotenv::dotenv;
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

async fn handler(payload: EventPayload) {
    let owner = "jaykchen";
    let repo = "vitesse-lite";

    let octo = get_octo(Some(String::from("jaykchen")));
    let issues = octo.issues(owner, repo);

    match payload {
        EventPayload::IssueCommentEvent(e) => {
            if e.comment.user.r#type != "Bot" {
                if let Some(b) = e.comment.body {
                    if let Some(r) = chat_completion(&b) {
                        send_message_to_channel("ik8", "general", r);
                    }
                }
            }
        }
        _ => (),
    };
}

pub fn chat_completion(prompt: &str) -> Option<String> {
    dotenv().ok();
    let api_token = env::var("OPENAI_API_TOKEN").unwrap();
    let mut writer = Vec::new();

    let params = serde_json::json!({
        "model": "text-davinci-003",
        "prompt": prompt,
        "max_tokens": 7,
        "temperature": 0,
        "top_p": 1,
        "n": 1,
        "stream": false,
        "logprobs": null,
        "stop": "\n"
    });
    let uri = Uri::try_from("https://api.openai.com/v1/completions").unwrap();
    let bearer_token = format!("Bearer {}", api_token);
    let body = serde_json::to_vec(&params).unwrap_or_default();
    match Request::new(&uri)
        .method(Method::POST)
        .header("Authorization", &bearer_token)
        .header("Content-Type", "application/json")
        .header("Content-Length", &body.len())
        .body(&body)
        .send(&mut writer)
    {
        Ok(res) => {
            if !res.status_code().is_success() {
                send_message_to_channel("ik8", "general", res.status_code().to_string());
            }
            let raw: ChatResponse = serde_json::from_slice(&writer).unwrap();
            let answer = raw.choices[0].text.clone();
            return Some(answer);
            // serde_json::from_str(&writer).ok()
        }
        Err(_) => {}
    };

    None
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatResponse {
    created: i64,
    choices: Vec<Choice>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Choice {
    text: String,
    index: i64,
}

// pub struct Choice {
//     message: Message,
//     index: i64,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct Message {
//     role: String,
//     content: String,
// }
