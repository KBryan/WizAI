use leptos::*;
use crate::api::cma::{self, CreateCmaRequest, CmaReport, ComparablesQuery};
use crate::components::common::show_toast;
use crate::api::pdf_export::download_cma_pdf;
use crate::components::cma::comparables::ComparablesList;
use crate::components::cma::price_chart::PriceRecommendation;

#[component]
pub fn CmaPage() -> impl IntoView {
    let (address, set_address) = create_signal(String::new());
    let (property_type, set_property_type) = create_signal("Detached".to_string());
    let (is_generating, set_is_generating) = create_signal(false);
    let (is_loading_comps, set_is_loading_comps) = create_signal(false);
    let (cma_result, set_cma_result) = create_signal(None);
    let (comparables, set_comparables) = create_signal(vec![]);
    let (cma_id, set_cma_id) = create_signal(None::<String>);

    let property_types = vec![
        "Detached",
        "Semi-Detached",
        "Townhouse",
        "Condo",
        "Commercial",
    ];

    let generate_cma = move |_| {
        if address().trim().is_empty() {
            return;
        }
        
        let addr = address();
        let pt = property_type();
        
        spawn_local(async move {
            set_is_generating.set(true);
            
            match cma::create_cma(CreateCmaRequest {
                address: addr.clone(),
                property_type: pt.clone(),
                mls_number: None,
            }).await {
                Ok(response) => {
                    set_cma_id.set(Some(response.cma_id.clone()));
                    
                    let poll_interval = leptos::web_sys::window()
                        .and_then(|w| w.set_interval_with_callback_and_timeout_and_arguments(
                            Box::new(move || {
                                let cma_id_clone = response.cma_id.clone();
                                spawn_local(async move {
                                    match cma::get_cma_status(&cma_id_clone).await {
                                        Ok(status) => {
                                            if status.status == "Completed" || status.status == "approved" {
                                                match cma::get_cma(&cma_id_clone).await {
                                                    Ok(cma_report) => {
                                                        set_cma_result.set(Some(cma_report.clone()));
                                                        fetch_comparables(addr.clone(), pt.clone());
                                                    }
                                                    Err(_) => {}
                                                }
                                                set_is_generating.set(false);
                                            }
                                        }
                                        Err(_) => {}
                                    }
                                });
                            }),
                            2000,
                        ))
                        .and_then(|_| Some(()));
                    
                    if poll_interval.is_none() {
                        set_is_generating.set(false);
                    }
                }
                Err(e) => {
                    show_toast(&format!("Failed to create CMA: {}", e), crate::components::common::ToastType::Error);
                    set_is_generating.set(false);
                }
            }
        });
    };
    
    let fetch_comparables = move |addr: String, pt: String| {
        let set_comps = set_comparables.clone();
        let set_loading = set_is_loading_comps.clone();
        
        spawn_local(async move {
            set_loading.set(true);
            
            match cma::search_comparables(ComparablesQuery {
                address: addr,
                property_type: Some(pt),
                radius_km: Some(1.0),
                limit: Some(5),
            }).await {
                Ok(response) => {
                    set_comps.set(response.comparables.into_iter().map(|c| {
                        ComparableData {
                            address: c.address,
                            price: c.price as f64,
                            sqft: c.square_feet as i32,
                            beds: c.bedrooms as i32,
                            baths: c.bathrooms as i32,
                            distance: c.distance_km,
                            confidence: c.match_confidence as i32,
                        }
                    }).collect());
                }
                Err(e) => {
                    show_toast(&format!("Failed to load comparables: {}", e), crate::components::common::ToastType::Error);
                }
            }
            
            set_loading.set(false);
        });
    };

    let export_pdf = move |_| {
        if let Some(cma) = cma_result() {
            let json = serde_json::to_string(&cma).unwrap_or_default();
            let filename = format!("CMA_{}.html", cma.subject_address.replace(" ", "_"));
            let _ = download_cma_pdf(&json, &filename);
            show_toast("Export started", crate::components::common::ToastType::Success);
        }
    };

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <h1 class="text-2xl font-bold text-neutral-900">Comparative Market Analysis</h1>
            </div>

            <div class="card p-6">
                <h2 class="text-lg font-semibold mb-4">Generate CMA</h2>

                <div class="grid grid-cols-3 gap-4 mb-4">
                    <div class="col-span-2">
                        <label class="block text-sm font-medium text-neutral-700 mb-1">Property Address</label>
                        <input
                            type="text"
                            value=address()
                            on:input=move |e| set_address.set(event_target_value(&e))
                            placeholder="Enter property address (e.g., 123 Main St, Pickering)"
                            class="input"
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-neutral-700 mb-1">Property Type</label>
                        <select
                            value=property_type()
                            on:change=move |e| set_property_type.set(event_target_value(&e))
                            class="input"
                        >
                            {property_types.into_iter().map(|pt| {
                                view! { <option value=pt selected={property_type() == pt}>{pt}</option> }
                            }).collect::<Vec<_>>()}
                        </select>
                    </div>
                </div>

                <button
                    on:click=generate_cma
                    disabled=is_generating() || address().trim().is_empty()
                    class="btn-primary"
                >
                    <Show when=is_generating() fallback=|_| "Generate CMA">
                        <span class="flex items-center">
                            <span class="w-4 h-4 mr-2 border-2 border-white border-t-transparent rounded-full animate-spin"></span>
                            Generating...
                        </span>
                    </Show>
                </button>
            </div>

            <Show when=cma_result().is_some() || !comparables().is_empty()>
                <div class="grid grid-cols-3 gap-6">
                    <div class="col-span-2 space-y-6">
                        <ComparablesList comparables=comparables />
                    </div>
                    <div class="space-y-6">
                        <PriceRecommendation 
                            result=cma_result.clone()
                            comparables=comparables.clone()
                            on_export=export_pdf
                        />
                    </div>
                </div>
            </Show>
            
            <Show when=is_loading_comps()>
                <div class="card p-8 text-center">
                    <div class="inline-block w-8 h-8 border-4 border-primary-600 border-t-transparent rounded-full animate-spin"></div>
                    <p class="mt-4 text-neutral-600">Loading comparables...</p>
                </div>
            </Show>
        </div>
    }
}

#[derive(Clone)]
pub struct CmaResult {
    pub comparables: Vec<ComparableData>,
    pub price_low: f64,
    pub price_mid: f64,
    pub price_high: f64,
    pub confidence: i32,
    pub avg_days_on_market: i32,
    pub list_to_sale_ratio: f32,
}

#[derive(Clone)]
pub struct ComparableData {
    pub address: String,
    pub price: f64,
    pub sqft: i32,
    pub beds: i32,
    pub baths: i32,
    pub distance: f64,
    pub confidence: i32,
}

impl From<crate::api::cma::ComparableProperty> for ComparableData {
    fn from(c: crate::api::cma::ComparableProperty) -> Self {
        Self {
            address: c.address,
            price: c.price as f64,
            sqft: c.square_feet as i32,
            beds: c.bedrooms as i32,
            baths: c.bathrooms as i32,
            distance: c.distance_km,
            confidence: c.match_confidence as i32,
        }
    }
}
