use leptos::*;
use leptos_router::{use_location, use_navigate};

#[component]
pub fn Sidebar(on_copilot_click: Callback<()>) -> impl IntoView {
    let navigate = use_navigate();

    let nav_items = vec![
        ("/", "Home", "M3 7 10l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 8.5 4.293 4.207a1 1 0 010-1.414L5.707.379a1 1 0 011.414-1.414l4.293 4.293L15.707.379a1 1 0 011.414 1.414L12.414 6.5l4.293 4.293a1 1 0 01-1.414 1.414L11 7.914z"),
        ("/leads", "Leads", "M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"),
        ("/cma", "CMA", "M9 17v-2m3 2v-4m3 4v-6m2 10H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"),
        ("/market", "Market", "M3 3h18v18H3zM3 9h18M9 21V9"),
        ("/comms", "Comms", "M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"),
    ];

    view! {
        <aside class="w-64 bg-neutral-900 text-white flex flex-col">
            <div class="p-4 border-b border-neutral-800">
                <h1 class="text-xl font-bold flex items-center gap-2">
                    <span class="text-2xl">W</span>
                    WizAI
                </h1>
                <p class="text-sm text-neutral-400">Agent Dashboard</p>
            </div>

            <nav class="flex-1 p-4 space-y-1">
                {nav_items.iter().map(|(path, label, icon)| {
                    let path = path.to_string();
                    let is_active = use_location().pathname == *path;
                    view! {
                        <a
                            href=path.clone()
                            class=move || {
                                format!(
                                    "flex items-center gap-3 px-3 py-2 rounded-lg transition-colors {}",
                                    if is_active {
                                        "bg-primary-600 text-white".to_string()
                                    } else {
                                        "text-neutral-300 hover:bg-neutral-800 hover:text-white".to_string()
                                    }
                                )
                            }
                        >
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d=icon />
                            </svg>
                            {label}
                        </a>
                    }
                }).collect::<Vec<_>>()}
            </nav>

            <div class="p-4 border-t border-neutral-800">
                <button
                    on:click=on_copilot_click
                    class="w-full flex items-center justify-center gap-2 px-4 py-3 bg-primary-600 hover:bg-primary-700 rounded-lg transition-colors"
                >
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
                    </svg>
                    AI Copilot
                </button>
            </div>
        </aside>
    }
}
