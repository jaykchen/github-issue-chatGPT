use github_flows::{get_octo, listen_to_event, EventPayload};
use http_req::{
    request::{Method, Request},
    uri::Uri,
};
use openai_flows::chat_completion;
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

    if let EventPayload::IssueCommentEvent(e) = payload {
        // let octocrab = get_octo(None);

        let comment_obj = e.comment;
        let comment_id = comment_obj.id;
        let issue_number = e.issue.number;
        let comment = comment_obj.body.expect("possibly empty comment");

        send_message_to_channel("ik8", "general", comment.clone());

        if let Some(gpt_answer) =
            chat_completion("jaykchen", &format!("issue#{}", issue_number), &comment)
        {
            send_message_to_channel("ik8", "general", gpt_answer.choice);
        }

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
