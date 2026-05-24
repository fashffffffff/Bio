use crate::audio::AudioMeter;
use crate::components::tilt_surface::{TiltShell, TiltSurface};
use crate::i18n::{self, Lang};
use crate::media::MusicQueue;
use crate::media_catalog::use_catalog;
use crate::site_config;
use crate::track_art;
use leptos::html::Video;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

#[component]
pub fn InfoPage() -> impl IntoView {
    let lang = i18n::use_lang();
    let meter = AudioMeter::expect();
    let audio_ref = meter.audio_ref;
    let music = expect_context::<RwSignal<MusicQueue>>();
    let catalog = use_catalog();

    let video_ref = NodeRef::<Video>::new();
    let video_playlist = RwSignal::new(Vec::<String>::new());
    let video_index = RwSignal::new(0usize);

    let current_video_src = Memo::new(move |_| {
        let list = video_playlist.get();
        let idx = video_index.get();
        list.get(idx).cloned().unwrap_or_default()
    });

    Effect::new(move |_| {
        let c = catalog.get();
        if !c.loaded || c.shuffled_video.is_empty() {
            video_playlist.set(Vec::new());
            video_index.set(0);
            return;
        }
        video_playlist.set(c.shuffled_video.clone());
        video_index.set(0);
    });

    let avatar_src = Memo::new(move |_| {
        catalog
            .get()
            .avatar_url()
            .map(|s| s.to_string())
            .unwrap_or_default()
    });

    let github_sub = Memo::new(move |_| match lang.get() {
        Lang::En => site_config::GITHUB_SUBTITLE_EN,
        Lang::Ru => site_config::GITHUB_SUBTITLE_RU,
    });

    let track_label = Memo::new(move |_| {
        music
            .get()
            .current()
            .map(|s| s.rsplit('/').next().unwrap_or(s).to_string())
            .unwrap_or_else(|| "—".to_string())
    });

    let cover_url = RwSignal::new(None::<String>);
    let cover_for = RwSignal::new(String::new());

    Effect::new(move |_| {
        let src = music
            .get()
            .current()
            .map(|s| s.to_string())
            .unwrap_or_default();

        if let Some(old) = cover_url.get_untracked() {
            track_art::revoke_object_url(&old);
        }
        cover_url.set(None);
        cover_for.set(src.clone());

        if src.is_empty() {
            return;
        }

        spawn_local(async move {
            if let Some(url) = track_art::fetch_cover_object_url(&src).await {
                if cover_for.get_untracked() == src {
                    cover_url.set(Some(url));
                } else {
                    track_art::revoke_object_url(&url);
                }
            }
        });
    });

    // Sync UI sliders with whatever Boot left on the element.
    Effect::new(move |_| {
        if let Some(a) = audio_ref.get() {
            meter.volume.set(a.volume() as f64);
            meter.paused.set(a.paused());
        }
    });

    let on_video_ended = move |_| {
        let list = video_playlist.get_untracked();
        if list.is_empty() {
            return;
        }
        video_index.update(|i| *i = (*i + 1) % list.len());
    };

    let nudge_video_play = move |_| {
        if let Some(v) = video_ref.get() {
            let _ = v.play();
        }
    };

    // Reactive `src=` on <video> often does not reload; set via DOM when the playlist updates.
    Effect::new(move |_| {
        let src = current_video_src.get();
        let Some(v) = video_ref.get() else {
            return;
        };
        if src.is_empty() {
            return;
        }
        v.set_src(&src);
        let _ = v.load();
        let _ = v.play();
    });

    let toggle_play = move |_| {
        if let Some(a) = audio_ref.get() {
            if a.paused() {
                let _ = a.play();
            } else {
                let _ = a.pause();
            }
        }
    };

    let next_track = move |_| {
        music.update(|q| q.next());
        if let Some(a) = audio_ref.get() {
            if let Some(src) = music.get_untracked().current() {
                a.set_src(src);
                let _ = a.play();
            }
        }
        meter.current.set(0.0);
    };

    let prev_track = move |_| {
        music.update(|q| q.prev());
        if let Some(a) = audio_ref.get() {
            if let Some(src) = music.get_untracked().current() {
                a.set_src(src);
                let _ = a.play();
            }
        }
        meter.current.set(0.0);
    };

    let on_volume = move |ev: leptos::ev::Event| {
        let input = ev
            .target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
        if let (Some(a), Some(input)) = (audio_ref.get(), input) {
            let v: f64 = input.value().parse().unwrap_or(0.8);
            a.set_volume(v);
        }
    };

    let on_seek = move |ev: leptos::ev::Event| {
        let input = ev
            .target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
        if let (Some(a), Some(input)) = (audio_ref.get(), input) {
            let pct: f64 = input.value().parse().unwrap_or(0.0);
            let dur = meter.duration.get_untracked();
            if dur.is_finite() && dur > 0.0 {
                a.set_current_time((pct / 100.0) * dur);
            }
        }
    };

    let progress_pct = move || {
        let t = meter.current.get();
        let d = meter.duration.get();
        if d.is_finite() && d > 0.0 {
            (t / d * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        }
    };

    let fmt_time = |secs: f64| {
        if !secs.is_finite() || secs < 0.0 {
            return "0:00".to_string();
        }
        let s = secs.floor() as u64;
        let m = s / 60;
        let r = s % 60;
        format!("{m}:{r:02}")
    };

    view! {
        <div class="info-root" class:has-video=move || !current_video_src.get().is_empty()>
            <div class="hud-top">
                <button
                    type="button"
                    class="lang-switch"
                    on:click=move |_| lang.update(|l| *l = l.toggle())
                >
                    {move || lang.get().label()}
                </button>
            </div>

            <div class="info-stack">

                <TiltSurface shell=TiltShell::Section class="profile-shell glass">
                    <div class="profile-top">
                        <div class="avatar-wrap">
                            <Show when=move || !avatar_src.get().is_empty()>
                                <img
                                    class="avatar"
                                    alt="avatar"
                                    width="184"
                                    height="184"
                                    decoding="sync"
                                    prop:src=move || avatar_src.get()
                                />
                            </Show>
                        </div>
                        <div class="identity">
                            <div class="name-row">
                                <h1 class="display-name">{site_config::DISPLAY_NAME}</h1>
                            </div>
                            <div class="handle mono">{site_config::HANDLE}</div>
                            <div class="role">{site_config::ROLE_TITLE}</div>
                            <div class="tagline">{site_config::TAGLINE}</div>
                        </div>
                        <p class="quote-eerie">{site_config::CARD_QUOTE}</p>
                    </div>

                    <div class="skills" aria-label="skills">
                        <For
                            each=move || {
                                site_config::SKILLS
                                    .iter()
                                    .enumerate()
                                    .map(|(i, s)| (i, *s))
                                    .collect::<Vec<_>>()
                            }
                            key=|(i, _)| *i
                            children=move |(_, label)| {
                                view! { <span class="skill-pill mono">{label}</span> }
                            }
                        />
                    </div>

                    <div class="links-row">
                        <a
                            class="social-link social-tg"
                            href=site_config::TELEGRAM_URL
                            target="_blank"
                            rel="noreferrer"
                        >
                            <span class="social-icon" aria-hidden="true">
                                <svg viewBox="0 0 24 24" width="22" height="22">
                                    <path
                                        fill="currentColor"
                                        d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm4.64 6.8c-.15 1.58-.8 5.42-1.13 7.19-.14.75-.42 1-.68 1.03-.58.05-1.02-.38-1.58-.75-.88-.58-1.38-.94-2.23-1.5-.99-.65-.35-1.01.22-1.59.15-.15 2.71-2.48 2.76-2.69a.2.2 0 00-.05-.18c-.06-.05-.14-.03-.21-.02-.09.02-1.49.95-4.22 2.79-.4.27-.76.41-1.08.4-.36-.01-1.04-.2-1.55-.37-.63-.2-1.12-.31-1.08-.66.02-.18.27-.36.74-.55 2.92-1.27 4.86-2.11 5.83-2.51 2.78-1.16 3.35-1.36 3.73-1.36.08 0 .27.02.39.12.1.08.13.19.14.27-.01.06.01.24 0 .38z"
                                    />
                                </svg>
                            </span>
                            <span class="social-label">
                                <span class="social-title">{move || i18n::telegram_cta(lang.get())}</span>
                                <span class="social-hint">"message"</span>
                            </span>
                        </a>
                        <a
                            class="social-link social-gh"
                            href=site_config::GITHUB_URL
                            target="_blank"
                            rel="noreferrer"
                        >
                            <span class="social-icon" aria-hidden="true">
                                <svg viewBox="0 0 24 24" width="22" height="22">
                                    <path
                                        fill="currentColor"
                                        d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A8.34 8.34 0 0024 12c0-6.63-5.37-12-12-12z"
                                    />
                                </svg>
                            </span>
                            <span class="social-label">
                                <span class="social-title">{move || i18n::github_cta(lang.get())}</span>
                                <span class="social-hint">{move || github_sub.get()}</span>
                            </span>
                        </a>
                    </div>
                </TiltSurface>
            </div>

            <div class="bg-fallback"></div>
            <video
                class="bg-video"
                class:video-off=move || current_video_src.get().is_empty()
                node_ref=video_ref
                muted
                autoplay
                playsinline
                on:ended=on_video_ended
                on:loadeddata=nudge_video_play
            ></video>

            <TiltSurface shell=TiltShell::Footer class="player player-v2 glass" max_deg=5.0>
                <div class="player-art" aria-hidden="true">
                    <Show
                        when=move || cover_url.get().is_some()
                        fallback=|| view! {
                            <div class="track-art track-art--empty">
                                <svg class="track-art-glyph" viewBox="0 0 24 24" aria-hidden="true">
                                    <circle cx="12" cy="12" r="9" fill="none" stroke="currentColor" stroke-width="1.25" />
                                    <circle cx="12" cy="12" r="2.5" fill="currentColor" />
                                </svg>
                            </div>
                        }
                    >
                        <img
                            class="track-art"
                            alt=""
                            prop:src=move || cover_url.get().unwrap_or_default()
                        />
                    </Show>
                </div>
                <div class="player-mid">
                    <div class="transport">
                        <button
                            type="button"
                            class="transport-btn transport-btn--skip transport-btn--prev"
                            aria-label="Previous track"
                            on:click=prev_track
                        >
                            <svg class="transport-icon transport-icon--skip" viewBox="0 0 24 24" aria-hidden="true">
                                <path
                                    fill="currentColor"
                                    d="M11.2 7.4c-.35 0-.63.28-.63.63v8.14c0 .5.54.8.98.57l4.9-3.2c.35-.23.35-.74 0-.97l-4.9-3.17a.74.74 0 0 0-.98.57zm5.9 0c-.35 0-.63.28-.63.63v8.14c0 .5.54.8.98.57l4.9-3.2c.35-.23.35-.74 0-.97l-4.9-3.17a.74.74 0 0 0-.98.57z"
                                />
                            </svg>
                        </button>
                        <button
                            type="button"
                            class="transport-btn transport-btn--play"
                            aria-label="Play or pause"
                            on:click=toggle_play
                        >
                            <Show
                                when=move || meter.paused.get()
                                fallback=|| view! {
                                    <svg class="transport-icon transport-icon--play" viewBox="0 0 24 24" aria-hidden="true">
                                        <rect x="8.2" y="7.5" width="2.8" height="9" rx="1.1" fill="currentColor" />
                                        <rect x="13" y="7.5" width="2.8" height="9" rx="1.1" fill="currentColor" />
                                    </svg>
                                }
                            >
                                <svg class="transport-icon transport-icon--play" viewBox="0 0 24 24" aria-hidden="true">
                                    <path
                                        fill="currentColor"
                                        d="M9.4 7.6c-.32 0-.58.26-.58.58v7.72c0 .42.45.68.82.5l6.9-3.86c.33-.18.33-.64 0-.82l-6.9-3.86a.58.58 0 0 0-.82.5z"
                                    />
                                </svg>
                            </Show>
                        </button>
                        <button
                            type="button"
                            class="transport-btn transport-btn--skip"
                            aria-label="Next track"
                            on:click=next_track
                        >
                            <svg class="transport-icon transport-icon--skip" viewBox="0 0 24 24" aria-hidden="true">
                                <path
                                    fill="currentColor"
                                    d="M6.9 7.4c-.35 0-.63.28-.63.63v8.14c0 .5.54.8.98.57l4.9-3.2c.35-.23.35-.74 0-.97l-4.9-3.17a.74.74 0 0 0-.98.57zm5.9 0c-.35 0-.63.28-.63.63v8.14c0 .5.54.8.98.57l4.9-3.2c.35-.23.35-.74 0-.97l-4.9-3.17a.74.74 0 0 0-.98.57z"
                                />
                            </svg>
                        </button>
                    </div>
                    <div class="timeline">
                        <span class="time mono">{move || fmt_time(meter.current.get())}</span>
                        <input
                            class="seek range-dense"
                            type="range"
                            min="0"
                            max="100"
                            step="0.1"
                            prop:value=move || progress_pct().to_string()
                            on:input=on_seek
                        />
                        <span class="time mono">{move || fmt_time(meter.duration.get())}</span>
                    </div>
                </div>
                <div class="player-track">
                    <span class="now-title mono" title=move || track_label.get()>
                        {move || track_label.get()}
                    </span>
                </div>

                <div class="player-vol">
                    <span class="vol-label mono">"VOL"</span>
                    <input
                        class="volume range-dense"
                        type="range"
                        min="0"
                        max="1"
                        step="0.01"
                        prop:value=move || meter.volume.get().to_string()
                        on:input=on_volume
                    />
                </div>
            </TiltSurface>
        </div>
    }
}
