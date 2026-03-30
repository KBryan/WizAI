use leptos::*;

struct DraftData {
    id: String,
    draft_type: String,
    recipient: String,
    subject: String,
    content: String,
    risk_level: String,
    status: String,
    created_at: String,
}

#[component]
pub fn DraftModal(draft: DraftData, on_close: Callback<()>) -> impl IntoView {
    let risk_class = match draft.risk_level.as_str() {
        "Low" => "badge-low-risk",
        "Medium" => "badge-medium-risk",
        "High" => "badge-high-risk",
        _ => "badge",
    };

    view! {
            <div class="fixed inset-0 z-50 flex items-center justify-center">
                <div class="absolute inset-0 bg-black/50" on:click=on_close></div>

                <div class="relative bg-white rounded-xl shadow-xl w-full max-w-3xl max-h-[90vh] overflow-hidden">
                    <div class="flex items-center justify-between p-6 border-b border-neutral-200">
                        <div>
                            <h2 class="text-xl font-bold text-neutral-900">{draft.subject.clone()}</h2>
                            <p class="text-sm text-neutral-500">To: {draft.recipient.clone()}</p>
                        </div>
                        <button on:click=on_close class="p-2 hover:bg-neutral-100 rounded-lg">
                            <svg class="w-5 h-5 text-neutral-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                            </svg>
                        </button>
                    </div>

                    <div class="p-6 overflow-y-auto max-h-[calc(90vh-180px)]">
                        <div class="flex items-center gap-4 mb-6">
                            <span class={risk_class}>{draft.risk_level} Risk</span>
                            <span class="px-2 py-1 rounded-full text-xs font-medium bg-neutral-100 text-neutral-700">
                                {draft.status.replace("PendingApproval", "Pending Review")}
                            </span>
                            <span class="text-sm text-neutral-500">Created: {draft.created_at}</span>
                        </div>

                        <div class="bg-neutral-50 rounded-lg p-4 mb-6">
                            <h3 class="text-sm font-medium text-neutral-700 mb-2">Message Preview</h3>
                            <p class="text-neutral-900 whitespace-pre-wrap">
                                Dear {draft.recipient.split('@').next().unwrap_or("Client")},

    Thank you for your interest in the property. I wanted to follow up on our recent conversation and confirm our scheduled viewing.

    Property: 123 Main Street, Pickering
    Date: March 15, 2026
    Time: 2:00 PM

    Please let me know if this time works for you, or if you'd like to reschedule.

    Best regards,
    Your Agent
                            </p>
                        </div>
                    </div>

                    <div class="flex items-center justify-between p-6 border-t border-neutral-200 bg-neutral-50">
                        <div class="flex gap-3">
                            <button class="btn-danger">
                                <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                                </svg>
                                Reject
                            </button>
                        </div>
                        <div class="flex gap-3">
                            <button class="btn-secondary">Edit</button>
                            <button class="btn-success">
                                <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                                </svg>
                                Approve & Send
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        }
}
