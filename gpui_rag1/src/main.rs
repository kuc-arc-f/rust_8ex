use dotenvy::dotenv;
use gpui::*;
use gpui_component::{
    input::{Input, InputEvent, InputState},
    Root,
};
use gpui_component_assets::Assets;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

mod model;
mod mod_chat;

/// 発言者
#[derive(Clone, Copy, PartialEq)]
enum Role {
    User,
    Ai,
}

struct Message {
    role: Role,
    text: String,
}

struct ChatView {
    messages: Vec<Message>,
    input: Entity<InputState>,
    scroll: ScrollHandle,
    _sub: Subscription,
}

async fn get_vec_search(query: String) -> Result<String, Box<dyn std::error::Error>> 
{
    let mut ret :String = "".to_string();
    dotenv().ok();
    let prefix_key = env::var("PREFIX_KEY")
        .expect("PREFIX_KEY が設定されていません");

    let url = "http://localhost:8080/v1/embeddings";

    let body = model::EmbeddingRequest {
        input: query.to_string(),
    };

    let client = reqwest::Client::new();

    let res = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&body) // 自動で Content-Type: application/json が付与されます
        .send()
        .await?;

    // ステータスコード確認
    let status = res.status();
    println!("Status: {}", status);

    // レスポンスを JSON として取得
    let text = res.text().await?;
    //println!("Response body: {}", text);

    let resp: model::EmbeddingResponse = serde_json::from_str(&text)?;

    // data 内の embedding を全て取得
    let mut vec2 :Vec<f64> = Vec::new();
    for item in &resp.data {
        println!("index = {}", item.index);
        //println!("embedding (len={}): {:?}", item.embedding.len(), item.embedding);
        vec2 = item.embedding.clone();
        println!("embedding (len={})", item.embedding.len());
    }
    let req1 = model::VectorSearchReq {
        prefix: String::from(&prefix_key),
        limit: 3,
        vector: vec2,
    };
    //print!("req!{:?}", req1);
    let url_2 = "http://localhost:8888/api/select";    
    let res2 = client
        .post(url_2)
        .header("Content-Type", "application/json")
        .json(&req1) // 自動で Content-Type: application/json が付与されます
        .send()
        .await?;  
    let stat2 = res2.status();
    println!("stat2: {}", stat2);
    let text2 = res2.text().await?;  
    //println!("text2={}", text2);
    // JSON をパース
    let res3 : model::VecContetRes = serde_json::from_str(&text2)?;
    let rows: Vec<model::Row> = res3.rows;
    println!("取得件数: {}", rows.len());
    println!("----------------------------------");
    // 各要素を出力
    let mut matches : String = "".to_string();
    for (i, row) in rows.iter().enumerate() {
        //println!("[{}]", i);
        println!("  id       : {}", row.id);
        //println!("  content  : {}", row.content);
        println!("  distance : {}", row.distance);
        let content_str = format!("{}\n\n", &row.content);
        matches.push_str(&content_str.clone().to_string());
    }  
    let mut out_str : String = "".to_string();
    if matches.len() > 0 {
        out_str = format!("context: {}\n", matches);
        let out_add2 = format!("user query: {}\n" , query);
        out_str.push_str(&out_add2);
    }else {
        out_str = format!("user query: {}\n", query);
    }  
    let send_text = format!("日本語で、回答して欲しい。\n 要約して欲しい。\n\n{}", out_str);
    ret = mod_chat::chat_post(send_text.clone()).await;
    Ok(ret)
}

