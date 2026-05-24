use crate::util;
use gloo_net::http::Request;
use leptos::prelude::*;
use serde::Deserialize;

pub const MANIFEST_URL: &str = "/media/manifest.json";

#[derive(Clone, Debug, Default, Deserialize)]
pub struct MediaManifest {
    #[serde(default)]
    pub ava: Vec<String>,
    #[serde(default)]
    pub video: Vec<String>,
    #[serde(default)]
    pub audio: Vec<String>,
    #[serde(default)]
    pub background: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct MediaCatalog {
    pub loaded: bool,
    pub manifest: MediaManifest,
    pub boot_background: Option<String>,
    pub shuffled_video: Vec<String>,
    pub shuffled_audio: Vec<String>,
}

impl Default for MediaCatalog {
    fn default() -> Self {
        Self {
            loaded: false,
            manifest: MediaManifest::default(),
            boot_background: None,
            shuffled_video: Vec::new(),
            shuffled_audio: Vec::new(),
        }
    }
}

impl MediaCatalog {
    pub fn from_manifest(manifest: MediaManifest) -> Self {
        let boot_background = util::pick_random(&manifest.background);
        let shuffled_video = util::shuffle_copy(&manifest.video);
        let shuffled_audio = util::shuffle_copy(&manifest.audio);
        Self {
            loaded: true,
            manifest,
            boot_background,
            shuffled_video,
            shuffled_audio,
        }
    }

    pub fn avatar_url(&self) -> Option<&str> {
        self.manifest.ava.first().map(|s| s.as_str())
    }
}

pub async fn fetch_manifest() -> MediaManifest {
    match Request::get(MANIFEST_URL).send().await {
        Ok(resp) if resp.ok() => resp.json().await.unwrap_or_default(),
        _ => MediaManifest::default(),
    }
}

pub fn use_catalog() -> RwSignal<MediaCatalog> {
    expect_context::<RwSignal<MediaCatalog>>()
}
