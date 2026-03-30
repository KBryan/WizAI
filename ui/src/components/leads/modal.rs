use crate::api::types::*;
use leptos::*;

#[component]
pub fn LeadModal(lead: Lead, on_close: Callback<()>) -> impl IntoView {
    view! {
        <div class="fixed inset-0 z-50 flex items-center justify-center">
            <div class="absolute inset-0 bg-black/50" on:click=on_close></div>

            <div class="relative bg-white rounded-xl shadow-xl w-full max-w-2xl max-h-[90vh] overflow-hidden">
                <div class="flex items-center justify-between p-6 border-b border-neutral-200">
                    <div>
                        <h2 class="text-xl font-bold text-neutral-900">{lead.name.clone()}</h2>
                        <p class="text-sm text-neutral-500">{lead.email.clone()}</p>
                    </div>
                    <button on:click=on_close class="p-2 hover:bg-neutral-100 rounded-lg">
                        <svg class="w-5 h-5 text-neutral-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                </div>

                <div class="p-6 overflow-y-auto max-h-[calc(90vh-180px)]">
                    <div class="grid grid-cols-2 gap-6">
                        <div>
                            <h3 class="text-sm font-medium text-neutral-500 mb-2">Contact</h3>
                            <div class="space-y-2">
                                <p class="text-neutral-900">{lead.email.clone()}</p>
                                <Show when=lead.phone.is_some()>
                                    <p class="text-neutral-900">{lead.phone.clone().unwrap()}</p>
                                </Show>
                            </div>
                        </div>

                        <div>
                            <h3 class="text-sm font-medium text-neutral-500 mb-2">Status</h3>
                            <span class={format!("badge {}", match lead.score >= 8 { true => "badge-hot", false if lead.score >= 5 => "badge-warm" => "badge-cold" })}>
                                {format!("{:?}", lead.status)} - Score: {lead.score}
                            </span>
                        </div>

                        <div>
                            <h3 class="text-sm font-medium text-neutral-500 mb-2">Source</h3>
                            <p class="text-neutral-900">{format!("{:?}", lead.source)}</p>
                        </div>

                        <div>
                            <h3 class="text-sm font-medium text-neutral-500 mb-2">Timeline</h3>
                            <p class="text-neutral-900">{lead.timeline.clone().unwrap_or_else(|| "Not specified".to_string())}</p>
                        </div>

                        <Show when=lead.property_interest.is_some()>
                            <div class="col-span-2">
                                <h3 class="text-sm font-medium text-neutral-500 mb-2">Property Interest</h3>
                                <p class="text-neutral-900">{lead.property_interest.clone().unwrap()}</p>
                            </div>
                        </Show>

                        <Show when=lead.budget.is_some()>
                            <div class="col-span-2">
                                <h3 class="text-sm font-medium text-neutral-500 mb-2">Budget</h3>
                                <p class="text-neutral-900">{lead.budget.clone().unwrap()}</p>
                            </div>
                        </Show>

                        <Show when=lead.notes.is_some()>
                            <div class="col-span-2">
                                <h3 class="text-sm font-medium text-neutral-500 mb-2">Notes</h3>
                                <p class="text-neutral-900">{lead.notes.clone().unwrap()}</p>
                            </div>
                        </Show>
                    </div>
                </div>

                <div class="flex items-center justify-end gap-3 p-6 border-t border-neutral-200 bg-neutral-50">
                    <button class="btn-secondary">Edit</button>
                    <button class="btn-primary">Save Changes</button>
                </div>
            </div>
        </div>
    }
}
