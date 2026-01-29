//! 設定ファイル読み込み。
//!
//! DLLと同じディレクトリにある `chamsae.json` を読み込み、
//! IMEトグルキーなどの設定を取得する。
//! ファイルが存在しない場合はデフォルト設定で新規作成する。
//! パース失敗時はデフォルト値にフォールバックする。

use serde::{Deserialize, Serialize};
use std::path::Path;

/// トグルキー設定。
pub struct ToggleKey {
    /// 仮想キーコード。
    pub vk: u32,
    /// Shiftキー同時押し。
    pub shift: bool,
    /// Ctrlキー同時押し。
    pub ctrl: bool,
    /// Altキー同時押し。
    pub alt: bool,
}

/// 言語プロファイル設定。
pub struct Languages {
    /// 日本語キーボードプロファイルを登録するか。
    pub japanese: bool,
    /// 韓国語キーボードプロファイルを登録するか。
    pub korean: bool,
}

/// IME設定。
pub struct Config {
    pub toggle_key: ToggleKey,
    pub languages: Languages,
}

/// JSON設定ファイルのトグルキー定義。
#[derive(Serialize, Deserialize)]
struct ToggleKeyJson {
    key: String,
    shift: bool,
    ctrl: bool,
    alt: bool,
}

/// JSON設定ファイルの言語プロファイル定義。
#[derive(Serialize, Deserialize)]
struct LanguagesJson {
    japanese: bool,
    korean: bool,
}

impl Default for LanguagesJson {
    fn default() -> Self {
        Self {
            japanese: true,
            korean: false,
        }
    }
}

/// JSON設定ファイルのルート構造。
#[derive(Serialize, Deserialize)]
struct ConfigJson {
    toggle_key: ToggleKeyJson,
    #[serde(default)]
    languages: LanguagesJson,
}

/// キー名文字列を仮想キーコードに変換する。
///
/// 対応するキー名:
/// - `"A"`〜`"Z"`: アルファベットキー (0x41〜0x5A)
/// - `"0"`〜`"9"`: 数字キー (0x30〜0x39)
/// - `"Space"`: スペースキー (0x20)
fn key_name_to_vk(name: &str) -> Option<u32> {
    // アルファベットキー。
    if name.len() == 1 {
        let ch = name.as_bytes()[0];
        if ch.is_ascii_uppercase() {
            return Some(ch as u32);
        }
        if ch.is_ascii_digit() {
            return Some(ch as u32);
        }
    }

    // 名前付きキー。
    match name {
        "Space" => Some(0x20),
        _ => None,
    }
}

impl Default for ConfigJson {
    fn default() -> Self {
        Self {
            toggle_key: ToggleKeyJson {
                key: "Space".to_string(),
                shift: true,
                ctrl: false,
                alt: false,
            },
            languages: LanguagesJson {
                japanese: true,
                korean: false,
            },
        }
    }
}

impl Config {
    /// デフォルト設定を返す (Shift+Space、日本語有効、韓国語無効)。
    pub fn default() -> Self {
        Self {
            toggle_key: ToggleKey {
                vk: 0x20, // VK_SPACE
                shift: true,
                ctrl: false,
                alt: false,
            },
            languages: Languages {
                japanese: true,
                korean: false,
            },
        }
    }

    /// 設定ファイルを読み込む。
    ///
    /// `dll_dir` 内の `chamsae.json` を読み込む。
    /// ファイルが存在しない場合はデフォルト設定で新規作成する。
    /// パース失敗やキー名不正の場合はデフォルト値を返す。
    pub fn load(dll_dir: &Path) -> Self {
        let path = dll_dir.join("chamsae.json");

        if !path.exists() {
            // デフォルト設定ファイルを新規作成。
            let default_json = ConfigJson::default();
            if let Ok(content) = serde_json::to_string_pretty(&default_json) {
                let _ = std::fs::write(&path, content);
            }
            return Self::default();
        }

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };

        let json: ConfigJson = match serde_json::from_str(&content) {
            Ok(j) => j,
            Err(_) => return Self::default(),
        };

        let vk = match key_name_to_vk(&json.toggle_key.key) {
            Some(v) => v,
            None => return Self::default(),
        };

        Self {
            toggle_key: ToggleKey {
                vk,
                shift: json.toggle_key.shift,
                ctrl: json.toggle_key.ctrl,
                alt: json.toggle_key.alt,
            },
            languages: Languages {
                japanese: json.languages.japanese,
                korean: json.languages.korean,
            },
        }
    }

    /// DLLのディレクトリパスを取得して設定を読み込む。
    #[cfg(windows)]
    pub fn load_from_dll() -> Self {
        match get_dll_directory() {
            Some(dir) => Self::load(&dir),
            None => Self::default(),
        }
    }
}

