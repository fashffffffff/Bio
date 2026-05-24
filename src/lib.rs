mod audio;
pub mod components;
mod i18n;
mod media;
mod media_catalog;
mod meta_edit;
mod pages;
mod site_config;
mod track_art;
mod util;

use audio::AudioMeter;
use i18n::Lang;
use leptos::html::Audio;
use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use media::MusicQueue;
use media_catalog::{fetch_manifest, MediaCatalog};
use components::global_section_nav::GlobalSectionNav;
use pages::{boot::BootPage, info::InfoPage, tools::ToolsPage};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlAudioElement;

#[component]
pub fn App() -> impl IntoView {
    console_error_panic_hook::set_once();

    Effect::new(move |_| {
        if let Some(w) = web_sys::window() {
            if let Some(doc) = w.document() {
                let _ = doc.set_title(crate::site_config::SITE_TITLE);
            }
        }
    });

    let audio_ref = NodeRef::<Audio>::new();
    let meter = AudioMeter {
        audio_ref,
        current: RwSignal::new(0.0),
        duration: RwSignal::new(0.0),
        paused: RwSignal::new(true),
        volume: RwSignal::new(0.75),
    };

    let music = RwSignal::new(MusicQueue::default());
    let lang = RwSignal::new(Lang::En);
    let catalog = RwSignal::new(MediaCatalog::default());

    provide_context(meter);
    provide_context(music);
    provide_context(lang);
    provide_context(catalog);

    let advance_track = {
        let music = music;
        move || {
            music.update(|q| q.next());
            if let Some(a) = meter.audio_ref.get_untracked() {
                if let Some(src) = music.get_untracked().current() {
                    a.set_src(src);
                    let _ = a.load();
                    let _ = a.play();
                } else {
                    let _ = a.pause();
                }
            }
            meter.current.set(0.0);
        }
    };

    Effect::new(move |_| {
        spawn_local(async move {
            let manifest = fetch_manifest().await;
            catalog.set(MediaCatalog::from_manifest(manifest));
        });
    });

    view! {
        <audio
            class="global-audio"
            node_ref=meter.audio_ref
            preload="metadata"
            on:timeupdate=move |ev| {
                if let Some(el) = ev
                    .target()
                    .and_then(|t| t.dyn_into::<HtmlAudioElement>().ok())
                {
                    meter.current.set(el.current_time());
                    let dur = el.duration();
                    if dur.is_finite() && dur > 0.0 {
                        meter.duration.set(dur);
                    }
                    meter.paused.set(el.paused());
                }
            }
            on:play=move |_| meter.paused.set(false)
            on:pause=move |_| meter.paused.set(true)
            on:volumechange=move |ev| {
                if let Some(el) = ev
                    .target()
                    .and_then(|t| t.dyn_into::<HtmlAudioElement>().ok())
                {
                    meter.volume.set(el.volume() as f64);
                }
            }
            on:ended=move |_| advance_track()
        />
        <Router>
            <GlobalSectionNav />
            <Routes fallback=|| view! { <p class="route-fallback">"404"</p> }>
                <Route path=path!("/") view=BootPage />
                <Route path=path!("/info") view=InfoPage />
                <Route path=path!("/tools") view=ToolsPage />
            </Routes>
        </Router>
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    leptos::mount::mount_to_body(|| view! { <App /> });
}
