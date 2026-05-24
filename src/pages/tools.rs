use crate::i18n;
use crate::meta_edit::{mime_for_image_bytes, read_mp3_meta, read_mp3_tag, write_mp3_meta, Mp3MetaView};
use crate::track_art;
use js_sys::{Array, Uint8Array};
use leptos::prelude::*;
use leptos::html::Input;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};

/// ~300 MiB — caps memory use in the browser (all processing is client-side).
const MAX_TOOL_FILE_BYTES: f64 = 300.0 * 1024.0 * 1024.0;

#[component]
pub fn ToolsPage() -> impl IntoView {
    let lang = i18n::use_lang();
    let file_input = NodeRef::<Input>::new();
    let cover_input = NodeRef::<Input>::new();

    let file_name = RwSignal::new(String::new());
    let status = RwSignal::new(String::new());
    let mp3 = RwSignal::new(None::<Mp3MetaView>);
    let audio_body = RwSignal::new(None::<Vec<u8>>);
    let original_tag = RwSignal::new(None::<id3::Tag>);

    let title = RwSignal::new(String::new());
    let artist = RwSignal::new(String::new());
    let album = RwSignal::new(String::new());

    let new_cover = RwSignal::new(None::<(Vec<u8>, String)>);
    let cover_preview_url = RwSignal::new(None::<String>);

    let on_pick = move |_| {
        if let Some(input) = file_input.get() {
            let _ = input.click();
        }
    };

    let on_pick_cover = move |_| {
        if let Some(input) = cover_input.get() {
            let _ = input.click();
        }
    };

    let on_file = move |ev: leptos::ev::Event| {
        let input = ev
            .target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
        let Some(input) = input else {
            return;
        };
        let Some(file_list) = input.files() else {
            return;
        };
        if file_list.length() == 0 {
            return;
        };
        let Some(file) = file_list.get(0) else {
            return;
        };

        if file.size() > MAX_TOOL_FILE_BYTES {
            status.set(i18n::tools_file_too_large(lang.get_untracked()).to_string());
            return;
        }

        let picked_name = file.name();
        file_name.set(picked_name.clone());
        status.set(i18n::tools_loading(lang.get_untracked()).to_string());

        revoke_cover_preview_url(&cover_preview_url);
        new_cover.set(None);

        spawn_local(async move {
            match read_file_bytes(&file).await {
                Ok(bytes) => {
                    let lower = picked_name.to_lowercase();
                    if lower.ends_with(".mp3") {
                        if let Some((view, audio)) = read_mp3_meta(&bytes) {
                            original_tag.set(read_mp3_tag(&bytes));
                            title.set(view.title.clone());
                            artist.set(view.artist.clone());
                            album.set(view.album.clone());
                            mp3.set(Some(view));
                            audio_body.set(Some(audio));
                            status.set(i18n::tools_mp3_ready(lang.get_untracked()).to_string());
                        }
                    } else if lower.ends_with(".jpg")
                        || lower.ends_with(".jpeg")
                        || lower.ends_with(".png")
                        || lower.ends_with(".webp")
                    {
                        clear_mp3(
                            &mp3,
                            &audio_body,
                            &original_tag,
                            &title,
                            &artist,
                            &album,
                            &new_cover,
                            &cover_preview_url,
                        );
                        status.set(i18n::tools_photo_soon(lang.get_untracked()).to_string());
                    } else if lower.ends_with(".mp4")
                        || lower.ends_with(".webm")
                        || lower.ends_with(".mov")
                    {
                        clear_mp3(
                            &mp3,
                            &audio_body,
                            &original_tag,
                            &title,
                            &artist,
                            &album,
                            &new_cover,
                            &cover_preview_url,
                        );
                        status.set(i18n::tools_video_soon(lang.get_untracked()).to_string());
                    } else {
                        clear_mp3(
                            &mp3,
                            &audio_body,
                            &original_tag,
                            &title,
                            &artist,
                            &album,
                            &new_cover,
                            &cover_preview_url,
                        );
                        status.set(i18n::tools_unknown(lang.get_untracked()).to_string());
                    }
                }
                Err(_) => {
                    status.set(i18n::tools_read_fail(lang.get_untracked()).to_string());
                }
            }
        });
    };

    let on_cover_file = move |ev: leptos::ev::Event| {
        let Some(mp3_loaded) = mp3.get() else {
            return;
        };
        let _ = mp3_loaded;

        let input = ev
            .target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
        let Some(input) = input else {
            return;
        };
        let Some(file_list) = input.files() else {
            return;
        };
        if file_list.length() == 0 {
            return;
        };
        let Some(file) = file_list.get(0) else {
            return;
        };

        if file.size() > MAX_TOOL_FILE_BYTES {
            status.set(i18n::tools_file_too_large(lang.get_untracked()).to_string());
            return;
        }

        let cover_name = file.name();
        spawn_local(async move {
            match read_file_bytes(&file).await {
                Ok(bytes) => {
                    let Some(mime) = mime_for_image_bytes(&cover_name, &bytes) else {
                        status.set(i18n::tools_cover_invalid(lang.get_untracked()).to_string());
                        return;
                    };
                    if let Some(url) = cover_preview_url.get_untracked() {
                        track_art::revoke_object_url(&url);
                    }
                    cover_preview_url.set(None);

                    new_cover.set(Some((bytes.clone(), mime.clone())));

                    if let Some(url) = bytes_to_preview_url(&bytes, &mime) {
                        cover_preview_url.set(Some(url));
                    }

                    mp3.update(|m| {
                        if let Some(v) = m.as_mut() {
                            v.has_cover = true;
                        }
                    });
                    status.set(i18n::tools_cover_updated(lang.get_untracked()).to_string());
                }
                Err(_) => {
                    status.set(i18n::tools_read_fail(lang.get_untracked()).to_string());
                }
            }
        });
    };

    let on_clear_cover = move |_| {
        revoke_cover_preview_url(&cover_preview_url);
        new_cover.set(None);
        let had_orig = original_tag
            .get_untracked()
            .as_ref()
            .map(|t| t.pictures().next().is_some())
            .unwrap_or(false);
        mp3.update(|m| {
            if let Some(v) = m.as_mut() {
                v.has_cover = had_orig;
            }
        });
        status.set(i18n::tools_cover_cleared(lang.get_untracked()).to_string());
    };

    let on_export = move |_| {
        let Some(audio) = audio_body.get_untracked() else {
            return;
        };
        let cover_override = new_cover.get_untracked();
        let view = Mp3MetaView {
            title: title.get_untracked(),
            artist: artist.get_untracked(),
            album: album.get_untracked(),
            has_cover: mp3
                .get_untracked()
                .map(|m| m.has_cover)
                .unwrap_or(false),
        };
        let tag_ref = original_tag.get_untracked();
        let tag_borrow = tag_ref.as_ref();
        let Some(out) =
            write_mp3_meta(&audio, &view, tag_borrow, cover_override) else {
            status.set(i18n::tools_write_fail(lang.get_untracked()).to_string());
            return;
        };
        let name = file_name.get_untracked();
        let download_name = if name.is_empty() {
            "edited.mp3".to_string()
        } else {
            let stem = name.trim_end_matches(".mp3");
            format!("{stem}_edited.mp3")
        };
        if download_bytes(&out, &download_name, "audio/mpeg") {
            status.set(i18n::tools_export_ok(lang.get_untracked()).to_string());
        } else {
            status.set(i18n::tools_export_fail(lang.get_untracked()).to_string());
        }
    };

    view! {
        <div class="tools-root">
            <div class="hud-top">
                <button
                    type="button"
                    class="lang-switch"
                    on:click=move |_| lang.update(|l| *l = l.toggle())
                >
                    {move || lang.get().label()}
                </button>
            </div>

            <div class="tools-stack">

                <section class="tools-card glass">
                    <h2 class="tools-title mono">{move || i18n::tools_meta_title(lang.get())}</h2>
                    <p class="tools-body">{move || i18n::tools_meta_lead(lang.get())}</p>

                    <input
                        class="tools-file-input"
                        type="file"
                        accept="audio/mpeg,.mp3,image/*,video/*"
                        node_ref=file_input
                        on:change=on_file
                    />

                    <input
                        class="tools-file-input"
                        type="file"
                        accept="image/jpeg,image/png,image/webp,.jpg,.jpeg,.png,.webp"
                        node_ref=cover_input
                        on:change=on_cover_file
                    />

                    <button type="button" class="tools-btn mono" on:click=on_pick>
                        {move || i18n::tools_pick_file(lang.get())}
                    </button>

                    <p class="tools-status mono">{move || status.get()}</p>
                    <Show when=move || file_name.get().is_empty() == false>
                        <p class="tools-filename mono">{move || file_name.get()}</p>
                    </Show>

                    <Show when=move || mp3.get().is_some()>
                        <div class="meta-form">
                            <label class="meta-field">
                                <span class="meta-label mono">"title"</span>
                                <input
                                    class="meta-input mono"
                                    type="text"
                                    prop:value=move || title.get()
                                    on:input=move |ev| {
                                        if let Some(el) = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                                            title.set(el.value());
                                        }
                                    }
                                />
                            </label>
                            <label class="meta-field">
                                <span class="meta-label mono">"artist"</span>
                                <input
                                    class="meta-input mono"
                                    type="text"
                                    prop:value=move || artist.get()
                                    on:input=move |ev| {
                                        if let Some(el) = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                                            artist.set(el.value());
                                        }
                                    }
                                />
                            </label>
                            <label class="meta-field">
                                <span class="meta-label mono">"album"</span>
                                <input
                                    class="meta-input mono"
                                    type="text"
                                    prop:value=move || album.get()
                                    on:input=move |ev| {
                                        if let Some(el) = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                                            album.set(el.value());
                                        }
                                    }
                                />
                            </label>

                            <div class="meta-cover-row">
                                <button type="button" class="tools-btn mono" on:click=on_pick_cover>
                                    {move || i18n::tools_pick_cover(lang.get())}
                                </button>
                                <Show when=move || new_cover.get().is_some()>
                                    <button type="button" class="tools-btn tools-btn--ghost mono" on:click=on_clear_cover>
                                        {move || i18n::tools_clear_cover(lang.get())}
                                    </button>
                                </Show>
                            </div>
                            <Show when=move || cover_preview_url.get().is_some()>
                                <img
                                    class="meta-cover-preview"
                                    alt="cover preview"
                                    prop:src=move || cover_preview_url.get().unwrap_or_default()
                                />
                            </Show>

                            <p class="tools-hint mono">
                                {move || {
                                    if new_cover.get().is_some() {
                                        i18n::tools_cover_updated(lang.get())
                                    } else if mp3.get().map(|m| m.has_cover).unwrap_or(false) {
                                        i18n::tools_cover_kept(lang.get())
                                    } else {
                                        i18n::tools_no_cover(lang.get())
                                    }
                                }}
                            </p>
                            <button type="button" class="tools-btn tools-btn--accent mono" on:click=on_export>
                                {move || i18n::tools_download_mp3(lang.get())}
                            </button>
                        </div>
                    </Show>

                    <div class="tools-roadmap">
                        <p class="tools-hint mono">{move || i18n::tools_roadmap_photo(lang.get())}</p>
                        <p class="tools-hint mono">{move || i18n::tools_roadmap_video(lang.get())}</p>
                    </div>
                </section>
            </div>
        </div>
    }
}

