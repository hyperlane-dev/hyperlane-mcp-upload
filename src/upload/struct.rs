use super::*;

/// Final result of a successful ltpp.vip upload.
#[derive(Clone, Debug, Serialize)]
pub struct UploadResult {
    /// Absolute URL pointing to the uploaded file on ltpp.vip.
    pub url: String,
    /// Original file name uploaded to ltpp.vip.
    pub file_name: String,
    /// Size of the uploaded file in bytes.
    pub size: u64,
}

/// Configuration for the ltpp.vip upload client.
#[derive(Clone, Debug)]
pub struct UploadConfig {
    /// Hostname of the ltpp.vip service.
    pub host: String,
    /// Base path of the upload REST API.
    pub base_path: String,
    /// Size in bytes of each chunk sent to the save endpoint.
    pub chunk_size: usize,
}

impl Default for UploadConfig {
    /// Returns the default upload config pointing at `https://ltpp.vip/api/upload`
    /// with 5 MiB chunks.
    fn default() -> Self {
        let host: String = LTPP_HOST.to_owned();
        let base_path: String = LTPP_BASE_PATH.to_owned();
        let chunk_size: usize = DEFAULT_CHUNK_SIZE;
        Self {
            host,
            base_path,
            chunk_size,
        }
    }
}
