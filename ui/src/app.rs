use crate::components::cma::CmaPage;
use crate::components::comms::CommsPage;
use crate::components::leads::LeadsPage;
use crate::components::market::MarketPage;
use crate::components::shell::Shell;
use leptos::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    provide_contexts();

    view! {
        <Router>
            <Shell>
                <Routes>
                    <Route path="/" component=HomePage />
                    <Route path="/leads" component=LeadsPage />
                    <Route path="/cma" component=CmaPage />
                    <Route path="/market" component=MarketPage />
                    <Route path="/comms" component=CommsPage />
                </Routes>
            </Shell>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div class="p-6">
            <h1 class="text-2xl font-bold text-neutral-900 mb-4">Welcome to WizAI Dashboard</h1>
            <p class="text-neutral-600">Select a module from the sidebar to get started.</p>
        </div>
    }
}