/// DLLモジュールのディレクトリパスを取得する。
#[cfg(windows)]
fn get_dll_directory() -> Option<std::path::PathBuf> {
    use crate::com::dll_module;

    let hmodule = dll_module::get_module_handle();
    if hmodule.0.is_null() {
        return None;
    }

    let mut buf = [0u16; 260]; // MAX_PATH
    let len = unsafe {
        windows::Win32::System::LibraryLoader::GetModuleFileNameW(
            hmodule,
            &mut buf,
        )
    };

    if len == 0 {
        return None;
    }

    let path = String::from_utf16_lossy(&buf[..len as usize]);
    let path = std::path::PathBuf::from(path);
    path.parent().map(|p| p.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.toggle_key.vk, 0x20);
        assert!(config.toggle_key.shift);
        assert!(!config.toggle_key.ctrl);
        assert!(!config.toggle_key.alt);
        assert!(config.languages.japanese);
        assert!(!config.languages.korean);
    }

    #[test]
    fn test_key_name_to_vk_alphabet() {
        assert_eq!(key_name_to_vk("A"), Some(0x41));
        assert_eq!(key_name_to_vk("Z"), Some(0x5A));
        assert_eq!(key_name_to_vk("S"), Some(0x53));
    }

    #[test]
    fn test_key_name_to_vk_digits() {
        assert_eq!(key_name_to_vk("0"), Some(0x30));
        assert_eq!(key_name_to_vk("9"), Some(0x39));
    }

    #[test]
    fn test_key_name_to_vk_space() {
        assert_eq!(key_name_to_vk("Space"), Some(0x20));
    }

    #[test]
    fn test_key_name_to_vk_invalid() {
        assert_eq!(key_name_to_vk(""), None);
        assert_eq!(key_name_to_vk("a"), None);
        assert_eq!(key_name_to_vk("Enter"), None);
        assert_eq!(key_name_to_vk("unknown"), None);
    }

    #[test]
    fn test_load_valid_json() {
        let dir = tempfile::tempdir().unwrap();
        let json = r#"{
            "toggle_key": {
                "key": "S",
                "shift": false,
                "ctrl": false,
                "alt": true
            },
            "languages": {
                "japanese": false,
                "korean": true
            }
        }"#;
        fs::write(dir.path().join("chamsae.json"), json).unwrap();

        let config = Config::load(dir.path());
        assert_eq!(config.toggle_key.vk, 0x53); // 'S'
        assert!(!config.toggle_key.shift);
        assert!(!config.toggle_key.ctrl);
        assert!(config.toggle_key.alt);
        assert!(!config.languages.japanese);
        assert!(config.languages.korean);
    }

    #[test]
    fn test_load_json_without_languages_uses_default() {
        let dir = tempfile::tempdir().unwrap();
        let json = r#"{
            "toggle_key": {
                "key": "Space",
                "shift": true,
                "ctrl": false,
                "alt": false
            }
        }"#;
        fs::write(dir.path().join("chamsae.json"), json).unwrap();

        let config = Config::load(dir.path());
        assert!(config.languages.japanese);
        assert!(!config.languages.korean);
    }

    #[test]
    fn test_load_missing_file_creates_default() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config::load(dir.path());

        // デフォルト値が返される。
        assert_eq!(config.toggle_key.vk, 0x20);
        assert!(config.toggle_key.shift);
        assert!(config.languages.japanese);
        assert!(!config.languages.korean);

        // ファイルが新規作成されている。
        let path = dir.path().join("chamsae.json");
        assert!(path.exists());

        let content = fs::read_to_string(&path).unwrap();
        let json: ConfigJson = serde_json::from_str(&content).unwrap();
        assert_eq!(json.toggle_key.key, "Space");
        assert!(json.toggle_key.shift);
        assert!(json.languages.japanese);
        assert!(!json.languages.korean);
    }

    #[test]
    fn test_load_invalid_json_fallback() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("chamsae.json"), "not json").unwrap();

        let config = Config::load(dir.path());
        assert_eq!(config.toggle_key.vk, 0x20);
        assert!(config.toggle_key.shift);
    }

    #[test]
    fn test_load_invalid_key_name_fallback() {
        let dir = tempfile::tempdir().unwrap();
        let json = r#"{
            "toggle_key": {
                "key": "InvalidKey",
                "shift": false,
                "ctrl": false,
                "alt": true
            }
        }"#;
        fs::write(dir.path().join("chamsae.json"), json).unwrap();

        let config = Config::load(dir.path());
        assert_eq!(config.toggle_key.vk, 0x20);
        assert!(config.toggle_key.shift);
    }
}
