use crate::state::ws::use_ws_state;
use leptos::*;

#[component]
pub fn Header() -> impl IntoView {
    let ws_state = use_ws_state();

    let (notifications_open, set_notifications_open) = create_signal(false);

    view! {
        <header class="h-16 bg-white border-b border-neutral-200 flex items-center justify-between px-6">
            <div class="flex items-center gap-4">
                <h2 class="text-lg font-semibold text-neutral-900">
                    Dashboard
                </h2>
            </div>

            <div class="flex items-center gap-4">
                <ConnectionStatus connected=ws_state.connected />

                <div class="relative">
                    <button
                        on:click=move |_| set_notifications_open.update(|v| *v = !*v)
                        class="p-2 text-neutral-500 hover:text-neutral-700 hover:bg-neutral-100 rounded-lg"
                    >
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
                        </svg>
                    </button>
                </div>

                <div class="h-8 w-8 bg-primary-600 rounded-full flex items-center justify-center text-white font-medium">
                    A
                </div>
            </div>
        </header>
    }
}

#[component]
fn ConnectionStatus(connected: ReadSignal<bool>) -> impl IntoView {
    view! {
        <div class="flex items-center gap-2 text-sm">
            <span
                class=move || {
                    format!(
                        "w-2 h-2 rounded-full {}",
                        if connected() { "bg-success-500" } else { "bg-danger-500" }
                    )
                }
            ></span>
            <span class="text-neutral-600">
                {move || if connected() { "Connected" } else { "Disconnected" }}
            </span>
        </div>
    }
}
