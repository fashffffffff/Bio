use id3::Tag;
use js_sys::{Array, Uint8Array};
use std::io::Cursor;
use web_sys::{Blob, BlobPropertyBag, Url};

/// Fetches the start of an MP3 and returns a blob object URL for embedded cover art, if any.
pub async fn fetch_cover_object_url(track_url: &str) -> Option<String> {
    let bytes = fetch_tag_bytes(track_url).await?;
    let tag = Tag::read_from2(&mut Cursor::new(bytes)).ok()?;
    let picture = tag.pictures().next()?;
    let mime = if picture.mime_type.is_empty() {
        "image/jpeg"
    } else {
        picture.mime_type.as_str()
    };
    bytes_to_object_url(&picture.data, mime)
}

async fn fetch_tag_bytes(track_url: &str) -> Option<Vec<u8>> {
    use gloo_net::http::Request;

    let ranged = Request::get(track_url)
        .header("Range", "bytes=0-262143")
        .send()
        .await
        .ok()
        .filter(|r| r.ok())?
        .binary()
        .await
        .ok();

    if let Some(b) = ranged {
        if !b.is_empty() {
            return Some(b);
        }
    }

    Request::get(track_url)
        .send()
        .await
        .ok()
        .filter(|r| r.ok())?
        .binary()
        .await
        .ok()
}

fn bytes_to_object_url(data: &[u8], mime: &str) -> Option<String> {
    let array = Uint8Array::from(data);
    let parts = Array::new();
    parts.push(&array);
    let opts = BlobPropertyBag::new();
    opts.set_type(mime);
    let blob = Blob::new_with_u8_array_sequence_and_options(&parts, &opts).ok()?;
    Url::create_object_url_with_blob(&blob).ok()
}

pub fn revoke_object_url(url: &str) {
    if url.is_empty() {
        return;
    }
    let _ = Url::revoke_object_url(url);
}
