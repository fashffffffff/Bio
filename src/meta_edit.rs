use id3::frame::{Picture, PictureType};
use id3::Frame;
use id3::Tag;
use id3::TagLike;
use id3::Version;
use std::io::Cursor;

#[derive(Clone, Debug, Default)]
pub struct Mp3MetaView {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub has_cover: bool,
}

/// Split MP3 into ID3 tag (if any) and raw audio bytes after the tag.
pub fn read_mp3_meta(bytes: &[u8]) -> Option<(Mp3MetaView, Vec<u8>)> {
    let mut cursor = Cursor::new(bytes);
    match Tag::read_from2(&mut cursor) {
        Ok(tag) => {
            let audio_start = cursor.position() as usize;
            let audio = bytes.get(audio_start..)?.to_vec();
            let view = Mp3MetaView {
                title: tag.title().map(str::to_string).unwrap_or_default(),
                artist: tag.artist().map(str::to_string).unwrap_or_default(),
                album: tag.album().map(str::to_string).unwrap_or_default(),
                has_cover: tag.pictures().next().is_some(),
            };
            Some((view, audio))
        }
        Err(_) => {
            // No ID3v2 header at start — treat entire file as MPEG audio.
            Some((Mp3MetaView::default(), bytes.to_vec()))
        }
    }
}

/// `new_cover`: replace embedded picture (front cover). If `None`, keep pictures from `keep_cover_from` when present.
pub fn write_mp3_meta(
    audio: &[u8],
    view: &Mp3MetaView,
    keep_cover_from: Option<&Tag>,
    new_cover: Option<(Vec<u8>, String)>,
) -> Option<Vec<u8>> {
    let mut tag = Tag::new();
    if !view.title.is_empty() {
        tag.set_title(view.title.clone());
    }
    if !view.artist.is_empty() {
        tag.set_artist(view.artist.clone());
    }
    if !view.album.is_empty() {
        tag.set_album(view.album.clone());
    }

    if let Some((data, mime)) = new_cover {
        if !data.is_empty() && !mime.is_empty() {
            let pic = Picture {
                mime_type: mime,
                picture_type: PictureType::CoverFront,
                description: String::new(),
                data,
            };
            let _ = tag.add_frame(Frame::from(pic));
        }
    } else if let Some(old) = keep_cover_from {
        for p in old.pictures() {
            let _ = tag.add_frame(Frame::from(p.clone()));
        }
    }

    let mut header = Vec::new();
    tag.write_to(&mut header, Version::Id3v24).ok()?;
    let mut out = header;
    out.extend_from_slice(audio);
    Some(out)
}

pub fn read_mp3_tag(bytes: &[u8]) -> Option<Tag> {
    let mut cursor = Cursor::new(bytes);
    Tag::read_from2(&mut cursor).ok()
}

pub fn mime_for_image_bytes(name: &str, data: &[u8]) -> Option<String> {
    let lower = name.to_lowercase();
    if lower.ends_with(".png") {
        return Some("image/png".to_string());
    }
    if lower.ends_with(".webp") {
        return Some("image/webp".to_string());
    }
    if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        return Some("image/jpeg".to_string());
    }
    // Magic sniff
    if data.len() >= 8 && data.starts_with(&[0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]) {
        return Some("image/png".to_string());
    }
    if data.len() >= 3 && data[0] == 0xff && data[1] == 0xd8 && data[2] == 0xff {
        return Some("image/jpeg".to_string());
    }
    if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        return Some("image/webp".to_string());
    }
    None
}
