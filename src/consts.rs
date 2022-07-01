use const_format::concatcp;

pub const SETTING_AUTO_REFRESH: &str = concat!(env!("CARGO_PKG_NAME"), "_setting_auto_refresh");

pub const SPOTIFY_STORAGE: &str = concat!(env!("CARGO_PKG_NAME"), "_spotify_auth");
pub const SPOTIFY_STATE_STORAGE: &str = concatcp!(SPOTIFY_STORAGE, "_state");

pub const SPOTIFY_CLIENT_ID: &str = "be6201c1e3154c51b50ffb302e770db5";
