use leptos::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <div class="min-h-screen bg-neutral-50">
                <Nav/>
                <main class="container mx-auto px-4 py-8">
                    <Routes>
                        <Route path="/" view=HomePage/>
                        <Route path="/leads" view=LeadsPage/>
                        <Route path="/cma" view=CmaPage/>
                        <Route path="/market" view=MarketPage/>
                        <Route path="/comms" view=CommsPage/>
                    </Routes>
                </main>
            </div>
        </Router>
    }
}

#[component]
fn Nav() -> impl IntoView {
    view! {
        <nav class="bg-neutral-900 text-white">
            <div class="container mx-auto px-4">
                <div class="flex items-center justify-between h-16">
                    <div class="flex items-center gap-2">
                        <span class="text-xl font-bold text-primary-400">"WizAI"</span>
                        <span class="text-sm text-neutral-400">"Agent Dashboard"</span>
                    </div>
                    <div class="flex items-center gap-6">
                        <A href="/" class="hover:text-primary-400 transition-colors">"Home"</A>
                        <A href="/leads" class="hover:text-primary-400 transition-colors">"Leads"</A>
                        <A href="/cma" class="hover:text-primary-400 transition-colors">"CMA"</A>
                        <A href="/market" class="hover:text-primary-400 transition-colors">"Market"</A>
                        <A href="/comms" class="hover:text-primary-400 transition-colors">"Comms"</A>
                    </div>
                </div>
            </div>
        </nav>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div class="space-y-8">
            <div class="text-center py-12">
                <h1 class="text-4xl font-bold text-neutral-900 mb-4">"Welcome to WizAI Dashboard"</h1>
                <p class="text-lg text-neutral-600 max-w-2xl mx-auto">
                    "Your AI-powered real estate assistant. Manage leads, generate CMAs, track market trends, and handle communications all in one place."
                </p>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <DashboardCard
                    title="Leads"
                    count="24"
                    description="Active leads in pipeline"
                    href="/leads"
                />
                <DashboardCard
                    title="CMAs"
                    count="8"
                    description="Reports generated this month"
                    href="/cma"
                />
                <DashboardCard
                    title="Market"
                    count="5"
                    description="Areas being tracked"
                    href="/market"
                />
                <DashboardCard
                    title="Comms"
                    count="12"
                    description="Drafts pending approval"
                    href="/comms"
                />
            </div>
        </div>
    }
}

#[component]
fn DashboardCard(
    title: &'static str,
    count: &'static str,
    description: &'static str,
    href: &'static str,
) -> impl IntoView {
    view! {
        <a href=href class="block p-6 bg-white rounded-xl shadow-sm border border-neutral-200 hover:shadow-md hover:border-primary-300 transition-all">
            <h3 class="text-sm font-medium text-neutral-500 uppercase tracking-wide">{title}</h3>
            <p class="text-3xl font-bold text-neutral-900 mt-2">{count}</p>
            <p class="text-sm text-neutral-600 mt-1">{description}</p>
        </a>
    }
}

