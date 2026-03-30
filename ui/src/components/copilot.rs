use crate::api::client::ApiClient;
use crate::api::types::*;
use crate::components::common::*;
use crate::state::ws::{use_ws_state, WebSocketMessage};
use leptos::*;

#[component]
pub fn CopilotPanel(on_close: Callback<()>) -> impl IntoView {
    let ws_state = use_ws_state();
    let (messages, set_messages) = create_signal(vec![]);
    let (input, set_input) = create_signal(String::new());
    let (is_streaming, set_is_streaming) = create_signal(false);
    let (streaming_content, set_streaming_content) = create_signal(String::new());

    let suggestions = vec![
        "Show my hot leads",
        "Generate CMA for address",
        "Market trends for Pickering",
        "Follow up overdue leads",
    ];

    let send_message = move |_| {
        let msg = input().trim();
        if msg.is_empty() || is_streaming() {
            return;
        }

        let user_msg = ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            role: MessageRole::User,
            content: msg.to_string(),
            timestamp: chrono::Utc::now(),
        };

        set_messages.update(|m| m.push(user_msg.clone()));
        set_input.set(String::new());
        set_is_streaming.set(true);
        set_streaming_content.set(String::new());

        ws_state.send(WebSocketMessage::ChatMessage {
            content: msg.to_string(),
        });
    };

    on_mount(move || {
        let ws = ws_state.clone();
        create_effect(move |_| {
            if let Some(msg) = ws.last_message() {
                match msg {
                    WebSocketMessage::ChatResponse { content, done } => {
                        set_streaming_content.update(|s| s.push_str(content));
                        if *done {
                            let ai_msg = ChatMessage {
                                id: uuid::Uuid::new_v4().to_string(),
                                role: MessageRole::Assistant,
                                content: streaming_content(),
                                timestamp: chrono::Utc::now(),
                            };
                            set_messages.update(|m| m.push(ai_msg));
                            set_is_streaming.set(false);
                            set_streaming_content.set(String::new());
                        }
                    }
                    _ => {}
                }
            }
        });
    });

    view! {
        <div class="fixed bottom-0 left-64 right-0 h-96 bg-white border-t border-neutral-300 shadow-lg flex flex-col">
            <div class="flex items-center justify-between px-4 py-3 border-b border-neutral-200">
                <h3 class="font-semibold text-neutral-900">AI Copilot</h3>
                <button
                    on:click=on_close
                    class="p-1 hover:bg-neutral-100 rounded"
                >
                    <svg class="w-5 h-5 text-neutral-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                    </svg>
                </button>
            </div>

            <div class="flex-1 overflow-y-auto p-4 space-y-4">
                {messages().into_iter().map(|msg| {
                    view! {
                        <div class={format!("flex {}", if msg.role == MessageRole::User { "justify-end" } else { "justify-start" })}>
                            <div class={format!(
                                "max-w-[70%] rounded-lg px-4 py-2 {}",
                                if msg.role == MessageRole::User {
                                    "bg-primary-600 text-white"
                                } else {
                                    "bg-neutral-100 text-neutral-900"
                                }
                            )}>
                                <p class="text-sm">{msg.content}</p>
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}

                <Show when=is_streaming()>
                    <div class="flex justify-start">
                        <div class="bg-neutral-100 rounded-lg px-4 py-2 max-w-[70%]">
                            <div class="flex items-center gap-1">
                                <span class="w-2 h-2 bg-neutral-400 rounded-full animate-bounce"></span>
                                <span class="w-2 h-2 bg-neutral-400 rounded-full animate-bounce" style="animation-delay: 0.1s"></span>
                                <span class="w-2 h-2 bg-neutral-400 rounded-full animate-bounce" style="animation-delay: 0.2s"></span>
                            </div>
                        </div>
                    </div>
                </Show>
            </div>

            <div class="px-4 py-3 border-t border-neutral-200">
                <div class="flex flex-wrap gap-2 mb-3">
                    {suggestions.into_iter().map(|s| {
                        let s = s.to_string();
                        view! {
                            <button
                                on:click=move |_| set_input.set(s.clone())
                                class="px-3 py-1 text-xs bg-neutral-100 hover:bg-neutral-200 rounded-full text-neutral-700"
                            >
                                {s}
                            </button>
                        }
                    }).collect::<Vec<_>>()}
                </div>

                <div class="flex gap-2">
                    <input
                        type="text"
                        value=input()
                        on:input=move |e| set_input.set(event_target_value(&e))
                        on:key_down=move |e| {
                            if e.key() == "Enter" { send_message(()); }
                        }
                        placeholder="Ask the AI anything..."
                        class="flex-1 px-4 py-2 border border-neutral-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
                    />
                    <button
                        on:click=send_message
                        disabled=is_streaming()
                        class="btn-primary"
                    >
                        Send
                    </button>
                </div>
            </div>
        </div>
    }
}
