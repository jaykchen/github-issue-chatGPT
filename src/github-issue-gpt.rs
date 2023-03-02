use http_req::{
    request::{Method, Request},
    uri::Uri,
};
use std::convert::TryFrom;
// use github_flows::{get_octo, listen_to_event, EventPayload};
use serde::{Deserialize, Serialize};
use slack_flows::send_message_to_channel;
use std::env;
#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() -> anyhow::Result<()> {
    // listen_to_event("jaykchen", "vitesse-lite", vec!["fork"], handler).await;

    let api_token: String = env::var("OPENAI-API-TOKEN").expect("token not found");

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
    let message_content = raw.choices[0].message.content.to_string();
    send_message_to_channel("ik8", "general", message_content);
    Ok(())
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

// async fn handler(payload: EventPayload) {

//     let octocrab = get_octo(None);

//     let repo = octocrab
//         .repos("jaykchen", "vitesse-lite")
//         .get()
//         .await
//         .expect("repo not obtained");

//     let full_name = repo.full_name.expect("full_name not obtained");
//     let visibility = repo.visibility.expect("visibility not obtained");

//     if let EventPayload::ForkEvent(e) = payload {
//         let forkee = e.forkee;
//         let forkee_name = forkee.owner.expect("no owner field").login;
//         let html_url = forkee.html_url.expect("no html_url field").to_string();

//         let text = format!(
//             "{} forked your {}({})!\n{}",
//             forkee_name, full_name, visibility, html_url
//         );
//     }
// }
