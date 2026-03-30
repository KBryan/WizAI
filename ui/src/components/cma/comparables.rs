use leptos::*;

#[derive(Clone)]
struct ComparableData {
    address: String,
    price: f64,
    sqft: i32,
    beds: i32,
    baths: i32,
    distance: f64,
    confidence: i32,
}

#[component]
pub fn ComparablesList(comparables: Vec<ComparableData>) -> impl IntoView {
    view! {
        <div class="card">
            <div class="p-4 border-b border-neutral-200">
                <h2 class="text-lg font-semibold">Comparable Properties</h2>
                <p class="text-sm text-neutral-500">{comparables.len()} properties found</p>
            </div>

            <div class="divide-y divide-neutral-200">
                {comparables.into_iter().map(|comp| {
                    view! {
                        <div class="p-4 hover:bg-neutral-50">
                            <div class="flex items-start justify-between">
                                <div>
                                    <h3 class="font-medium text-neutral-900">{comp.address}</h3>
                                    <p class="text-sm text-neutral-500">{comp.distance} km away</p>
                                </div>
                                <div class="text-right">
                                    <p class="text-lg font-bold text-neutral-900">{format!("${:.0}", comp.price)}</p>
                                    <p class="text-xs text-neutral-500">{comp.confidence}% match</p>
                                </div>
                            </div>

                            <div class="flex items-center gap-4 mt-3 text-sm text-neutral-600">
                                <span>{comp.beds} beds</span>
                                <span>{comp.baths} baths</span>
                                <span>{comp.sqft} sqft</span>
                                <span>{format!("${:.0}/sqft", comp.price / comp.sqft as f64)}</span>
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
