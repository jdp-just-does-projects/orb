use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaKind {
    Image,
    Raw,
    Video,
    /// Apple Live Photo style image+video pair.
    Live,
}

impl MediaKind {
    /// Classify by lowercase file extension. Returns `None` for non-media.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "jpg" | "jpeg" | "jpe" | "png" | "gif" | "webp" | "bmp" | "tif" | "tiff" | "heic"
            | "heif" | "avif" | "jxl" | "psd" => Some(Self::Image),
            "cr2" | "cr3" | "nef" | "nrw" | "arw" | "srf" | "sr2" | "dng" | "raf" | "orf"
            | "rw2" | "pef" | "erf" | "3fr" | "iiq" | "mrw" | "x3f" | "rwl" => Some(Self::Raw),
            "mp4" | "m4v" | "mov" | "mkv" | "webm" | "avi" | "mts" | "m2ts" | "ts" | "mpg"
            | "mpeg" | "wmv" | "flv" | "3gp" | "mxf" => Some(Self::Video),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Raw => "raw",
            Self::Video => "video",
            Self::Live => "live",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_extensions() {
        assert_eq!(MediaKind::from_extension("JPG"), Some(MediaKind::Image));
        assert_eq!(MediaKind::from_extension("cr3"), Some(MediaKind::Raw));
        assert_eq!(MediaKind::from_extension("MOV"), Some(MediaKind::Video));
        assert_eq!(MediaKind::from_extension("txt"), None);
    }
}
