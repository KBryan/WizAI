use crate::components::common::ToastContainer;
use crate::components::copilot::CopilotPanel;
use crate::components::header::Header;
use crate::components::sidebar::Sidebar;
use crate::state::ws::WebSocketState;
use leptos::*;

#[component]
pub fn Shell(children: Children) -> impl IntoView {
    provide(WebSocketState::new());

    let (copilot_open, set_copilot_open) = create_signal(false);
    provide_context(|| CopilotOpen(create_signal(false)));

    view! {
        <div class="h-screen flex flex-col bg-neutral-50">
            <Header />

            <div class="flex flex-1 overflow-hidden">
                <Sidebar on_copilot_click=move |_| set_copilot_open.update(|v| *v = !*v) />

                <main class="flex-1 overflow-auto p-6">
                    {children()}
                </main>
            </div>

            <ToastContainer />

            <Show when=copilot_open()>
                <CopilotPanel on_close=move |_| set_copilot_open.update(|v| *v = false) />
            </Show>
        </div>
    }
}

#[derive(Clone)]
pub struct CopilotOpen(pub Signal<bool>);

impl CopilotOpen {
    pub fn toggle(&self) {
        self.0.update(|v| *v = !*v);
    }
}
