use leptos::*;
use crate::api::leads::{self, LeadSummary, UpdateLeadRequest};
use crate::components::common::show_toast;
use crate::components::leads::stats::StatsBar;
use crate::components::leads::modal::LeadModal;

#[component]
pub fn LeadsPage() -> impl IntoView {
    let (leads, set_leads) = create_signal(vec![]);
    let (is_loading, set_is_loading) = create_signal(true);
    let (selected_lead, set_selected_lead) = create_signal(None);
    let (error, set_error) = create_signal(None::<String>);
    
    let load_leads = move || {
        spawn_local(async move {
            set_is_loading.set(true);
            set_error.set(None);
            
            match leads::list_leads(None, Some(1), Some(100)).await {
                Ok(response) => {
                    set_leads.set(response.items);
                }
                Err(e) => {
                    set_error.set(Some(e.clone()));
                    show_toast(&format!("Failed to load leads: {}", e), crate::components::common::ToastType::Error);
                    set_leads.set(vec![]);
                }
            }
            
            set_is_loading.set(false);
        });
    };
    
    let refresh_lead = move |lead_id: String, new_status: String| {
        let leads_clone = leads.clone();
        let set_leads_clone = set_leads.clone();
        
        spawn_local(async move {
            let request = UpdateLeadRequest {
                status: Some(new_status.clone()),
                ..Default::default()
            };
            
            match leads::update_lead(&lead_id, request).await {
                Ok(_) => {
                    let updated_leads: Vec<LeadSummary> = leads_clone()
                        .into_iter()
                        .map(|l| {
                            if l.id == lead_id {
                                LeadSummary {
                                    status: new_status.clone(),
                                    ..l
                                }
                            } else {
                                l
                            }
                        })
                        .collect();
                    set_leads_clone.set(updated_leads);
                    show_toast("Lead status updated", crate::components::common::ToastType::Success);
                }
                Err(e) => {
                    show_toast(&format!("Failed to update lead: {}", e), crate::components::common::ToastType::Error);
                    load_leads();
                }
            }
        });
    };
    
    on_mount(load_leads);
    
    let status_columns = vec![
        ("new", "New", "bg-blue-100 text-blue-800"),
        ("qualified", "Qualified", "bg-green-100 text-green-800"),
        ("contacted", "Contacted", "bg-yellow-100 text-yellow-800"),
        ("negotiating", "Negotiating", "bg-orange-100 text-orange-800"),
        ("closed_won", "Closed Won", "bg-emerald-100 text-emerald-800"),
        ("closed_lost", "Closed Lost", "bg-red-100 text-red-800"),
    ];
    
    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <h1 class="text-2xl font-bold text-neutral-900">Lead Pipeline</h1>
                <div class="flex gap-2">
                    <button on:click=move |_| load_leads() class="btn-secondary">
                        <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                        </svg>
                        Refresh
                    </button>
                    <button class="btn-primary">
                        <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                        </svg>
                        Add Lead
                    </button>
                </div>
            </div>
            
            <Show when=is_loading()>
                <div class="card p-8 text-center">
                    <div class="inline-block w-8 h-8 border-4 border-primary-600 border-t-transparent rounded-full animate-spin"></div>
                    <p class="mt-4 text-neutral-600">Loading leads...</p>
                </div>
            </Show>
            
            <Show when=error().is_some()>
                <div class="card p-4 bg-danger-50 border-danger-200">
                    <p class="text-danger-700">{error().unwrap()}</p>
                </div>
            </Show>
            
            <Show when=!is_loading() && error().is_none()>
                <StatsBar leads=leads />
                
                <div class="flex gap-4 overflow-x-auto pb-4">
                    {status_columns.into_iter().map(|(status, label, color_class)| {
                        let status_filter = status.to_string();
                        let label_s = label.to_string();
                        let color_c = color_class.to_string();
                        let on_drop = Callback::new(move |(lead_id, _new_status): (String, String)| {
                            refresh_lead(lead_id, status_filter.clone());
                        });
                        view! {
                            <PipelineColumn
                                status=status_filter
                                label=label_s
                                color_class=color_c
                                leads=leads
                                on_drop=on_drop
                                on_lead_click=move |lead| set_selected_lead.set(Some(lead))
                            />
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </Show>
            
            <Show when=selected_lead().is_some()>
                <LeadModal
                    lead_id=selected_lead().unwrap().id
                    on_close=move |_| set_selected_lead.set(None)
                />
            </Show>
        </div>
    }
}

#[derive(Clone)]
pub struct LeadData {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub status: String,
    pub source: String,
    pub is_urgent: bool,
    pub created_at: String,
}

impl From<LeadSummary> for LeadData {
    fn from(ls: LeadSummary) -> Self {
        Self {
            id: ls.id,
            name: ls.name,
            email: ls.email,
            phone: ls.phone,
            status: ls.status,
            source: ls.source,
            is_urgent: ls.is_urgent,
            created_at: ls.created_at,
        }
    }
}

#[component]
fn PipelineColumn(
    status: String,
    label: String,
    color_class: String,
    leads: ReadSignal<Vec<LeadSummary>>,
    on_drop: Callback<(String, String)>,
    on_lead_click: Callback<LeadSummary>,
) -> impl IntoView {
    let (is_drag_over, set_is_drag_over) = create_signal(false);
    
    let column_leads = Signal::derive(move || {
        leads()
            .into_iter()
            .filter(|lead| lead.status.to_lowercase() == status.to_lowercase())
            .collect::<Vec<_>>()
    });
    
    let drag_over = move |e: web_sys::DragEvent| {
        e.prevent_default();
        set_is_drag_over.set(true);
    };
    
    let drag_leave = move |_e: web_sys::DragEvent| {
        set_is_drag_over.set(false);
    };
    
    let drop = move |e: web_sys::DragEvent| {
        e.prevent_default();
        set_is_drag_over.set(false);
        
        if let Some(data_transfer) = e.data_transfer() {
            if let Ok(lead_id) = data_transfer.get_data(&status) {
                on_drop.emit((lead_id, status.clone()));
            }
        }
    };
    
    view! {
        <div class="flex-shrink-0 w-80">
            <div
                on:dragover=drag_over
                on:dragleave=drag_leave
                on:drop=drop
                class={format!(
                    "bg-neutral-100 rounded-xl p-4 transition-colors {}",
                    if is_drag_over() { "bg-primary-50 ring-2 ring-primary-300 ring-inset" } else { "" }
                )}
            >
                <div class="flex items-center justify-between mb-4">
                    <h3 class="font-semibold text-neutral-900">{label}</h3>
                    <span class={format!("px-2 py-1 rounded-full text-xs font-medium {}", color_class)}>
                        {column_leads().len()}
                    </span>
                </div>
                
                <div class="space-y-3 min-h-[200px]">
                    {column_leads().into_iter().map(|lead| {
                        let lead_data: LeadData = lead.clone().into();
                        let on_click = on_lead_click.clone();
                        view! {
                            <DraggableLeadCard lead=lead_data on_click=move |_| on_click(lead.clone()) />
                        }
                    }).collect::<Vec<_>>()}
                    
                    {if column_leads().is_empty() {
                        view! {
                            <div class="text-center py-8 text-neutral-400 text-sm">
                                {if is_drag_over() { "Drop lead here" } else { "No leads in this stage" }}
                            </div>
                        }
                    } else {
                        view! { <></> }
                    }}
                </div>
            </div>
        </div>
    }
}

#[component]
fn DraggableLeadCard(lead: LeadData, on_click: Callback<()>) -> impl IntoView {
    let (is_dragging, set_is_dragging) = create_signal(false);
    
    let drag_start = move |e: web_sys::DragEvent| {
        e.data_transfer()
            .and_then(|dt| {
                let _ = dt.set_data(&lead.id);
            });
        set_is_dragging.set(true);
    };
    
    let drag_end = move |_e: web_sys::DragEvent| {
        set_is_dragging.set(false);
    };
    
    let score = if lead.is_urgent { 9 } else { 6 };
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
                "bg-white rounded-lg p-4 shadow-sm cursor-grab active:cursor-grabbing transition-all {}",
                if is_dragging() { "opacity-50 scale-95 shadow-md" } else { "hover:shadow-md hover:scale-[1.02]" }
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
                <span>{if lead.is_urgent { "🔥" } else { "" }}</span>
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
