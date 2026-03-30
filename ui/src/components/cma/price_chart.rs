use leptos::*;
use crate::components::cma::cma_page::{CmaResult, ComparableData};

#[component]
pub fn PriceRecommendation(
    result: Option<crate::api::cma::CmaReport>,
    comparables: Vec<ComparableData>,
    on_export: Callback<()>,
) -> impl IntoView {
    let price_low = result.as_ref().and_then(|r| r.price_recommendation_low);
    let price_mid = result.as_ref().and_then(|r| r.price_recommendation_mid);
    let price_high = result.as_ref().and_then(|r| r.price_recommendation_high);
    let confidence = result.as_ref().map(|r| r.confidence).unwrap_or(0);
    let avg_dom = result.as_ref().and_then(|r| r.avg_days_on_market);
    let ltsr = result.as_ref().and_then(|r| r.list_to_sale_ratio);
    
    view! {
        <div class="card p-6">
            <h2 class="text-lg font-semibold mb-4">Price Recommendation</h2>
            
            <div class="space-y-4">
                <div class="flex justify-between items-center">
                    <span class="text-neutral-600">Low</span>
                    <span class="text-lg font-medium">
                        {price_low.map(|p| format!("${:,.0}", p)).unwrap_or_else(|| "$N/A".to_string())}
                    </span>
                </div>
                
                <div class="relative">
                    <div class="h-10 bg-gradient-to-r from-primary-500 to-primary-600 rounded-lg flex items-center justify-center">
                        <span class="text-white font-bold text-xl">
                            {price_mid.map(|p| format!("${:,.0}", p)).unwrap_or_else(|| "$N/A".to_string())}
                        </span>
                    </div>
                </div>
                
                <div class="flex justify-between items-center">
                    <span class="text-neutral-600">High</span>
                    <span class="text-lg font-medium">
                        {price_high.map(|p| format!("${:,.0}", p)).unwrap_or_else(|| "$N/A".to_string())}
                    </span>
                </div>
            </div>
            
            <div class="mt-6 pt-6 border-t border-neutral-200">
                <div class="flex justify-between items-center mb-2">
                    <span class="text-sm text-neutral-600">Confidence</span>
                    <span class="badge bg-success-100 text-success-700">{confidence}%</span>
                </div>
                <div class="flex justify-between items-center mb-2">
                    <span class="text-sm text-neutral-600">Avg Days on Market</span>
                    <span class="text-sm font-medium">{avg_dom.map(|d| format!("{:.0} days", d)).unwrap_or_else(|| "N/A".to_string())}</span>
                </div>
                <div class="flex justify-between items-center">
                    <span class="text-sm text-neutral-600">List-to-Sale Ratio</span>
                    <span class="text-sm font-medium">{ltsr.map(|r| format!("{:.1}%", r)).unwrap_or_else(|| "N/A".to_string())}</span>
                </div>
            </div>
            
            <div class="mt-6">
                <button on:click=on_export class="btn-primary w-full">
                    <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                    </svg>
                    Export Report
                </button>
            </div>
            
            <Show when=!comparables.is_empty()>
                <div class="mt-6 pt-6 border-t border-neutral-200">
                    <h3 class="text-sm font-semibold mb-3">Comparable Prices</h3>
                    <div class="space-y-2">
                        {comparables.iter().take(5).map(|comp| {
                            view! {
                                <div class="flex justify-between text-sm">
                                    <span class="text-neutral-600 truncate flex-1">{comp.address.clone()}</span>
                                    <span class="font-medium ml-2">${:,.0}", comp.price}</span>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            </Show>
        </div>
    }
}
