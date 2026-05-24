use crate::i18n;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

/// INFO / TOOLS segmented control: dim inactive tab, frosted sliding thumb.
#[component]
pub fn SectionNav() -> impl IntoView {
    let lang = i18n::use_lang();
    let location = use_location();

    let info_active = Memo::new({
        move |_| {
            let path = location.pathname.get();
            path == "/info" || path == "/info/"
        }
    });
    let tools_active = Memo::new({
        move |_| {
            let path = location.pathname.get();
            path == "/tools" || path == "/tools/"
        }
    });

    view! {
        <nav class="section-switcher" aria-label="Section">
            <div class="section-switcher__track">
                <div class=move || {
                    if tools_active.get() {
                        "section-switcher__thumb section-switcher__thumb--tools"
                    } else {
                        "section-switcher__thumb"
                    }
                }></div>
                <div class="section-switcher__links">
                    <A href="/info">
                        <span class=move || {
                            if info_active.get() {
                                "section-switcher__link section-switcher__link--active"
                            } else {
                                "section-switcher__link section-switcher__link--dim"
                            }
                        }>
                            {move || i18n::section_info(lang.get())}
                        </span>
                    </A>
                    <A href="/tools">
                        <span class=move || {
                            if tools_active.get() {
                                "section-switcher__link section-switcher__link--active"
                            } else {
                                "section-switcher__link section-switcher__link--dim"
                            }
                        }>
                            {move || i18n::section_tools(lang.get())}
                        </span>
                    </A>
                </div>
            </div>
        </nav>
    }
}
