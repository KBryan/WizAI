use leptos::*;

#[derive(Clone)]
struct MarketData {
    area: String,
    avg_price: String,
    median_price: String,
    price_per_sqft: String,
    median_dom: i32,
    inventory: i32,
    sales_volume: i32,
    trend: String,
    market_status: String,
}

#[component]
pub fn MarketPage() -> impl IntoView {
    let (selected_area, set_selected_area) = create_signal("Pickering".to_string());
    let (selected_period, set_selected_period) = create_signal("90d".to_string());

    let areas = vec!["Pickering", "Ajax", "Whitby", "Oshawa", "Durham Region"];
    let periods = vec![("30d", "30 Days"), ("90d", "90 Days"), ("1y", "1 Year")];

    let mock_data = MarketData {
        area: "Pickering".to_string(),
        avg_price: "$950,000".to_string(),
        median_price: "$925,000".to_string(),
        price_per_sqft: "$620".to_string(),
        median_dom: 12,
        inventory: 145,
        sales_volume: 89,
        trend: "+2.3%".to_string(),
        market_status: "Balanced".to_string(),
    };

    view! {
        <div class="space-y-6">
            <h1 class="text-2xl font-bold text-neutral-900">Market Intelligence</h1>

            <div class="flex items-center gap-4">
                <div>
                    <label class="block text-sm font-medium text-neutral-700 mb-1">Area</label>
                    <select
                        value=selected_area()
                        on:change=move |e| set_selected_area.set(event_target_value(&e))
                        class="input w-48"
                    >
                        {areas.into_iter().map(|area| {
                            view! { <option value=area selected={selected_area() == area}>{area}</option> }
                        }).collect::<Vec<_>>()}
                    </select>
                </div>

                <div class="flex gap-1 bg-neutral-100 p-1 rounded-lg">
                    {periods.into_iter().map(|(value, label)| {
                        let value = value.to_string();
                        view! {
                            <button
                                on:click=move |_| set_selected_period.set(value.clone())
                                class={format!(
                                    "px-4 py-2 rounded-md text-sm font-medium transition-colors {}",
                                    if selected_period() == value {
                                        "bg-white shadow text-primary-600"
                                    } else {
                                        "text-neutral-600 hover:text-neutral-900"
                                    }
                                )}
                            >
                                {label}
                            </button>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>

            <div class="grid grid-cols-4 gap-4">
                <MetricCard
                    label="Average Price"
                    value=mock_data.avg_price.clone()
                    trend=mock_data.trend.clone()
                    trend_positive=true
                />
                <MetricCard
                    label="Median Price"
                    value=mock_data.median_price.clone()
                    trend="+1.8%".to_string()
                    trend_positive=true
                />
                <MetricCard
                    label="Days on Market"
                    value=format!("{} days", mock_data.median_dom)
                    trend="-3 days".to_string()
                    trend_positive=true
                />
                <MetricCard
                    label="Active Listings"
                    value=format!("{}", mock_data.inventory)
                    trend="-12%".to_string()
                    trend_positive=false
                />
            </div>

            <div class="grid grid-cols-3 gap-6">
                <div class="col-span-2">
                    <div class="card p-6">
                        <h2 class="text-lg font-semibold mb-4">Price Trend</h2>
                        <TrendChart period=selected_period() />
                    </div>
                </div>

                <div>
                    <div class="card p-6">
                        <h2 class="text-lg font-semibold mb-4">Market Status</h2>
                        <MarketStatusBadge status=mock_data.market_status.clone() />

                        <div class="mt-6 space-y-3">
                            <div class="flex justify-between">
                                <span class="text-sm text-neutral-600">Sales Volume</span>
                                <span class="font-medium">{mock_data.sales_volume}</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="text-sm text-neutral-600">Price/Sq Ft</span>
                                <span class="font-medium">{mock_data.price_per_sqft}</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div class="card">
                <div class="p-4 border-b border-neutral-200">
                    <h2 class="text-lg font-semibold">Neighborhood Comparison</h2>
                </div>
                <table class="w-full">
                    <thead class="bg-neutral-50">
                        <tr>
                            <th class="text-left px-6 py-3 text-sm font-medium text-neutral-600">Neighborhood</th>
                            <th class="text-right px-6 py-3 text-sm font-medium text-neutral-600">Avg Price</th>
                            <th class="text-right px-6 py-3 text-sm font-medium text-neutral-600">DOM</th>
                            <th class="text-right px-6 py-3 text-sm font-medium text-neutral-600">Inventory</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-neutral-200">
                        <tr class="hover:bg-neutral-50">
                            <td class="px-6 py-4 font-medium">Pickering South</td>
                            <td class="px-6 py-4 text-right">$980,000</td>
                            <td class="px-6 py-4 text-right">10</td>
                            <td class="px-6 py-4 text-right">45</td>
                        </tr>
                        <tr class="hover:bg-neutral-50">
                            <td class="px-6 py-4 font-medium">Pickering North</td>
                            <td class="px-6 py-4 text-right">$920,000</td>
                            <td class="px-6 py-4 text-right">14</td>
                            <td class="px-6 py-4 text-right">38</td>
                        </tr>
                        <tr class="hover:bg-neutral-50">
                            <td class="px-6 py-4 font-medium">Rural Pickering</td>
                            <td class="px-6 py-4 text-right">$1,150,000</td>
                            <td class="px-6 py-4 text-right">21</td>
                            <td class="px-6 py-4 text-right">22</td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>
    }
}

#[component]
fn MetricCard(label: String, value: String, trend: String, trend_positive: bool) -> impl IntoView {
    view! {
        <div class="card p-4">
            <p class="text-sm text-neutral-500 mb-1">{label}</p>
            <p class="text-2xl font-bold text-neutral-900">{value}</p>
            <p class={format!("text-sm {} mt-1", if trend_positive { "text-success-600" } else { "text-danger-600" })}>
                {trend}
            </p>
        </div>
    }
}

#[component]
fn TrendChart(period: ReadSignal<String>) -> impl IntoView {
    view! {
        <div class="h-64 flex items-end justify-between gap-2">
            {vec![65, 68, 72, 70, 75, 78, 76, 80, 82, 79, 85, 88].into_iter().enumerate().map(|(i, h)| {
                let height = format!("{}%", h);
                let labels = vec!["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
                view! {
                    <div class="flex-1 flex flex-col items-center gap-2">
                        <div class="w-full bg-primary-500 rounded-t transition-all hover:bg-primary-600" style={format!("height: {}", height)}></div>
                        <span class="text-xs text-neutral-500">{labels[i]}</span>
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}

#[component]
fn MarketStatusBadge(status: String) -> impl IntoView {
    let (color_class, bg_class, description) = match status.as_str() {
        "Seller's" => (
            "text-seller-600",
            "bg-seller-100",
            "Low inventory favors sellers",
        ),
        "Buyer's" => (
            "text-buyer-600",
            "bg-buyer-100",
            "High inventory favors buyers",
        ),
        _ => (
            "text-success-600",
            "bg-success-100",
            "Balanced supply and demand",
        ),
    };

    view! {
        <div class="text-center">
            <span class={format!("text-4xl {}", match status.as_str() {
                "Seller's" => "📈",
                "Buyer's" => "📉",
                _ => "⚖️",
            })}></span>
            <p class={format!("text-xl font-bold mt-2 {}", color_class)}>{status} Market</p>
            <p class="text-sm text-neutral-600 mt-1">{description}</p>
        </div>
    }
}
