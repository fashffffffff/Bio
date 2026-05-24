use leptos::html::{Footer, Section};
use leptos::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use wasm_bindgen::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{DeviceOrientationEvent, PointerEvent};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TiltShell {
    Section,
    Footer,
}

static NEXT_TILT_ID: AtomicU32 = AtomicU32::new(1);

thread_local! {
    static ORIENT_LISTENERS: RefCell<HashMap<u32, Closure<dyn FnMut(DeviceOrientationEvent)>>> =
        RefCell::new(HashMap::new());
}

fn prefers_reduced_motion() -> bool {
    web_sys::window()
        .and_then(|w| w.match_media("(prefers-reduced-motion: reduce)").ok())
        .flatten()
        .map(|m| m.matches())
        .unwrap_or(false)
}

fn clamp(v: f64, min: f64, max: f64) -> f64 {
    v.max(min).min(max)
}

fn remove_orient_listener(id: u32) {
    ORIENT_LISTENERS.with(|map| {
        if let Some(closure) = map.borrow_mut().remove(&id) {
            if let Some(window) = web_sys::window() {
                let _ = window.remove_event_listener_with_callback(
                    "deviceorientation",
                    closure.as_ref().unchecked_ref(),
                );
            }
        }
    });
}

async fn orientation_allowed() -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };

    let ctor = match js_sys::Reflect::get(&window, &JsValue::from_str("DeviceOrientationEvent")) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let req = match js_sys::Reflect::get(&ctor, &JsValue::from_str("requestPermission")) {
        Ok(v) => v,
        Err(_) => return true,
    };

    if !req.is_function() {
        return true;
    }

    let func: js_sys::Function = req.unchecked_into();
    let promise_val = match func.call0(&ctor) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let promise: js_sys::Promise = promise_val.unchecked_into();
    match JsFuture::from(promise).await {
        Ok(result) => result.as_string().as_deref() == Some("granted"),
        Err(_) => false,
    }
}

fn pointer_tilt(ev: &PointerEvent, el: &web_sys::Element, max_deg: f64) -> (f64, f64) {
    let rect = el.get_bounding_client_rect();
    let w = rect.width().max(1.0);
    let h = rect.height().max(1.0);
    let cx = rect.left() + w * 0.5;
    let cy = rect.top() + h * 0.5;
    let nx = ((ev.client_x() as f64 - cx) / (w * 0.5)).clamp(-1.0, 1.0);
    let ny = ((ev.client_y() as f64 - cy) / (h * 0.5)).clamp(-1.0, 1.0);
    let rx = clamp(-ny * max_deg * 0.88, -max_deg, max_deg);
    let ry = clamp(nx * max_deg * 0.88, -max_deg, max_deg);
    (rx, ry)
}

