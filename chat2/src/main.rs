use gpui::*;
use gpui_component::{
    input::{Input, InputEvent, InputState},
    Root,
};
use gpui_component_assets::Assets;

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

    fn send(&mut self, text: String, cx: &mut Context<Self>) {
        self.messages.push(Message { role: Role::User, text: text.clone() });
        // TODO: ここで RAG / LLM を呼び出す。今はダミー応答。
        self.messages.push(Message {
            role: Role::Ai,
            text: format!("かしこまりました。\n「{text}」についての内容を要約します。"),
        });
        self.scroll.scroll_to_bottom();
        cx.notify();
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