impl ChatView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Type your text and press Enter:")
        });

        // Enter キーで送信
        let sub = cx.subscribe_in(
            &input,
            window,
            |this: &mut Self, state, ev: &InputEvent, window, cx| {
                if let InputEvent::PressEnter { .. } = ev {
                    let text = state.read(cx).value().to_string();
                    let text = text.trim().to_string();
                    if text.is_empty() {
                        return;
                    }
                    this.send(text, cx);
                    state.update(cx, |s, cx| s.set_value("", window, cx));
                }
            },
        );

        Self {
            messages: vec![],
            input,
            scroll: ScrollHandle::new(),
            _sub: sub,
        }
    }

    fn send(&mut self, text: String, cx: &mut Context<'_, Self>) {
        print!("send.text={}", &text);
        self.messages.push(Message { role: Role::User, text: text.clone() });
        // TODO: ここで RAG / LLM を呼び出す。今はダミー応答。
        self.messages.push(Message {
            role: Role::Ai,
            text: "考え中...".to_string(),
        });        
        self.scroll.scroll_to_bottom();
        cx.notify();
        // 重たい処理はバックグラウンドスレッドで実行する。
        // (メイン/UI スレッドで実行すると画面がフリーズする)
        let task = cx.background_executor().spawn(async move {
            // TODO: ここで RAG / LLM を呼び出す。今はダミーとして重い処理を模擬。
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let mut res2 :  String = "".to_string(); 
            runtime.block_on(async {
                let resp = get_vec_search(text.clone()).await;
                match resp {
                    Ok(value) => {
                        println!("結果: {}", value);
                        res2 = value;
                    },
                    Err(err) => println!("error: {}", err),
                }       
            });            
            return res2;         
        });
        // 処理完了後に UI スレッドへ結果を反映
        cx.spawn(async move |this, cx| {
            let reply = task.await;
            println!("reply={}" , reply);
            let _ = this.update(cx, |this, cx| {
                // プレースホルダーを実際の応答で差し替え
                if let Some(m) = this.messages.last_mut() {
                    if m.role == Role::Ai {
                        m.text = reply;
                    }
                }
                this.scroll.scroll_to_bottom();
                cx.notify();
            });
        })
        .detach();        

    }

    fn render_message(&self, m: &Message) -> Div {
        match m.role {
            // ユーザー発言: 青い帯
            Role::User => div()
                .w_full()
                .px_3()
                .py_4()
                .bg(rgb(0x2f7bff))
                .text_color(white())
                .child(m.text.clone()),
            // AI 応答: 枠線付きボックス
            Role::Ai => div()
                .w_full()
                .p_3()
                .border_1()
                .border_color(rgb(0xdddddd))
                .rounded_md()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_color(rgb(0x4ec9b0))
                        .font_weight(FontWeight::BOLD)
                        .child("AI:"),
                )
                .child(m.text.clone()),
        }
    }
}

impl Render for ChatView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col() // 縦並び: 履歴 → 入力欄
            .bg(rgb(0xf5f5f5))
            .text_color(rgb(0x000000))
            .font_family("Meiryo")
            .p_3()
            .gap_3()
            // ── 上段: チャット履歴(スクロール可、余白を全部使う) ──
            .child(
                div()
                    .id("messages")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .flex()
                    .flex_col()
                    .gap_4()
                    .children(self.messages.iter().map(|m| self.render_message(m))),
            )
            // ── 下段: 入力欄(固定) ──
            .child(
                div()
                    .w_full()
                    .flex_none()
                    .p_3()
                    .border_1()
                    .border_color(rgb(0xdddddd))
                    .rounded_md()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_color(rgb(0x4ec9b0))
                            .font_weight(FontWeight::BOLD)
                            .child("Input:"),
                    )
                    .child(Input::new(&self.input)),
            )
    }
}

fn main() {
    Application::new().with_assets(Assets).run(|cx: &mut App| {
        gpui_component::init(cx);

        cx.spawn(async move |cx| {
            cx.open_window(
                gpui::WindowOptions {
                    titlebar: Some(gpui::TitlebarOptions {
                        title: Some("Chat App".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },                
                |window, cx| {
                    let view = cx.new(|cx| ChatView::new(window, cx));
                    cx.new(|cx| Root::new(view, window, cx))
                })?;
            Ok::<_, anyhow::Error>(())
        })
        .detach();
        cx.activate(true);
    });
}
