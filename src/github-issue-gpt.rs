use github_flows::{get_octo, listen_to_event, EventPayload};
use http_req::{
    request::{Method, Request},
    uri::Uri,
};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::env;
#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() -> anyhow::Result<()> {
    listen_to_event("jaykchen", "vitesse-lite", vec!["fork"], handler).await;

    Ok(())
}

pub async fn get_answer(query: String) -> String {
    let api_token: String = env::var("OPENAI_API_TOKEN").expect("token not found");

    let prompt = r#"Please reply to me with the answer "You have abused Q&A api""#;

    let params = serde_json::json!({
                "model": "gpt-3.5-turbo",
        "messages": [{"role": "user", "content": prompt}],
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
    // let stream = TcpStream::connect((addr.host().unwrap(), addr.corr_port())).unwrap();
    // let mut stream = tls::Config::default()
    //     .connect(addr.host().unwrap_or(""), stream)
    //     .unwrap();

    let bearer_token = format!("Bearer {}", api_token);
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

    let octocrab = get_octo(None);

    if let EventPayload::IssueCommentEvent(e) = payload {
        let comment_obj = e.comment;
        let comment_id = comment_obj.id;
        // let query_str = format!("/repos/{owner}/{repo}/issues/comments/{comment_id}");

        let comment = comment_obj.body.expect("possibly empty comment");
        // let comment: String = octocrab
        //     .issues(owner, repo)
        //     .get_comment(comment_id)
        //     .await
        //     .unwrap()
        //     .body_text
        //     .unwrap_or("no comment obtained".to_string());

        let gpt_answer = get_answer(comment).await;

        let id = comment_id.to_string().parse::<u64>().unwrap_or(0);
        octocrab.issues(owner, repo).create_comment(id, gpt_answer);
    }
}
