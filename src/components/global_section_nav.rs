use crate::components::section_nav::SectionNav;
use leptos::prelude::*;
use leptos_router::hooks::use_location;

/// Single mount so `SectionNav` thumb can animate between /info ↔ /tools (no full remount).
#[component]
pub fn GlobalSectionNav() -> impl IntoView {
    let location = use_location();
    let visible = Memo::new(move |_| {
        let p = location.pathname.get();
        p == "/info"
            || p == "/info/"
            || p == "/tools"
            || p == "/tools/"
    });

    view! {
        <Show when=move || visible.get()>
            <div class="section-nav-anchor">
                <SectionNav />
            </div>
        </Show>
    }
}
