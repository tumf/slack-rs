//! Internationalization messages for auth commands

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Japanese,
}

impl Language {
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "en" | "EN" => Some(Language::English),
            "ja" | "JA" => Some(Language::Japanese),
            _ => None,
        }
    }
}

/// Message catalog for export/import operations
pub struct Messages {
    lang: Language,
    messages: HashMap<&'static str, (&'static str, &'static str)>,
}

impl Messages {
    pub fn new(lang: Language) -> Self {
        let mut messages = HashMap::new();

        // Warning messages
        messages.insert(
            "warn.export_sensitive",
            (
                "WARNING: You are about to export sensitive authentication data.\n\
                 This file will contain access tokens that can be used to access your Slack workspaces.\n\
                 Store this file securely and delete it after use.",
                "警告: 機密認証情報をエクスポートしようとしています。\n\
                 このファイルには Slack ワークスペースへのアクセスに使用できるアクセストークンが含まれます。\n\
                 このファイルは安全に保管し、使用後は削除してください。"
            ),
        );

        // Prompt messages
        messages.insert(
            "prompt.passphrase",
            ("Enter passphrase: ", "パスフレーズを入力してください: "),
        );

        messages.insert(
            "prompt.passphrase_confirm",
            ("Confirm passphrase: ", "パスフレーズを再入力してください: "),
        );

        // Error messages
        messages.insert(
            "error.bad_permissions",
            (
                "Error: File must have 0600 permissions (owner read/write only)",
                "エラー: ファイルは 0600 パーミッション（所有者の読み書きのみ）である必要があります"
            ),
        );

        messages.insert(
            "error.passphrase_mismatch",
            (
                "Error: Passphrases do not match",
                "エラー: パスフレーズが一致しません",
            ),
        );

        messages.insert(
            "error.empty_passphrase",
            (
                "Error: Empty passphrase not allowed",
                "エラー: 空のパスフレーズは許可されていません",
            ),
        );

        messages.insert(
            "error.confirmation_required",
            (
                "Error: Export requires --yes flag for confirmation",
                "エラー: エクスポートには --yes フラグによる確認が必要です",
            ),
        );

        messages.insert(
            "error.profile_exists",
            (
                "Error: Profile already exists (use --force to overwrite)",
                "エラー: プロファイルがすでに存在します（上書きするには --force を使用してください）"
            ),
        );

        // Success messages
        messages.insert(
            "success.export",
            (
                "✓ Profiles exported successfully",
                "✓ プロファイルのエクスポートが完了しました",
            ),
        );

        messages.insert(
            "success.import",
            (
                "✓ Profiles imported successfully",
                "✓ プロファイルのインポートが完了しました",
            ),
        );

        // Info messages
        messages.insert(
            "info.export_count",
            (
                "Exporting {count} profile(s)",
                "{count} 件のプロファイルをエクスポート中",
            ),
        );

        messages.insert(
            "info.import_count",
            (
                "Importing {count} profile(s)",
                "{count} 件のプロファイルをインポート中",
            ),
        );

        Self { lang, messages }
    }

    pub fn get(&self, key: &str) -> &'static str {
        match self.messages.get(key) {
            Some(&(en, ja)) => match self.lang {
                Language::English => en,
                Language::Japanese => ja,
            },
            None => "",
        }
    }

    #[allow(dead_code)]
    pub fn format(&self, key: &str, replacements: &[(&str, &str)]) -> String {
        let template = self.get(key);
        let mut result = template.to_string();
        for (placeholder, value) in replacements {
            result = result.replace(&format!("{{{}}}", placeholder), value);
        }
        result
    }
}

impl Default for Messages {
    fn default() -> Self {
        // Default to Japanese based on locale, or English if not detected
        let lang = std::env::var("LANG")
            .ok()
            .map(|lang| {
                if lang.starts_with("ja") {
                    Language::Japanese
                } else {
                    Language::English
                }
            })
            .unwrap_or(Language::English);

        Self::new(lang)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_messages_english() {
        let messages = Messages::new(Language::English);
        assert!(messages.get("warn.export_sensitive").starts_with("WARNING"));
        assert!(messages.get("success.export").contains("exported"));
    }

    #[test]
    fn test_messages_japanese() {
        let messages = Messages::new(Language::Japanese);
        assert!(messages.get("warn.export_sensitive").starts_with("警告"));
        assert!(messages.get("success.export").contains("エクスポート"));
    }

    #[test]
    fn test_format_message() {
        let messages = Messages::new(Language::English);
        let formatted = messages.format("info.export_count", &[("count", "3")]);
        assert!(formatted.contains("3"));
        assert!(formatted.contains("profile"));
    }

    #[test]
    fn test_language_from_code() {
        assert_eq!(Language::from_code("en"), Some(Language::English));
        assert_eq!(Language::from_code("ja"), Some(Language::Japanese));
        assert_eq!(Language::from_code("EN"), Some(Language::English));
        assert_eq!(Language::from_code("JA"), Some(Language::Japanese));
        assert_eq!(Language::from_code("fr"), None);
    }
}
