//! ユーザー辞書モジュール。
//!
//! DLLと同じディレクトリにある `user_dict.json` を読み込み、
//! ユーザー定義の変換エントリを提供する。
//! 変換エンジンで処理する前にユーザー辞書を参照し、
//! 完全一致すれば辞書の値を使用する。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// ユーザー辞書JSONのルート構造。
#[derive(Serialize, Deserialize)]
struct UserDictJson {
    entries: HashMap<String, String>,
}

/// ユーザー辞書。
///
/// `HashMap<String, String>` ベースの完全一致検索辞書。
/// キー (ローマ字) に対する変換結果を返す。
pub struct UserDict {
    entries: HashMap<String, String>,
}

impl UserDict {
    /// JSONファイルからユーザー辞書を読み込む。
    ///
    /// ファイルが存在しない、またはパース失敗の場合は空の辞書を返す。
    pub fn load(path: &Path) -> Self {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Self::empty(),
        };

        let json: UserDictJson = match serde_json::from_str(&content) {
            Ok(j) => j,
            Err(_) => return Self::empty(),
        };

        Self {
            entries: json.entries,
        }
    }

    /// 空のユーザー辞書を作成する。
    pub fn empty() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// キーに一致するエントリを検索する (完全一致)。
    pub fn lookup(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_load_valid_dict() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("user_dict.json");
        let json = r#"{
            "entries": {
                "addr": "서울시 강남구",
                "name": "김철수",
                "email": "이메일 주소"
            }
        }"#;
        fs::write(&path, json).unwrap();

        let dict = UserDict::load(&path);
        assert_eq!(dict.lookup("addr"), Some("서울시 강남구"));
        assert_eq!(dict.lookup("name"), Some("김철수"));
        assert_eq!(dict.lookup("email"), Some("이메일 주소"));
        assert_eq!(dict.lookup("nonexistent"), None);
    }

    #[test]
    fn test_load_missing_file() {
        let path = Path::new("/tmp/nonexistent_user_dict.json");
        let dict = UserDict::load(path);
        assert_eq!(dict.lookup("anything"), None);
    }

    #[test]
    fn test_load_invalid_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("user_dict.json");
        fs::write(&path, "not json").unwrap();

        let dict = UserDict::load(&path);
        assert_eq!(dict.lookup("anything"), None);
    }

    #[test]
    fn test_empty_dict() {
        let dict = UserDict::empty();
        assert_eq!(dict.lookup("anything"), None);
    }

    #[test]
    fn test_empty_entries() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("user_dict.json");
        let json = r#"{ "entries": {} }"#;
        fs::write(&path, json).unwrap();

        let dict = UserDict::load(&path);
        assert_eq!(dict.lookup("anything"), None);
    }
}
