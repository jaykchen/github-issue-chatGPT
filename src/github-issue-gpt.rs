use dotenv::dotenv;
use github_flows::{get_octo, listen_to_event, EventPayload};
use http_req::{
    request::{Method, Request},
    uri::Uri,
};
use serde::{Deserialize, Serialize};
use slack_flows::send_message_to_channel;
use std::borrow::Cow;
use std::convert::TryFrom;
use std::env;
use std::fmt;
use std::io;
use std::str;

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
                    let encoded = encode(&b);
                    if let Some(r) = chat_completion(&encoded) {
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
        "max_tokens": 512,
        "temperature": 0.7,
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
            let text = String::from_utf8(writer).unwrap();
            send_message_to_channel("ik8", "general", text.to_string());
            return Some(text);
            // let raw: ChatResponse = serde_json::from_slice(&writer).unwrap();
            // let answer = raw.choices[0].text.clone();
            // return Some(answer);
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

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Encoded<Str>(pub Str);

impl<Str: AsRef<[u8]>> Encoded<Str> {
    /// Long way of writing `Encoded(data)`
    ///
    /// Takes any string-like type or a slice of bytes, either owned or borrowed.
    #[inline(always)]
    pub fn new(string: Str) -> Self {
        Self(string)
    }

    #[inline(always)]
    pub fn to_str(&self) -> Cow<str> {
        encode_binary(self.0.as_ref())
    }

    /// Perform urlencoding to a string
    #[inline]
    #[allow(clippy::inherent_to_string_shadow_display)]
    pub fn to_string(&self) -> String {
        self.to_str().into_owned()
    }

    /// Perform urlencoding into a writer
    #[inline]
    pub fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        encode_into(self.0.as_ref(), false, |s| writer.write_all(s.as_bytes()))?;
        Ok(())
    }

    /// Perform urlencoding into a string
    #[inline]
    pub fn append_to(&self, string: &mut String) {
        append_string(&self.0.as_ref(), string, false);
    }
}

impl<'a> Encoded<&'a str> {
    /// Same as new, but hints a more specific type, so you can avoid errors about `AsRef<[u8]>` not implemented
    /// on references-to-references.
    #[inline(always)]
    pub fn str(string: &'a str) -> Self {
        Self(string)
    }
}

impl<String: AsRef<[u8]>> fmt::Display for Encoded<String> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        encode_into(self.0.as_ref(), false, |s| f.write_str(s))?;
        Ok(())
    }
}

/// Percent-encodes every byte except alphanumerics and `-`, `_`, `.`, `~`. Assumes UTF-8 encoding.
///
/// Call `.into_owned()` if you need a `String`
#[inline(always)]
pub fn encode(data: &str) -> Cow<str> {
    encode_binary(data.as_bytes())
}

/// Percent-encodes every byte except alphanumerics and `-`, `_`, `.`, `~`.
#[inline]
pub fn encode_binary(data: &[u8]) -> Cow<str> {
    // add maybe extra capacity, but try not to exceed allocator's bucket size
    let mut escaped = String::with_capacity(data.len() | 15);
    let unmodified = append_string(data, &mut escaped, true);
    if unmodified {
        return Cow::Borrowed(unsafe {
            // encode_into has checked it's ASCII
            str::from_utf8_unchecked(data)
        });
    }
    Cow::Owned(escaped)
}

fn append_string(data: &[u8], escaped: &mut String, may_skip: bool) -> bool {
    encode_into(data, may_skip, |s| {
        Ok::<_, std::convert::Infallible>(escaped.push_str(s))
    })
    .unwrap()
}

fn encode_into<E>(
    mut data: &[u8],
    may_skip_write: bool,
    mut push_str: impl FnMut(&str) -> Result<(), E>,
) -> Result<bool, E> {
    let mut pushed = false;
    loop {
        // Fast path to skip over safe chars at the beginning of the remaining string
        let ascii_len = data.iter()
            .take_while(|&&c| matches!(c, b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' |  b'-' | b'.' | b'_' | b'~')).count();

        let (safe, rest) = if ascii_len >= data.len() {
            if !pushed && may_skip_write {
                return Ok(true);
            }
            (data, &[][..]) // redundatnt to optimize out a panic in split_at
        } else {
            data.split_at(ascii_len)
        };
        pushed = true;
        if !safe.is_empty() {
            push_str(unsafe { str::from_utf8_unchecked(safe) })?;
        }
        if rest.is_empty() {
            break;
        }

        match rest.split_first() {
            Some((byte, rest)) => {
                let enc = &[b'%', to_hex_digit(byte >> 4), to_hex_digit(byte & 15)];
                push_str(unsafe { str::from_utf8_unchecked(enc) })?;
                data = rest;
            }
            None => break,
        };
    }
    Ok(false)
}

#[inline]
fn to_hex_digit(digit: u8) -> u8 {
    match digit {
        0..=9 => b'0' + digit,
        10..=255 => b'A' - 10 + digit,
    }
}
