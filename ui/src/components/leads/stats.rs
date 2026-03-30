use leptos::*;

#[component]
pub fn StatsBar(leads: ReadSignal<Vec<crate::api::types::Lead>>) -> impl IntoView {
    let stats = Signal::derive(move || {
        let total = leads().len();
        let hot_leads = leads().iter().filter(|l| l.score >= 8).count();
        let avg_score = if total > 0 {
            leads().iter().map(|l| l.score as f32).sum::<f32>() / total as f32
        } else {
            0.0
        };
        let closed_won = leads()
            .iter()
            .filter(|l| format!("{:?}", l.status) == "ClosedWon")
            .count();
        let conversion_rate = if total > 0 {
            (closed_won as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        (total, hot_leads, avg_score, conversion_rate)
    });

    view! {
        <div class="grid grid-cols-4 gap-4">
            <StatCard
                label="Total Leads"
                value=move || format!("{}", stats().0)
                icon="users"
                color="primary"
            />
            <StatCard
                label="Hot Leads"
                value=move || format!("{}", stats().1)
                icon="fire"
                color="danger"
            />
            <StatCard
                label="Avg Score"
                value=move || format!("{:.1}", stats().2)
                icon="chart"
                color="warning"
            />
            <StatCard
                label="Conversion Rate"
                value=move || format!("{:.1}%", stats().3)
                icon="check"
                color="success"
            />
        </div>
    }
}

#[component]
fn StatCard(
    label: &'static str,
    value: impl Fn() -> String + 'static,
    icon: &'static str,
    color: &'static str,
) -> impl IntoView {
    let (color_class, bg_class) = match color {
        "primary" => ("text-primary-600", "bg-primary-50"),
        "danger" => ("text-danger-600", "bg-danger-50"),
        "warning" => ("text-warning-600", "bg-warning-50"),
        "success" => ("text-success-600", "bg-success-50"),
        _ => ("text-neutral-600", "bg-neutral-50"),
    };

    let icon_path = match icon {
        "users" => "M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0z",
        "fire" => "M17.657 18.657A8 8 0 016.343 7.343S7 9 9 10c0-2 .5-5 2.986-7C14 5 16.09 5.777 17.656 7.343A7.975 7.975 0 0120 13a7.975 7.975 0 01-2.343 5.657z",
        "chart" => "M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z",
        "check" => "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z",
        _ => "",
    };

    view! {
        <div class="card p-4">
            <div class="flex items-center gap-4">
                <div class={format!("p-3 rounded-lg {}", bg_class)}>
                    <svg class={format!("w-6 h-6 {}", color_class)} fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={icon_path} />
                    </svg>
                </div>
                <div>
                    <p class="text-sm text-neutral-500">{label}</p>
                    <p class="text-2xl font-bold text-neutral-900">{value()}</p>
                </div>
            </div>
        </div>
    }
}
