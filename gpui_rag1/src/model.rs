use serde::{Serialize, Deserialize};
use serde_json::{json, Value};

// リクエストボディ
#[derive(Debug, Serialize)]
pub struct EmbeddingRequest {
    pub input: String,
}
#[derive(Debug, Deserialize)]
pub struct EmbeddingResponse {
    pub model: String,
    pub object: String,
    pub usage: Usage,
    pub data: Vec<EmbeddingData>,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Deserialize)]
pub struct EmbeddingData {
    pub embedding: Vec<f64>,
    pub index: u32,
    pub object: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct VectorSearchReq {
    pub prefix: String,
    pub limit: i32,
    pub vector: Vec<f64>,
}

/// rows 配列の各要素
#[derive(Debug, Serialize, Deserialize)]
pub struct Row {
    pub content: String,
    pub distance: f64,
    pub id: String,
}

/// ルート JSON 構造
#[derive(Debug, Serialize, Deserialize)]
pub struct VecContetRes {
    pub rows: Vec<Row>,
}
