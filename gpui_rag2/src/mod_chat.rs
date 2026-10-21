use anyhow::{Context, Result};
use dotenvy::dotenv;
use reqwest::Client;
use reqwest::Error;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};

//use rusqlite::{ffi::sqlite3_auto_extension, Connection, params};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt;
use std::env;

// JSONデータの構造を定義
#[derive(Debug, Deserialize, Serialize)]
struct Message {
    role: String,
    content: String,
    refusal: Option<String>,
    reasoning: Option<String>,
    reasoning_details: Option<Vec<ReasoningDetail>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ReasoningDetail {
    #[serde(rename = "type")]
    type_field: String,
    text: String,
    format: Option<String>,
    index: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Choice {
    index: i32,
    logprobs: Option<serde_json::Value>,
    finish_reason: String,
    native_finish_reason: Option<String>,
    message: Message,
}

#[derive(Debug, Deserialize, Serialize)]
struct Usage {
    prompt_tokens: i32,
    completion_tokens: i32,
    total_tokens: i32,
    cost: f64,
    is_byok: bool,
    prompt_tokens_details: PromptTokensDetails,
    cost_details: CostDetails,
    completion_tokens_details: CompletionTokensDetails,
}

#[derive(Debug, Deserialize, Serialize)]
struct PromptTokensDetails {
    cached_tokens: i32,
    cache_write_tokens: i32,
    audio_tokens: i32,
    video_tokens: i32,
}

#[derive(Debug, Deserialize, Serialize)]
struct CostDetails {
    upstream_inference_cost: f64,
    upstream_inference_prompt_cost: f64,
    upstream_inference_completions_cost: f64,
}

#[derive(Debug, Deserialize, Serialize)]
struct CompletionTokensDetails {
    reasoning_tokens: i32,
    image_tokens: i32,
    audio_tokens: i32,
}

#[derive(Debug, Deserialize, Serialize)]
struct Response {
    id: String,
    object: String,
    created: i64,
    model: String,
    provider: String,
    system_fingerprint: Option<String>,
    service_tier: Option<String>,
    choices: Vec<Choice>,
    usage: Usage,
}

pub async fn chat_post(query: String) -> String {
    let mut ret = "".to_string(); 
    let api_key = env::var("OPENROUTER_API_KEY")
        .expect("OPENROUTER_API_KEY が設定されていません");
    let model_name = env::var("OPENROUTER_MODEL")
        .expect("OPENROUTER_MODEL が設定されていません");

    let client = reqwest::Client::new();

    let body = json!({
        "model": &model_name,
        "messages": [
            {
                "role": "user",
                "content": &query
            }
        ]
    });

    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header(AUTHORIZATION, format!("Bearer {}", api_key))
        .header(CONTENT_TYPE, "application/json")
        .json(&body)
        .send()
        .await.unwrap();

    // ステータスコード確認
    println!("Status: {}", response.status());

    // レスポンス本文を取得
    let response_text = response.text().await.unwrap();
    println!("Response:\n{}", response_text);
    let json_data = response_text.clone();

    let mut out_str = "".to_string();
    // JSONをパース
    match serde_json::from_str::<Response>(&json_data) {
        Ok(response) => {
            // choices[0].message.content を取得
            if let Some(first_choice) = response.choices.first() {
                out_str = first_choice.message.content.clone();
                //println!("first_choice={:?}", first_choice);
            } else {
                println!("choices is empty");
            }
        }
        Err(e) => {
            eprintln!("JSON parse error: {}", e);
        }
    }
    ret = out_str;

    return ret;
}
