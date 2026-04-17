use leptos::*;

#[component]
pub fn LoadingSkeleton(props: SkeletonProps) -> impl IntoView {
    view! {
        <div class={format!("animate-pulse {}", props.class)}>
            <div class="bg-neutral-200 rounded h-full"></div>
        </div>
    }
}

#[derive(Clone)]
pub struct SkeletonProps {
    pub class: String,
}

impl SkeletonProps {
    pub fn new(class: &str) -> Self {
        Self {
            class: class.to_string(),
        }
    }
}

#[component]
pub fn LeadCardSkeleton() -> impl IntoView {
    view! {
        <div class="bg-white rounded-lg p-4 shadow-sm">
            <div class="flex items-start justify-between mb-2">
                <div class="flex-1">
                    <div class="h-4 bg-neutral-200 rounded w-3/4 mb-2 animate-pulse"></div>
                    <div class="h-3 bg-neutral-200 rounded w-1/2 animate-pulse"></div>
                </div>
                <div class="h-6 w-12 bg-neutral-200 rounded-full animate-pulse"></div>
            </div>
            <div class="h-3 bg-neutral-200 rounded w-full mb-3 animate-pulse"></div>
            <div class="flex justify-between">
                <div class="h-3 bg-neutral-200 rounded w-1/4 animate-pulse"></div>
                <div class="h-3 bg-neutral-200 rounded w-1/4 animate-pulse"></div>
            </div>
        </div>
    }
}

#[component]
pub fn PipelineSkeleton() -> impl IntoView {
    view! {
        <div class="flex-shrink-0 w-80">
            <div class="bg-neutral-100 rounded-xl p-4">
                <div class="flex items-center justify-between mb-4">
                    <div class="h-5 bg-neutral-200 rounded w-20 animate-pulse"></div>
                    <div class="h-6 w-8 bg-neutral-200 rounded-full animate-pulse"></div>
                </div>
                <div class="space-y-3 min-h-[200px]">
                    <LeadCardSkeleton />
                    <LeadCardSkeleton />
                    <LeadCardSkeleton />
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn CmaCardSkeleton() -> impl IntoView {
    view! {
        <div class="bg-white rounded-lg p-4 border border-neutral-200">
            <div class="flex justify-between items-start mb-3">
                <div class="flex-1">
                    <div class="h-5 bg-neutral-200 rounded w-4/5 mb-2 animate-pulse"></div>
                    <div class="h-3 bg-neutral-200 rounded w-1/3 animate-pulse"></div>
                </div>
                <div class="h-6 w-20 bg-neutral-200 rounded animate-pulse"></div>
            </div>
            <div class="flex items-center gap-4">
                <div class="h-4 bg-neutral-200 rounded w-12 animate-pulse"></div>
                <div class="h-4 bg-neutral-200 rounded w-12 animate-pulse"></div>
                <div class="h-4 bg-neutral-200 rounded w-16 animate-pulse"></div>
            </div>
        </div>
    }
}

#[component]
pub fn MetricCardSkeleton() -> impl IntoView {
    view! {
        <div class="bg-white rounded-xl p-4 shadow-sm">
            <div class="flex items-center gap-4">
                <div class="w-12 h-12 bg-neutral-200 rounded-lg animate-pulse"></div>
                <div class="flex-1">
                    <div class="h-3 bg-neutral-200 rounded w-3/4 mb-2 animate-pulse"></div>
                    <div class="h-6 bg-neutral-200 rounded w-1/2 animate-pulse"></div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn TableRowSkeleton() -> impl IntoView {
    view! {
        <tr class="animate-pulse">
            <td class="px-6 py-4">
                <div class="h-4 bg-neutral-200 rounded w-3/4 animate-pulse"></div>
            </td>
            <td class="px-6 py-4">
                <div class="h-4 bg-neutral-200 rounded w-1/2 ml-auto animate-pulse"></div>
            </td>
            <td class="px-6 py-4">
                <div class="h-4 bg-neutral-200 rounded w-1/4 ml-auto animate-pulse"></div>
            </td>
            <td class="px-6 py-4">
                <div class="h-4 bg-neutral-200 rounded w-1/4 ml-auto animate-pulse"></div>
            </td>
        </tr>
    }
}
