use crate::api::leads::{self, LeadSummary, UpdateLeadRequest};
use leptos::*;

#[derive(Clone)]
pub struct DragState {
    pub dragging_lead: Option<LeadSummary>,
    pub source_status: Option<String>,
    pub over_column: Option<String>,
}

impl DragState {
    pub fn new() -> Self {
        Self {
            dragging_lead: None,
            source_status: None,
            over_column: None,
        }
    }
}

#[component]
pub fn LeadCard(
    lead: crate::components::leads::leads_page::LeadData,
    on_click: Callback<()>,
) -> impl IntoView {
    let (is_dragging, set_is_dragging) = create_signal(false);

    let drag_start = move |e: web_sys::DragEvent| {
        e.data_transfer().and_then(|dt| dt.set_data(&lead.id).ok());
        set_is_dragging.set(true);
    };

    let drag_end = move |_e: web_sys::DragEvent| {
        set_is_dragging.set(false);
    };

    let score = lead.is_urgent as i32 * 5 + 5;
    let score_class = if score >= 8 {
        "badge-hot"
    } else if score >= 5 {
        "badge-warm"
    } else {
        "badge-cold"
    };

    let score_label = if score >= 8 {
        "Hot"
    } else if score >= 5 {
        "Warm"
    } else {
        "Cold"
    };

    view! {
        <div
            draggable="true"
            on:dragstart=drag_start
            on:dragend=drag_end
            on:click=on_click
            class={format!(
                "card p-4 cursor-grab active:cursor-grabbing transition-all {}",
                if is_dragging() { "opacity-50 scale-95" } else { "hover:shadow-md hover:scale-[1.02]" }
            )}
        >
            <div class="flex items-start justify-between mb-2">
                <h4 class="font-medium text-neutral-900">{lead.name.clone()}</h4>
                <span class={score_class}>{score_label}</span>
            </div>

            <div class="text-sm text-neutral-500 mb-3">
                {lead.email.clone().unwrap_or_default()}
            </div>

            <div class="flex items-center justify-between text-xs text-neutral-400">
                <span>Source: {lead.source}</span>
                <span>{if lead.is_urgent { "🔥 Urgent" } else { "" }}</span>
            </div>

            <div class="flex gap-2 mt-3 pt-3 border-t border-neutral-100">
                <button class="flex-1 p-2 text-neutral-500 hover:text-primary-600 hover:bg-primary-50 rounded" title="Call">
                    <svg class="w-4 h-4 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 5a2 2 0 012-2h3.28a1 1 0 01.948.684l1.498 4.493a1 1 0 01-.502 1.21l-2.257 1.13a11.042 11.042 0 005.516 5.516l1.13-2.257a1 1 0 011.21-.502l4.493 1.498a1 1 0 01.684.949V19a2 2 0 01-2 2h-1C9.716 21 3 14.284 3 6V5z" />
                    </svg>
                </button>
                <button class="flex-1 p-2 text-neutral-500 hover:text-primary-600 hover:bg-primary-50 rounded" title="Email">
                    <svg class="w-4 h-4 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
                    </svg>
                </button>
                <button class="flex-1 p-2 text-neutral-500 hover:text-primary-600 hover:bg-primary-50 rounded" title="SMS">
                    <svg class="w-4 h-4 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
                    </svg>
                </button>
            </div>
        </div>
    }
}