fn revoke_cover_preview_url(sig: &RwSignal<Option<String>>) {
    if let Some(url) = sig.get_untracked() {
        track_art::revoke_object_url(&url);
    }
    sig.set(None);
}

fn clear_mp3(
    mp3: &RwSignal<Option<Mp3MetaView>>,
    audio_body: &RwSignal<Option<Vec<u8>>>,
    original_tag: &RwSignal<Option<id3::Tag>>,
    title: &RwSignal<String>,
    artist: &RwSignal<String>,
    album: &RwSignal<String>,
    new_cover: &RwSignal<Option<(Vec<u8>, String)>>,
    cover_preview_url: &RwSignal<Option<String>>,
) {
    revoke_cover_preview_url(cover_preview_url);
    mp3.set(None);
    audio_body.set(None);
    original_tag.set(None);
    title.set(String::new());
    artist.set(String::new());
    album.set(String::new());
    new_cover.set(None);
    cover_preview_url.set(None);
}

fn bytes_to_preview_url(data: &[u8], mime: &str) -> Option<String> {
    let array = Uint8Array::from(data);
    let parts = Array::new();
    parts.push(&array);
    let opts = BlobPropertyBag::new();
    opts.set_type(mime);
    let blob = Blob::new_with_u8_array_sequence_and_options(&parts, &opts).ok()?;
    Url::create_object_url_with_blob(&blob).ok()
}

async fn read_file_bytes(file: &web_sys::File) -> Result<Vec<u8>, ()> {
    use wasm_bindgen_futures::JsFuture;
    let promise = file.array_buffer();
    let buf = JsFuture::from(promise).await.map_err(|_| ())?;
    let array = Uint8Array::new(&buf);
    Ok(array.to_vec())
}

fn download_bytes(data: &[u8], filename: &str, mime: &str) -> bool {
    let array = Uint8Array::from(data);
    let parts = Array::new();
    parts.push(&array);
    let opts = BlobPropertyBag::new();
    opts.set_type(mime);
    let Some(blob) = Blob::new_with_u8_array_sequence_and_options(&parts, &opts).ok() else {
        return false;
    };
    let Some(url) = Url::create_object_url_with_blob(&blob).ok() else {
        return false;
    };
    let Some(window) = web_sys::window() else {
        return false;
    };
    let Some(document) = window.document() else {
        return false;
    };
    let Ok(el) = document.create_element("a") else {
        return false;
    };
    let Ok(a) = el.dyn_into::<HtmlAnchorElement>() else {
        return false;
    };
    a.set_href(&url);
    a.set_download(filename);
    a.click();
    let _ = Url::revoke_object_url(&url);
    true
}
