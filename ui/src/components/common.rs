use leptos::*;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Toast {
    pub id: String,
    pub message: String,
    pub toast_type: ToastType,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

#[component]
pub fn ToastContainer() -> impl IntoView {
    let (toasts, set_toasts) = create_signal(vec![]);
    provide_context::<Signal<Vec<Toast>>>(toasts);

    let dismiss = move |id: String| {
        set_toasts.update(|t| t.retain(|toast| toast.id != id));
    };

    view! {
        <div class="fixed top-4 right-4 z-50 space-y-2">
            {toasts().into_iter().map(|toast| {
                let id = toast.id.clone();
                view! {
                    <div
                        class={format!(
                            "flex items-center gap-3 px-4 py-3 rounded-lg shadow-lg {}",
                            match toast.toast_type {
                                ToastType::Success => "bg-success-500 text-white",
                                ToastType::Error => "bg-danger-500 text-white",
                                ToastType::Warning => "bg-warning-500 text-white",
                                ToastType::Info => "bg-primary-500 text-white",
                            }
                        )}
                    >
                        <span>{toast.message}</span>
                        <button
                            on:click=move |_| dismiss(id.clone())
                            class="ml-2 hover:opacity-75"
                        >
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                            </svg>
                        </button>
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}

pub fn show_toast(message: &str, toast_type: ToastType) {
    if let Some(toasts) = get_context::<Signal<Vec<Toast>>> > (leptos::Scope::current()) {
        let toast = Toast {
            id: uuid::Uuid::new_v4().to_string(),
            message: message.to_string(),
            toast_type,
        };
        toasts.update(|t| t.push(toast));
    }
}
