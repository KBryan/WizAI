use leptos::*;
use wasm_bindgen_futures::spawn_local;
use std::collections::HashMap;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct AddressSuggestion {
    pub address: String,
    pub city: String,
    pub province: String,
    pub postal_code: Option<String>,
}

#[component]
pub fn AddressAutocomplete(
    value: ReadSignal<String>,
    on_select: Callback<AddressSuggestion>,
) -> impl IntoView {
    let (input_value, set_input_value) = create_signal(value());
    let (suggestions, set_suggestions) = create_signal(vec![]);
    let (is_loading, set_is_loading) = create_signal(false);
    let (show_dropdown, set_show_dropdown) = create_signal(false);
    let (selected_index, set_selected_index) = create_signal(-1);
    
    let debounce_timer: std::cell::RefCell<Option<i32>> = std::cell::RefCell::new(None);
    
    let fetch_suggestions = move |query: String| {
        if query.len() < 3 {
            set_suggestions.set(vec![]);
            return;
        }
        
        set_is_loading.set(true);
        
        spawn_local(async move {
            let mock_suggestions = vec![
                AddressSuggestion {
                    address: format!("{} Main St", query),
                    city: "Pickering".to_string(),
                    province: "ON".to_string(),
                    postal_code: Some("L1V 1A1".to_string()),
                },
                AddressSuggestion {
                    address: format!("{} King St", query),
                    city: "Ajax".to_string(),
                    province: "ON".to_string(),
                    postal_code: Some("L1S 1A2".to_string()),
                },
                AddressSuggestion {
                    address: format!("{} Queen St", query),
                    city: "Whitby".to_string(),
                    province: "ON".to_string(),
                    postal_code: Some("L1N 1A3".to_string()),
                },
                AddressSuggestion {
                    address: format!("{} Dundas St", query),
                    city: "Oshawa".to_string(),
                    province: "ON".to_string(),
                    postal_code: Some("L1H 1A4".to_string()),
                },
            ];
            
            set_suggestions.set(mock_suggestions);
            set_show_dropdown.set(true);
            set_is_loading.set(false);
        });
    };
    
    let on_input = move |e: web_sys::InputEvent| {
        let value = event_target_value(&e);
        set_input_value.set(value.clone());
        
        if let Some(window) = web_sys::window() {
            if let Some(timer) = debounce_timer.borrow_mut().take() {
                window.clear_interval_with_handle(timer);
            }
            
            let handle = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                Box::new(move || {
                    fetch_suggestions(value.clone());
                }),
                300,
            ).ok();
            
            *debounce_timer.borrow_mut() = handle;
        }
    };
    
    let on_select_suggestion = move |suggestion: AddressSuggestion| {
        set_input_value.set(format!("{}, {}", suggestion.address, suggestion.city));
        set_show_dropdown.set(false);
        set_suggestions.set(vec![]);
        on_select(suggestion);
    };
    
    let on_key_down = move |e: web_sys::KeyboardEvent| {
        let suggestions_len = suggestions().len() as i32;
        
        match e.key().as_str() {
            "ArrowDown" => {
                e.prevent_default();
                set_selected_index.update(|i| (*i + 1).min(suggestions_len - 1));
            }
            "ArrowUp" => {
                e.prevent_default();
                set_selected_index.update(|i| (*i - 1).max(0));
            }
            "Enter" => {
                if selected_index() >= 0 && selected_index() < suggestions_len {
                    on_select_suggestion(suggestions()[selected_index() as usize].clone());
                }
            }
            "Escape" => {
                set_show_dropdown.set(false);
            }
            _ => {}
        }
    };
    
    view! {
        <div class="relative">
            <input
                type="text"
                value=input_value()
                on:input=on_input
                on:keydown=move |e| on_key_down(event_target(&e))
                on:focus=move |_| {
                    if !suggestions().is_empty() {
                        set_show_dropdown.set(true);
                    }
                }
                on:blur=move |_| {
                    set_timeout(move || set_show_dropdown.set(false), 200);
                }
                placeholder="Start typing an address..."
                class="input"
            />
            
            <Show when=is_loading()>
                <div class="absolute right-3 top-1/2 -translate-y-1/2">
                    <div class="w-4 h-4 border-2 border-primary-600 border-t-transparent rounded-full animate-spin"></div>
                </div>
            </Show>
            
            <Show when=show_dropdown() && !suggestions().is_empty()>
                <div class="absolute z-50 w-full mt-1 bg-white rounded-lg shadow-lg border border-neutral-200 max-h-60 overflow-auto">
                    {suggestions().into_iter().enumerate().map(|(idx, suggestion)| {
                        let is_selected = selected_index() == idx as i32;
                        let full_address = format!("{}, {}, {}", suggestion.address, suggestion.city, suggestion.province);
                        view! {
                            <button
                                on:click=move |_| on_select_suggestion(suggestion.clone())
                                class={format!(
                                    "w-full text-left px-4 py-3 hover:bg-primary-50 transition-colors {}",
                                    if is_selected { "bg-primary-50" } else { "" }
                                )}
                            >
                                <div class="font-medium text-neutral-900">{suggestion.address.clone()}</div>
                                <div class="text-sm text-neutral-500">
                                    {format!("{}, {} {}", suggestion.city, suggestion.province, suggestion.postal_code.unwrap_or_default())}
                                </div>
                            </button>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </Show>
        </div>
    }
}
