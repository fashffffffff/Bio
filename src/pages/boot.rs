use crate::audio::AudioMeter;
use crate::i18n;
use crate::media::CheckStatus;
use crate::media::MusicQueue;
use crate::media_catalog::use_catalog;
use crate::site_config;
use gloo_net::http::Request;
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{AudioContext, HtmlVideoElement};

#[component]
pub fn BootPage() -> impl IntoView {
    let navigate = use_navigate();
    let meter = AudioMeter::expect();
    let audio_ref = meter.audio_ref;
    let music = expect_context::<RwSignal<MusicQueue>>();
    let catalog = use_catalog();
    let lang = i18n::use_lang();

    let init = RwSignal::new(CheckStatus::Pending);
    let apis = RwSignal::new(CheckStatus::Pending);
    let audio = RwSignal::new(CheckStatus::Pending);
    let scene = RwSignal::new(CheckStatus::Pending);
    let go = RwSignal::new(CheckStatus::Pending);

    let checks_done = RwSignal::new(false);

    let ran = Rc::new(Cell::new(false));
    Effect::new({
        let ran = ran.clone();
        move |_| {
            if ran.get() {
                return;
            }
            ran.set(true);

            spawn_local(async move {
                TimeoutFuture::new(650).await;
                init.set(CheckStatus::Ok);
                TimeoutFuture::new(900).await;

                let api_status = match Request::get(site_config::HEALTHCHECK_URL).send().await {
                    Ok(resp) if resp.ok() => CheckStatus::Ok,
                    Ok(_) => CheckStatus::Warn,
                    Err(_) => CheckStatus::Warn,
                };
                apis.set(api_status);
                TimeoutFuture::new(950).await;

                // Never await `resume()` here — Brave/Chromium may block until user gesture.
                let audio_status = match AudioContext::new() {
                    Ok(ctx) => match ctx.resume() {
                        Ok(p) => {
                            spawn_local(async move {
                                let _ = JsFuture::from(p).await;
                            });
                            CheckStatus::Ok
                        }
                        Err(_) => CheckStatus::Warn,
                    },
                    Err(_) => CheckStatus::Warn,
                };
                audio.set(audio_status);
                TimeoutFuture::new(900).await;

                scene.set(scene_probe());
                TimeoutFuture::new(850).await;

                go.set(CheckStatus::Ok);
                TimeoutFuture::new(700).await;
                checks_done.set(true);
            });
        }
    });

    let boot_bg_image = Memo::new(move |_| {
        let url = catalog
            .get()
            .boot_background
            .clone()
            .unwrap_or_default();
        if url.is_empty() {
            "none".to_string()
        } else {
            format!(
                "linear-gradient(120deg, rgba(0,0,0,0.82), rgba(0,0,0,0.55)), url('{url}')"
            )
        }
    });

    let can_continue =
        Memo::new(move |_| checks_done.get() && catalog.with(|c| c.loaded));

    let boot_progress = Memo::new(move |_| {
        let steps = [init.get(), apis.get(), audio.get(), scene.get(), go.get()];
        let done = steps
            .iter()
            .filter(|s| !matches!(s, CheckStatus::Pending))
            .count();
        (done as f64 / steps.len() as f64) * 100.0
    });

    let on_continue = move |_| {
        let tracks = catalog.get_untracked().shuffled_audio.clone();

        music.update(|q| {
            q.tracks = tracks;
            q.index = 0;
        });

        if let Some(el) = audio_ref.get_untracked() {
            if let Some(src) = music.get_untracked().current() {
                el.set_src(src);
                let _ = el.play();
            }
        }

        navigate("/info", Default::default());
    };

    view! {
        <div
            class="boot-root"
            style:background-image=move || boot_bg_image.get()
            style:background-size="cover"
            style:background-position="center"
        >
            <div class="hud-top">
                <button
                    type="button"
                    class="lang-switch"
                    on:click=move |_| lang.update(|l| *l = l.toggle())
                >
                    {move || lang.get().label()}
                </button>
            </div>

            <div class="boot-center">
                <p class="boot-kicker mono">{move || i18n::boot_title(lang.get())}</p>
                <div class="window chrome boot-window">
                    <div class="titlebar">
                        <span class="dot red"></span>
                        <span class="dot yellow"></span>
                        <span class="dot green"></span>
                        <span class="title">"system_check.exe"</span>
                    </div>
                    <div class="boot-progress" aria-hidden="true">
                        <div
                            class="boot-progress-fill"
                            style:width=move || format!("{:.1}%", boot_progress.get())
                        ></div>
                    </div>
                    <div class="window-body mono">
                        <div class="check-row">
                            <span class="check-icon mono">{move || icon(init.get())}</span>
                            <span class=move || class_for(init.get())>
                                {move || i18n::boot_checks(lang.get()).init}
                            </span>
                        </div>
                        <div class="check-row">
                            <span class="check-icon mono">{move || icon(apis.get())}</span>
                            <span class=move || class_for(apis.get())>
                                {move || i18n::boot_checks(lang.get()).apis}
                            </span>
                        </div>
                        <div class="check-row">
                            <span class="check-icon mono">{move || icon(audio.get())}</span>
                            <span class=move || class_for(audio.get())>
                                {move || i18n::boot_checks(lang.get()).audio}
                            </span>
                        </div>
                        <div class="check-row">
                            <span class="check-icon mono">{move || icon(scene.get())}</span>
                            <span class=move || class_for(scene.get())>
                                {move || i18n::boot_checks(lang.get()).scene}
                            </span>
                        </div>
                        <div class="check-row">
                            <span class="check-icon mono">{move || icon(go.get())}</span>
                            <span class=move || class_for(go.get())>
                                {move || i18n::boot_checks(lang.get()).go}
                            </span>
                        </div>
                        <div class="scanline"></div>
                    </div>
                </div>

                <button
                    type="button"
                    class="continue-btn"
                    prop:disabled=move || !can_continue.get()
                    on:click=on_continue
                >
                    <span class="continue-label">{move || i18n::boot_continue(lang.get())}</span>
                    <span class="continue-dots">"..."</span>
                </button>
            </div>
        </div>
    }
}

fn icon(s: CheckStatus) -> &'static str {
    match s {
        CheckStatus::Pending => "·",
        CheckStatus::Ok => "✓",
        CheckStatus::Warn => "!",
        CheckStatus::Fail => "✗",
    }
}

fn class_for(s: CheckStatus) -> &'static str {
    match s {
        CheckStatus::Pending => "check-text pending",
        CheckStatus::Ok => "check-text ok",
        CheckStatus::Warn => "check-text warn",
        CheckStatus::Fail => "check-text fail",
    }
}

fn scene_probe() -> CheckStatus {
    let Some(window) = web_sys::window() else {
        return CheckStatus::Warn;
    };
    let Some(document) = window.document() else {
        return CheckStatus::Warn;
    };
    match document.create_element("video") {
        Ok(el) => match el.dyn_into::<HtmlVideoElement>() {
            Ok(v) => {
                let probe = v.can_play_type("video/mp4");
                let s = probe;
                if s.is_empty() {
                    CheckStatus::Warn
                } else {
                    CheckStatus::Ok
                }
            }
            Err(_) => CheckStatus::Warn,
        },
        Err(_) => CheckStatus::Warn,
    }
}
