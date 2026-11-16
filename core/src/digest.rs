//! The schemes: named rules that each compute one kind of digest. A digest is always written
//! with its scheme, so two digests of different schemes never compare equal.

use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;

pub const TEXT_SCHEME: &str = "text-1";
pub const FILE_SCHEME: &str = "file-1";
pub const PARTS_SCHEME: &str = "parts-1";

/// The schemes that produce a fingerprint, sorted by name. `record-1` is not among them, because
/// it says how a record is stored, not how a result was measured.
pub const FINGERPRINT_SCHEMES: [&str; 3] = [FILE_SCHEME, PARTS_SCHEME, TEXT_SCHEME];

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Model text as `text-1` folds it: NFC, every line ending as LF, spaces and tabs at line ends
/// removed. Nothing else is folded, so a re-wrapped or restyled text gets another digest.
pub fn normalize_text(text: &str) -> String {
    let composed: String = text.nfc().collect();
    let unified = composed.replace("\r\n", "\n").replace('\r', "\n");
    unified
        .split('\n')
        .map(|line| line.trim_end_matches([' ', '\t']))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn text_digest(text: &str) -> String {
    format!("{TEXT_SCHEME}:{}", sha256_hex(normalize_text(text).as_bytes()))
}

/// `file-1` is git's own object id, so the hash is the repository's and not Azimuth's.
pub fn file_digest(object_id: &str) -> String {
    format!("{FILE_SCHEME}:{object_id}")
}

pub fn parts_digest(bytes: &[u8]) -> String {
    format!("{PARTS_SCHEME}:{}", sha256_hex(bytes))
}

/// The scheme a written digest names: the text before the first colon.
pub fn scheme_of(digest: &str) -> Option<&str> {
    digest.split_once(':').map(|(scheme, _)| scheme)
}

/// The word of a scheme name without its number, so `parts-2` is recognised as a later `parts`.
pub fn scheme_word(scheme: &str) -> &str {
    scheme.rsplit_once('-').map(|(word, _)| word).unwrap_or(scheme)
}
