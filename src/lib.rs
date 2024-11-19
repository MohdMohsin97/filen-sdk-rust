mod error;
mod crypto;
mod api;
mod fs;

use api::API;
use crypto::Crypto;
use fs::FS;
pub use self::error::{Error, Result};

enum AuthVersion {
    V1,
    V2
} 

struct FilenSDKConfig {
    password: Option<String>,
    two_factor_code: Option<String>,
    master_key: Option<Vec<String>>,
    api_key: Option<String>,
    public_key: Option<String>,
    private_key: Option<String>,
    auth_version: AuthVersion,
    base_folder_uuid: Option<String>,
    user_id: Option<i32>,
    metedata_cache: Option<String>,
    tmp_path: Option<String>,
    connect_to_socket: bool
}

struct FilenSDK {
    pub config: FilenSDKConfig,
    _api: API,
    _crypto: Crypto,
    _fs: FS,
}