/// Holographic tilt: gyro on phone (tap once on iOS), pointer fallback elsewhere.
#[component]
pub fn TiltSurface(
    shell: TiltShell,
    #[prop(into)] class: String,
    #[prop(default = 7.0)] max_deg: f64,
    children: Children,
) -> AnyView {
    let rx = RwSignal::new(0.0_f64);
    let ry = RwSignal::new(0.0_f64);
    let gyro_on = RwSignal::new(false);
    let pointer_down = RwSignal::new(false);
    let base_beta = RwSignal::new(None::<f64>);
    let base_gamma = RwSignal::new(None::<f64>);

    let tilt_id = NEXT_TILT_ID.fetch_add(1, Ordering::Relaxed);
    let combined_class = format!("tilt-surface {class}");

    let mount_gyro = {
        let rx = rx;
        let ry = ry;
        let gyro_on = gyro_on;
        let base_beta = base_beta;
        let base_gamma = base_gamma;
        move || {
            if gyro_on.get_untracked() || prefers_reduced_motion() {
                return;
            }
            spawn_local(async move {
                if !orientation_allowed().await {
                    return;
                }
                let Some(window) = web_sys::window() else {
                    return;
                };

                let id = tilt_id;
                let closure = Closure::wrap(Box::new(move |ev: DeviceOrientationEvent| {
                    let Some(beta) = ev.beta() else {
                        return;
                    };
                    let Some(gamma) = ev.gamma() else {
                        return;
                    };

                    if base_beta.get_untracked().is_none() {
                        base_beta.set(Some(beta));
                        base_gamma.set(Some(gamma));
                    }
                    let b0 = base_beta.get_untracked().unwrap_or(beta);
                    let g0 = base_gamma.get_untracked().unwrap_or(gamma);

                    rx.set(clamp((beta - b0) * 0.38, -max_deg, max_deg));
                    ry.set(clamp((gamma - g0) * 0.48, -max_deg, max_deg));
                }) as Box<dyn FnMut(DeviceOrientationEvent)>);

                let _ = window.add_event_listener_with_callback(
                    "deviceorientation",
                    closure.as_ref().unchecked_ref(),
                );

                ORIENT_LISTENERS.with(|map| {
                    map.borrow_mut().insert(id, closure);
                });
                gyro_on.set(true);
            });
        }
    };

    Effect::new({
        let mount_gyro = mount_gyro.clone();
        move |_| {
            if !prefers_reduced_motion() {
                mount_gyro();
            }
            on_cleanup(move || remove_orient_listener(tilt_id));
        }
    });

    let tilt_style = move || {
        if prefers_reduced_motion() {
            return String::new();
        }
        format!(
            "--tilt-rx: {:.2}deg; --tilt-ry: {:.2}deg",
            rx.get(),
            ry.get()
        )
    };

    match shell {
        TiltShell::Section => {
            let surface_ref = NodeRef::<Section>::new();
            let mount_gyro = mount_gyro.clone();
            view! {
                <section
                    node_ref=surface_ref
                    class=combined_class.clone()
                    style=tilt_style
                    on:pointerdown=move |_| {
                        pointer_down.set(true);
                        mount_gyro();
                    }
                    on:pointerup=move |_| {
                        pointer_down.set(false);
                        if !gyro_on.get_untracked() {
                            rx.set(0.0);
                            ry.set(0.0);
                        }
                    }
                    on:pointercancel=move |_| {
                        pointer_down.set(false);
                        if !gyro_on.get_untracked() {
                            rx.set(0.0);
                            ry.set(0.0);
                        }
                    }
                    on:pointermove=move |ev: PointerEvent| {
                        if prefers_reduced_motion() {
                            return;
                        }
                        if gyro_on.get_untracked() && !pointer_down.get_untracked() {
                            return;
                        }
                        let Some(el) = surface_ref.get() else {
                            return;
                        };
                        let (tx, ty) = pointer_tilt(&ev, &el, max_deg);
                        rx.set(tx);
                        ry.set(ty);
                    }
                    on:pointerleave=move |_| {
                        pointer_down.set(false);
                        if !gyro_on.get_untracked() {
                            rx.set(0.0);
                            ry.set(0.0);
                        }
                    }
                >
                    {children()}
                </section>
            }
            .into_any()
        }
        TiltShell::Footer => {
            let surface_ref = NodeRef::<Footer>::new();
            let mount_gyro = mount_gyro.clone();
            view! {
                <footer
                    node_ref=surface_ref
                    class=combined_class
                    style=tilt_style
                    on:pointerdown=move |_| {
                        pointer_down.set(true);
                        mount_gyro();
                    }
                    on:pointerup=move |_| {
                        pointer_down.set(false);
                        if !gyro_on.get_untracked() {
                            rx.set(0.0);
                            ry.set(0.0);
                        }
                    }
                    on:pointercancel=move |_| {
                        pointer_down.set(false);
                        if !gyro_on.get_untracked() {
                            rx.set(0.0);
                            ry.set(0.0);
                        }
                    }
                    on:pointermove=move |ev: PointerEvent| {
                        if prefers_reduced_motion() {
                            return;
                        }
                        if gyro_on.get_untracked() && !pointer_down.get_untracked() {
                            return;
                        }
                        let Some(el) = surface_ref.get() else {
                            return;
                        };
                        let (tx, ty) = pointer_tilt(&ev, &el, max_deg);
                        rx.set(tx);
                        ry.set(ty);
                    }
                    on:pointerleave=move |_| {
                        pointer_down.set(false);
                        if !gyro_on.get_untracked() {
                            rx.set(0.0);
                            ry.set(0.0);
                        }
                    }
                >
                    {children()}
                </footer>
            }
            .into_any()
        }
    }
}