#[component]
fn LeadsPage() -> impl IntoView {
    let (leads, _set_leads) = create_signal(vec![
        ("John Smith", "Hot", "Buyer", 9),
        ("Sarah Johnson", "Warm", "Seller", 7),
        ("Mike Davis", "Cold", "Buyer", 4),
        ("Emily Chen", "Hot", "Buyer", 8),
    ]);

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <h1 class="text-2xl font-bold text-neutral-900">"Lead Pipeline"</h1>
                <button class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors">
                    "Add Lead"
                </button>
            </div>

            <div class="grid grid-cols-3 gap-4">
                <div class="bg-neutral-100 rounded-lg p-4">
                    <h3 class="font-semibold text-neutral-700 mb-4">"New" <span class="text-sm font-normal text-neutral-500">"(2)"</span></h3>
                    <div class="space-y-3">
                        {move || leads.get().iter().filter(|l| l.2 == "Buyer").take(2).map(|(name, temp, _, score)| {
                            view! {
                                <LeadCard name=*name temperature=*temp score=*score/>
                            }
                        }).collect_view()}
                    </div>
                </div>
                <div class="bg-neutral-100 rounded-lg p-4">
                    <h3 class="font-semibold text-neutral-700 mb-4">"Qualified" <span class="text-sm font-normal text-neutral-500">"(1)"</span></h3>
                    <div class="space-y-3">
                        {move || leads.get().iter().filter(|l| l.2 == "Seller").map(|(name, temp, _, score)| {
                            view! {
                                <LeadCard name=*name temperature=*temp score=*score/>
                            }
                        }).collect_view()}
                    </div>
                </div>
                <div class="bg-neutral-100 rounded-lg p-4">
                    <h3 class="font-semibold text-neutral-700 mb-4">"Closing" <span class="text-sm font-normal text-neutral-500">"(1)"</span></h3>
                    <div class="space-y-3">
                        <LeadCard name="Emily Chen" temperature="Hot" score=8/>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn LeadCard(name: &'static str, temperature: &'static str, score: i32) -> impl IntoView {
    let badge_class = match temperature {
        "Hot" => "bg-red-100 text-red-700",
        "Warm" => "bg-orange-100 text-orange-700",
        _ => "bg-blue-100 text-blue-700",
    };

    view! {
        <div class="bg-white p-4 rounded-lg shadow-sm border border-neutral-200 cursor-pointer hover:shadow-md transition-shadow">
            <div class="flex items-start justify-between">
                <div>
                    <h4 class="font-medium text-neutral-900">{name}</h4>
                    <span class={format!("inline-block mt-1 px-2 py-0.5 text-xs font-medium rounded {}", badge_class)}>
                        {temperature}
                    </span>
                </div>
                <div class="text-right">
                    <span class="text-lg font-bold text-primary-600">{score}</span>
                    <p class="text-xs text-neutral-500">"Score"</p>
                </div>
            </div>
        </div>
    }
}

#[component]
fn CmaPage() -> impl IntoView {
    let (address, set_address) = create_signal(String::new());
    let (result, set_result) = create_signal(None::<(f64, f64, f64)>);

    let generate = move |_| {
        if !address.get().is_empty() {
            set_result.set(Some((650000.0, 675000.0, 700000.0)));
        }
    };

    view! {
        <div class="space-y-6">
            <h1 class="text-2xl font-bold text-neutral-900">"CMA Generator"</h1>

            <div class="bg-white p-6 rounded-xl shadow-sm border border-neutral-200">
                <div class="flex gap-4">
                    <input
                        type="text"
                        placeholder="Enter property address..."
                        class="flex-1 px-4 py-2 border border-neutral-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500 outline-none"
                        on:input=move |ev| set_address.set(event_target_value(&ev))
                        prop:value=address
                    />
                    <button
                        on:click=generate
                        class="px-6 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors disabled:opacity-50"
                    >
                        "Generate CMA"
                    </button>
                </div>
            </div>

            {move || result.get().map(|(low, mid, high)| view! {
                <div class="bg-white p-6 rounded-xl shadow-sm border border-neutral-200">
                    <h2 class="text-lg font-semibold text-neutral-900 mb-4">"Price Recommendation"</h2>
                    <div class="grid grid-cols-3 gap-4 text-center">
                        <div class="p-4 bg-neutral-50 rounded-lg">
                            <p class="text-sm text-neutral-500">"Low"</p>
                            <p class="text-2xl font-bold text-neutral-900">{format!("${:.0}", low)}</p>
                        </div>
                        <div class="p-4 bg-primary-50 rounded-lg border-2 border-primary-500">
                            <p class="text-sm text-primary-600">"Recommended"</p>
                            <p class="text-2xl font-bold text-primary-700">{format!("${:.0}", mid)}</p>
                        </div>
                        <div class="p-4 bg-neutral-50 rounded-lg">
                            <p class="text-sm text-neutral-500">"High"</p>
                            <p class="text-2xl font-bold text-neutral-900">{format!("${:.0}", high)}</p>
                        </div>
                    </div>
                </div>
            })}
        </div>
    }
}

#[component]
fn MarketPage() -> impl IntoView {
    let areas = vec![
        ("Pickering", 725000, 3.2, 12),
        ("Ajax", 695000, 2.8, 15),
        ("Whitby", 710000, 3.5, 11),
        ("Oshawa", 625000, 4.1, 18),
    ];

    view! {
        <div class="space-y-6">
            <h1 class="text-2xl font-bold text-neutral-900">"Market Intelligence"</h1>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                {areas.into_iter().map(|(area, avg_price, change, dom)| view! {
                    <div class="bg-white p-6 rounded-xl shadow-sm border border-neutral-200">
                        <h3 class="text-lg font-semibold text-neutral-900">{area}</h3>
                        <div class="mt-4 grid grid-cols-3 gap-4">
                            <div>
                                <p class="text-sm text-neutral-500">"Avg Price"</p>
                                <p class="text-lg font-bold text-neutral-900">{format!("${:.0}K", avg_price as f64 / 1000.0)}</p>
                            </div>
                            <div>
                                <p class="text-sm text-neutral-500">"YoY Change"</p>
                                <p class={format!("text-lg font-bold {}", if change > 0.0 { "text-green-600" } else { "text-red-600" })}>
                                    {format!("{:+.1}%", change)}
                                </p>
                            </div>
                            <div>
                                <p class="text-sm text-neutral-500">"Avg DOM"</p>
                                <p class="text-lg font-bold text-neutral-900">{dom}" days"</p>
                            </div>
                        </div>
                    </div>
                }).collect_view()}
            </div>
        </div>
    }
}

#[component]
fn CommsPage() -> impl IntoView {
    let drafts = vec![
        ("Follow-up Email", "John Smith", "Medium", "Pending"),
        ("Listing Presentation", "Sarah Johnson", "Low", "Approved"),
        ("Offer Letter", "Mike Davis", "High", "Pending"),
    ];

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <h1 class="text-2xl font-bold text-neutral-900">"Communication Hub"</h1>
                <button class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors">
                    "New Draft"
                </button>
            </div>

            <div class="bg-white rounded-xl shadow-sm border border-neutral-200 overflow-hidden">
                <table class="w-full">
                    <thead class="bg-neutral-50 border-b border-neutral-200">
                        <tr>
                            <th class="px-6 py-3 text-left text-xs font-medium text-neutral-500 uppercase">"Type"</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-neutral-500 uppercase">"Recipient"</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-neutral-500 uppercase">"Risk"</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-neutral-500 uppercase">"Status"</th>
                            <th class="px-6 py-3 text-left text-xs font-medium text-neutral-500 uppercase">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-neutral-200">
                        {drafts.into_iter().map(|(draft_type, recipient, risk, status)| {
                            let risk_class = match risk {
                                "High" => "bg-red-100 text-red-700",
                                "Medium" => "bg-yellow-100 text-yellow-700",
                                _ => "bg-green-100 text-green-700",
                            };
                            let status_class = match status {
                                "Approved" => "bg-green-100 text-green-700",
                                _ => "bg-yellow-100 text-yellow-700",
                            };
                            view! {
                                <tr class="hover:bg-neutral-50">
                                    <td class="px-6 py-4 text-sm font-medium text-neutral-900">{draft_type}</td>
                                    <td class="px-6 py-4 text-sm text-neutral-600">{recipient}</td>
                                    <td class="px-6 py-4">
                                        <span class={format!("px-2 py-1 text-xs font-medium rounded {}", risk_class)}>{risk}</span>
                                    </td>
                                    <td class="px-6 py-4">
                                        <span class={format!("px-2 py-1 text-xs font-medium rounded {}", status_class)}>{status}</span>
                                    </td>
                                    <td class="px-6 py-4">
                                        <button class="text-primary-600 hover:text-primary-800 text-sm font-medium">"Review"</button>
                                    </td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
            </div>
        </div>
    }
}
