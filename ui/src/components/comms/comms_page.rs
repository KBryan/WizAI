use crate::components::comms::draft_modal::DraftModal;
use leptos::*;

#[derive(Clone)]
struct DraftData {
    id: String,
    draft_type: String,
    recipient: String,
    subject: String,
    risk_level: String,
    status: String,
    created_at: String,
}

#[component]
pub fn CommsPage() -> impl IntoView {
    let (drafts, _) = create_signal(vec![
        DraftData {
            id: "1".to_string(),
            draft_type: "Email".to_string(),
            recipient: "john.smith@email.com".to_string(),
            subject: "Property Viewing Confirmation - 123 Main St".to_string(),
            risk_level: "Low".to_string(),
            status: "PendingApproval".to_string(),
            created_at: "2 hours ago".to_string(),
        },
        DraftData {
            id: "2".to_string(),
            draft_type: "Email".to_string(),
            recipient: "sarah.jones@email.com".to_string(),
            subject: "CMA Report for 456 Oak Ave".to_string(),
            risk_level: "High".to_string(),
            status: "Draft".to_string(),
            created_at: "1 day ago".to_string(),
        },
        DraftData {
            id: "3".to_string(),
            draft_type: "SMS".to_string(),
            recipient: "+1 555-0123".to_string(),
            subject: "Just following up on your inquiry...".to_string(),
            risk_level: "Medium".to_string(),
            status: "Approved".to_string(),
            created_at: "3 days ago".to_string(),
        },
    ]);

    let (selected_tab, set_selected_tab) = create_signal("pending".to_string());
    let (selected_draft, set_selected_draft) = create_signal(None);

    let tabs = vec![
        ("pending", "Pending Review"),
        ("drafts", "Drafts"),
        ("sent", "Sent"),
        ("all", "All"),
    ];

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <h1 class="text-2xl font-bold text-neutral-900">Communications</h1>
                <button class="btn-primary">
                    <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                    </svg>
                    New Draft
                </button>
            </div>

            <div class="flex gap-1 border-b border-neutral-200">
                {tabs.into_iter().map(|(key, label)| {
                    let key = key.to_string();
                    view! {
                        <button
                            on:click=move |_| set_selected_tab.set(key.clone())
                            class={format!(
                                "px-4 py-3 text-sm font-medium border-b-2 -mb-px transition-colors {}",
                                if selected_tab() == key {
                                    "border-primary-600 text-primary-600"
                                } else {
                                    "border-transparent text-neutral-600 hover:text-neutral-900"
                                }
                            )}
                        >
                            {label}
                        </button>
                    }
                }).collect::<Vec<_>>()}
            </div>

            <div class="card">
                <div class="divide-y divide-neutral-200">
                    {drafts().into_iter().map(|draft| {
                        let on_click = move |_| set_selected_draft.set(Some(draft.clone()));
                        view! {
                            <DraftRow draft=draft on_click />
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>

            <Show when=selected_draft().is_some()>
                <DraftModal
                    draft=selected_draft().unwrap()
                    on_close=move |_| set_selected_draft.set(None)
                />
            </Show>
        </div>
    }
}

#[component]
fn DraftRow(draft: DraftData, on_click: Callback<()>) -> impl IntoView {
    let risk_class = match draft.risk_level.as_str() {
        "Low" => "badge-low-risk",
        "Medium" => "badge-medium-risk",
        "High" => "badge-high-risk",
        _ => "badge",
    };

    let status_class = match draft.status.as_str() {
        "Draft" => "bg-neutral-100 text-neutral-700",
        "PendingApproval" => "bg-warning-100 text-warning-700",
        "Approved" => "bg-success-100 text-success-700",
        "Sent" => "bg-primary-100 text-primary-700",
        _ => "bg-neutral-100 text-neutral-700",
    };

    view! {
        <div on:click=on_click class="p-4 hover:bg-neutral-50 cursor-pointer">
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-4">
                    <span class={format!(
                        "w-8 h-8 rounded-full flex items-center justify-center {}",
                        if draft.draft_type == "Email" { "bg-blue-100 text-blue-600" } else { "bg-green-100 text-green-600" }
                    )}>
                        {if draft.draft_type == "Email" { "✉️" } else { "💬" }}
                    </span>
                    <div>
                        <p class="font-medium text-neutral-900">{draft.subject}</p>
                        <p class="text-sm text-neutral-500">{draft.recipient}</p>
                    </div>
                </div>

                <div class="flex items-center gap-4">
                    <span class={risk_class}>{draft.risk_level}</span>
                    <span class={format!("px-2 py-1 rounded-full text-xs font-medium {}", status_class)}>
                        {draft.status.replace("PendingApproval", "Pending")}
                    </span>
                    <span class="text-sm text-neutral-400">{draft.created_at}</span>
                </div>
            </div>
        </div>
    }
}
