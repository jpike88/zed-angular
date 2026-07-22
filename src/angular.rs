use serde::Deserialize;
use std::path::PathBuf;
use std::{env, fs, vec};
use zed::lsp::{Completion, CompletionKind};
use zed::settings::LspSettings;
use zed::CodeLabelSpan;
use zed_extension_api::{self as zed, serde_json, Result};

const SERVER_PATH: &str = "node_modules/@angular/language-server/index.js";
const TYPESCRIPT_TSDK_PATH: &str = "node_modules/typescript/lib";

const ANGULAR_LANGUAGE_SERVER_PACKAGE_NAME: &str = "@angular/language-server";
const TYPESCRIPT_PACKAGE_NAME: &str = "typescript";

struct AngularExtension {
    did_find_server: bool,
}

impl AngularExtension {
    #[allow(dead_code)]
    pub const LANGUAGE_SERVER_ID: &'static str = "angular";

    fn file_exists_at_path(&self, path: &str) -> bool {
        fs::metadata(path).map_or(false, |stat| stat.is_file())
    }

    fn server_script_path(&mut self, language_server_id: &zed::LanguageServerId) -> Result<String> {
        let server_exists = self.file_exists_at_path(SERVER_PATH);

        if self.did_find_server && server_exists {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::CheckingForUpdate,
            );
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::Downloading,
        );

        if !self.file_exists_at_path(SERVER_PATH) {
            return Err(format!(
                "Expected Angular language server path '{}' was not found in the project. Please ensure '{}' is installed in your project's node_modules.",
                SERVER_PATH, ANGULAR_LANGUAGE_SERVER_PACKAGE_NAME
            )
            .into());
        }

        self.did_find_server = true;
        Ok(SERVER_PATH.to_string())
    }

    fn get_current_dir() -> Result<PathBuf> {
        env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))
    }

    fn get_ng_probe_locations(worktree: &zed::Worktree) -> Vec<String> {
        let mut paths = vec![];

        // 1. Probe the open project's root folder (where the user's local node_modules lives)
        paths.push(worktree.root_path());

        // 2. Probe the project's sub node_modules directory explicitly
        let project_node_modules = PathBuf::from(worktree.root_path()).join("node_modules");
        paths.push(project_node_modules.to_string_lossy().to_string());

        // 3. Probe the Zed extension's own node_modules directory as a fallback
        if let Ok(current_dir) = Self::get_current_dir() {
            let ext_node_modules = current_dir.join("node_modules");
            paths.push(ext_node_modules.to_string_lossy().to_string());
            paths.push(current_dir.to_string_lossy().to_string());
        }

        paths
    }

    fn get_ts_probe_locations(worktree: &zed::Worktree) -> Vec<String> {
        // Use the exact same resolution rules for TypeScript probing
        Self::get_ng_probe_locations(worktree)
    }
}

impl zed::Extension for AngularExtension {
    fn new() -> Self {
        Self {
            did_find_server: false,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let server_path = self.server_script_path(language_server_id)?;
        let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::new());
        let full_path_to_server = current_dir.join(&server_path);

        let mut args = vec![full_path_to_server.to_string_lossy().to_string()];
        args.push("--stdio".to_string());

        // Probe paths: This tells the language-server where to seek and resolve "typescript/lib/tsserverlibrary"
        args.push("--tsProbeLocations".to_string());
        args.push(Self::get_ts_probe_locations(worktree).join(","));

        args.push("--ngProbeLocations".to_string());
        args.push(Self::get_ng_probe_locations(worktree).join(","));

        // Provide the SDK path inside the project's folder hierarchy
        let absolute_tsdk_path = current_dir.join(TYPESCRIPT_TSDK_PATH);
        args.push("--tsdk".to_string());
        args.push(absolute_tsdk_path.to_string_lossy().to_string());

        Ok(zed::Command {
            command: zed::node_binary_path()?,
            args,
            env: Default::default(),
        })
    }

    fn label_for_completion(
        &self,
        _language_server_id: &zed::LanguageServerId,
        completion: Completion,
    ) -> Option<zed::CodeLabel> {
        println!("Label for completion {:?}", completion.kind);
        let highlight_name = match completion.kind? {
            CompletionKind::Class | CompletionKind::Interface => "type",
            CompletionKind::Constructor => "constructor",
            CompletionKind::Constant => "constant",
            CompletionKind::Function | CompletionKind::Method => "function",
            CompletionKind::Property | CompletionKind::Field => "property",
            CompletionKind::Variable => "variable",
            CompletionKind::Keyword => "keyword",
            CompletionKind::Enum => "enum",
            CompletionKind::Module => "module",
            _ => return None,
        };

        let len = completion.label.len();
        let name_span = CodeLabelSpan::literal(completion.label, Some(highlight_name.to_string()));

        let spans = if let Some(detail) = completion.detail {
            vec![
                name_span,
                CodeLabelSpan::literal(" ", None),
                CodeLabelSpan::literal(detail, Some("detail".to_string())),
            ]
        } else {
            vec![name_span]
        };

        Some(zed::CodeLabel {
            code: Default::default(),
            spans,
            filter_range: (0..len).into(),
        })
    }
}

zed::register_extension!(AngularExtension);
