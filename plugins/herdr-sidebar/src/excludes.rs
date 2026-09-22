//! User-managed VS Code-style Explorer and Search exclusions.
//!
//! This module is deliberately independent from the long-lived sidebar state
//! schema so it can evolve without conflicting with layout/settings changes.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    Global,
    Project,
}

impl Scope {
    pub fn label(self) -> &'static str {
        match self {
            Self::Global => "Global",
            Self::Project => "Project",
        }
    }

    pub fn other(self) -> Self {
        match self {
            Self::Global => Self::Project,
            Self::Project => Self::Global,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rules {
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub search: Vec<String>,
    #[serde(default)]
    pub use_ignore_files: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectiveRules {
    pub files: Vec<String>,
    pub search: Vec<String>,
    pub use_ignore_files: bool,
}

#[derive(Default, Serialize, Deserialize)]
struct File {
    #[serde(default)]
    global: Rules,
    #[serde(default)]
    projects: BTreeMap<String, Rules>,
}

fn path() -> Option<PathBuf> {
    Some(crate::state::plugin_state_dir()?.join("excludes.json"))
}

fn project_key(root: &Path) -> String {
    root.to_string_lossy().replace('\\', "/")
}

fn read(path: &Path) -> File {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|json| serde_json::from_str(json.trim_start_matches('\u{feff}')).ok())
        .unwrap_or_default()
}

fn combine(global: &Rules, project: Option<&Rules>) -> EffectiveRules {
    let merge = |base: &[String], extra: Option<&Vec<String>>| {
        let mut merged = base.to_vec();
        if let Some(extra) = extra {
            for pattern in extra {
                if !merged.contains(pattern) {
                    merged.push(pattern.clone());
                }
            }
        }
        merged
    };
    EffectiveRules {
        files: merge(&global.files, project.map(|rules| &rules.files)),
        search: merge(&global.search, project.map(|rules| &rules.search)),
        use_ignore_files: project
            .and_then(|rules| rules.use_ignore_files)
            .or(global.use_ignore_files)
            .unwrap_or(true),
    }
}

pub fn effective(root: &Path) -> EffectiveRules {
    let Some(path) = path() else {
        return combine(&Rules::default(), None);
    };
    let file = read(&path);
    combine(&file.global, file.projects.get(&project_key(root)))
}

pub fn scoped(root: &Path, scope: Scope) -> Rules {
    let Some(path) = path() else {
        return Rules::default();
    };
    let file = read(&path);
    match scope {
        Scope::Global => file.global,
        Scope::Project => file
            .projects
            .get(&project_key(root))
            .cloned()
            .unwrap_or_default(),
    }
}

pub fn update(root: &Path, scope: Scope, update: impl FnOnce(&mut Rules)) -> EffectiveRules {
    let Some(path) = path() else {
        let mut rules = Rules::default();
        update(&mut rules);
        return combine(&rules, None);
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut file = read(&path);
    match scope {
        Scope::Global => update(&mut file.global),
        Scope::Project => update(file.projects.entry(project_key(root)).or_default()),
    }
    if let Ok(json) = serde_json::to_string(&file) {
        let _ = std::fs::write(path, json);
    }
    combine(&file.global, file.projects.get(&project_key(root)))
}

/// Install the built-in example for a typical web application workspace.
pub fn apply_web_application_preset(root: &Path) -> EffectiveRules {
    update(root, Scope::Project, apply_web_application_rules)
}

fn apply_web_application_rules(rules: &mut Rules) {
    rules.files.clear();
    rules.search = vec![
        "**/.git".into(),
        "**/.next".into(),
        "**/build".into(),
        "**/node_modules".into(),
        "**/*.class".into(),
    ];
    rules.use_ignore_files = Some(false);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_application_preset_clears_file_excludes_and_sets_search_excludes() {
        let mut rules = Rules {
            files: vec!["**/dist".into()],
            search: vec!["**/generated".into()],
            use_ignore_files: Some(true),
        };

        apply_web_application_rules(&mut rules);

        assert!(rules.files.is_empty());
        assert_eq!(
            rules.search,
            [
                "**/.git",
                "**/.next",
                "**/build",
                "**/node_modules",
                "**/*.class"
            ]
        );
        assert_eq!(rules.use_ignore_files, Some(false));
    }
}
