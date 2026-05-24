use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    En,
    Ru,
}

impl Lang {
    pub fn toggle(self) -> Self {
        match self {
            Lang::En => Lang::Ru,
            Lang::Ru => Lang::En,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Lang::En => "EN",
            Lang::Ru => "RU",
        }
    }
}

pub fn boot_title(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "SYSTEM_LOADER",
        Lang::Ru => "SYSTEM_LOADER",
    }
}

pub fn boot_continue(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "> CLICK TO CONTINUE",
        Lang::Ru => "> НАЖМИ, ЧТОБЫ ПРОДОЛЖИТЬ",
    }
}

pub fn boot_checks(lang: Lang) -> BootCheckLabels {
    match lang {
        Lang::En => BootCheckLabels {
            init: "system initialized",
            apis: "apis reachable",
            audio: "audio pipeline",
            scene: "scene & effects ready",
            go: "all systems go",
        },
        Lang::Ru => BootCheckLabels {
            init: "система инициализирована",
            apis: "API / сеть доступны",
            audio: "аудио-конвейер",
            scene: "сцена и эффекты",
            go: "можно продолжать",
        },
    }
}

pub struct BootCheckLabels {
    pub init: &'static str,
    pub apis: &'static str,
    pub scene: &'static str,
    pub audio: &'static str,
    pub go: &'static str,
}

pub fn section_info(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "INFO",
        Lang::Ru => "INFO",
    }
}

pub fn section_tools(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "TOOLS",
        Lang::Ru => "TOOLS",
    }
}

pub fn telegram_cta(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Telegram",
        Lang::Ru => "Telegram",
    }
}

pub fn github_cta(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "GitHub",
        Lang::Ru => "GitHub",
    }
}

pub fn tools_meta_title(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "META EDITOR",
        Lang::Ru => "РЕДАКТОР МЕТА",
    }
}

pub fn tools_meta_lead(lang: Lang) -> &'static str {
    match lang {
        Lang::En => {
            "Edit file metadata in the browser. MP3 tags work today; photos and video are planned."
        }
        Lang::Ru => {
            "Редактирование метаданных в браузере. MP3 уже работает; фото и видео — в планах."
        }
    }
}

pub fn tools_pick_file(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Choose file",
        Lang::Ru => "Выбрать файл",
    }
}

pub fn tools_loading(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "reading file…",
        Lang::Ru => "читаю файл…",
    }
}

pub fn tools_mp3_ready(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "MP3 loaded — edit tags below",
        Lang::Ru => "MP3 загружен — правь теги ниже",
    }
}

pub fn tools_photo_soon(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "EXIF for photos — coming next (camera, GPS, device)",
        Lang::Ru => "EXIF для фото — скоро (камера, GPS, устройство)",
    }
}

pub fn tools_video_soon(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "MP4 metadata — coming later (title, cover, device info)",
        Lang::Ru => "метаданные MP4 — позже (название, обложка, устройство)",
    }
}

pub fn tools_unknown(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "unsupported file type",
        Lang::Ru => "тип файла не поддерживается",
    }
}

pub fn tools_read_fail(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "could not read file",
        Lang::Ru => "не удалось прочитать файл",
    }
}

pub fn tools_write_fail(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "could not write MP3 tags",
        Lang::Ru => "не удалось записать теги MP3",
    }
}

pub fn tools_export_ok(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "download started",
        Lang::Ru => "скачивание началось",
    }
}

pub fn tools_export_fail(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "download failed",
        Lang::Ru => "не удалось скачать",
    }
}

pub fn tools_download_mp3(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Download edited MP3",
        Lang::Ru => "Скачать изменённый MP3",
    }
}

pub fn tools_file_too_large(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "file exceeds 300 MB limit",
        Lang::Ru => "файл больше 300 МБ",
    }
}

pub fn tools_pick_cover(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Change cover art (JPEG, PNG, WebP)",
        Lang::Ru => "Сменить обложку (JPEG, PNG, WebP)",
    }
}

pub fn tools_cover_updated(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "cover loaded — include in download",
        Lang::Ru => "обложка загружена — попадёт в файл при скачивании",
    }
}

pub fn tools_cover_invalid(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "use JPEG, PNG, or WebP for cover",
        Lang::Ru => "обложка: JPEG, PNG или WebP",
    }
}

pub fn tools_cover_cleared(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "new cover cleared — original embedded art used again",
        Lang::Ru => "новая обложка сброшена — снова из исходного файла",
    }
}

pub fn tools_clear_cover(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Clear new cover",
        Lang::Ru => "Сбросить новую обложку",
    }
}

pub fn tools_cover_kept(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "embedded cover will be kept unless you replace it",
        Lang::Ru => "встроенная обложка сохранится, пока не заменишь",
    }
}

pub fn tools_no_cover(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "no embedded cover in source file",
        Lang::Ru => "в файле нет встроенной обложки",
    }
}

pub fn tools_roadmap_photo(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Photo: EXIF (location, camera, date) — next phase",
        Lang::Ru => "Фото: EXIF (место, камера, дата) — следующий этап",
    }
}

pub fn tools_roadmap_video(lang: Lang) -> &'static str {
    match lang {
        Lang::En => "Video: MP4 atoms — larger project, after photos",
        Lang::Ru => "Видео: атомы MP4 — крупнее, после фото",
    }
}

pub fn use_lang() -> RwSignal<Lang> {
    use_context::<RwSignal<Lang>>().expect("Lang context")
}
