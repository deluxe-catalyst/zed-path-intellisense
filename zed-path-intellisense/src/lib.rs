use zed_extension_api as zed;

struct PathIntellisenseExtension;

impl zed::Extension for PathIntellisenseExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        let server_path = worktree
            .which("zed-path-intellisense-lsp")
            .ok_or_else(|| {
                "zed-path-intellisense-lsp not found. Build it with: \
                 cargo build --release --manifest-path zed-path-intellisense-lsp/Cargo.toml"
                    .to_string()
            })?;

        Ok(zed::Command {
            command: server_path,
            args: vec![],
            env: Default::default(),
        })
    }

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> zed::Result<Option<serde_json::Value>> {
        Ok(None)
    }
}

zed::register_extension!(PathIntellisenseExtension);
