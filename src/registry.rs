use crate::backend::backend_type::BackendType;
use crate::cli::args::BackendArg;
use crate::config::SETTINGS;
use once_cell::sync::Lazy;
use std::collections::{BTreeMap, HashSet};
use std::env::consts::OS;
use std::iter::Iterator;
use strum::IntoEnumIterator;
use url::Url;

// the registry is generated from registry.toml in the project root
include!(concat!(env!("OUT_DIR"), "/registry.rs"));

#[derive(Debug, Clone)]
pub struct RegistryTool {
    #[allow(unused)]
    pub short: &'static str,
    pub backends: Vec<&'static str>,
    #[allow(unused)]
    pub aliases: &'static [&'static str],
    pub test: &'static Option<(&'static str, &'static str)>,
    pub os: &'static [&'static str],
}

impl RegistryTool {
    pub fn backends(&self) -> Vec<&'static str> {
        static BACKEND_TYPES: Lazy<HashSet<String>> = Lazy::new(|| {
            let mut backend_types = BackendType::iter()
                .map(|b| b.to_string())
                .collect::<HashSet<_>>();
            for backend in &SETTINGS.disable_backends {
                backend_types.remove(backend);
            }
            if cfg!(windows) {
                backend_types.remove("asdf");
            }
            if cfg!(unix) && !SETTINGS.experimental {
                backend_types.remove("aqua");
            }
            backend_types
        });
        self.backends
            .iter()
            .filter(|full| {
                full.split(':')
                    .next()
                    .map_or(false, |b| BACKEND_TYPES.contains(b))
            })
            .copied()
            .collect()
    }

    pub fn is_supported_os(&self) -> bool {
        self.os.is_empty() || self.os.contains(&OS)
    }

    pub fn ba(&self) -> Option<BackendArg> {
        self.backends()
            .first()
            .map(|f| BackendArg::new(self.short.to_string(), Some(f.to_string())))
    }
}

pub fn is_trusted_plugin(name: &str, remote: &str) -> bool {
    let normalized_url = normalize_remote(remote).unwrap_or("INVALID_URL".into());
    let is_shorthand = REGISTRY
        .get(name)
        .and_then(|tool| tool.backends().first().copied())
        .map(full_to_url)
        .is_some_and(|s| normalize_remote(&s).unwrap_or_default() == normalized_url);
    let is_mise_url = normalized_url.starts_with("github.com/mise-plugins/");

    !is_shorthand || is_mise_url
}

fn normalize_remote(remote: &str) -> eyre::Result<String> {
    let url = Url::parse(remote)?;
    let host = url.host_str().unwrap();
    let path = url.path().trim_end_matches(".git");
    Ok(format!("{host}{path}"))
}

pub fn full_to_url(full: &str) -> String {
    let (_backend, url) = full.split_once(':').unwrap_or(("", full));
    if url.starts_with("https://") {
        url.to_string()
    } else {
        format!("https://github.com/{url}.git")
    }
}
