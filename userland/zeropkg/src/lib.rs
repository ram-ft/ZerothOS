//! zeropkg — The Official Package Manager & Runtime for Zeroth OS (.zero files)

pub struct ZeroPermissions {
    pub display: bool,
    pub filesystem: &'static str,
    pub network: bool,
    pub audio: bool,
}

pub struct ZeroManifest {
    pub id: &'static str,
    pub name: &'static str,
    pub version: &'static str,
    pub author: &'static str,
    pub entry: &'static str,
    pub permissions: ZeroPermissions,
}

/// Package Header magic bytes for all .zero files: "ZER0\x01"
pub const ZERO_MAGIC: &[u8; 5] = b"ZER0\x01";

/// Validates whether a given file is a genuine Zeroth OS application bundle
pub fn validate_zero_header(header_bytes: &[u8]) -> bool {
    header_bytes.len() >= 5 && &header_bytes[0..5] == ZERO_MAGIC
}
