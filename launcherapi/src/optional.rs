use crate::validation::{ClientInfo, OsType};
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Action {
    Files(FileAction),
    Args(Vec<String>),
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct OptionalFiles {
    #[serde(default)]
    pub original_paths: Vec<String>,
    #[serde(default)]
    pub rename_paths: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FileAction {
    location: Location,
    files: OptionalFiles,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum Location {
    Profile,
    Libraries,
    Assets,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Rule {
    OsType(OsRule),
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OsRule {
    os_type: OsType,
    #[serde(default)]
    compare_mode: CompareMode,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum CompareMode {
    Equal,
    Unequal,
}

impl Default for CompareMode {
    fn default() -> Self {
        CompareMode::Equal
    }
}

pub trait Apply {
    fn apply(&self, client_info: &ClientInfo) -> bool;
}

impl Apply for OsRule {
    fn apply(&self, client_info: &ClientInfo) -> bool {
        match self.compare_mode {
            CompareMode::Equal => self.os_type == client_info.os_type,
            CompareMode::Unequal => self.os_type != client_info.os_type,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Optional {
    actions: Vec<Action>,
    rules: Vec<Rule>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_visible")]
    pub visible: bool,
    pub description: Option<String>,
    pub name: Option<String>,
}

fn default_visible() -> bool {
    false
}

fn default_enabled() -> bool {
    true
}

impl Optional {
    pub fn get_args(&self) -> Vec<String> {
        self.actions
            .iter()
            .filter_map(|action| match action {
                Action::Args(args) => Some(args.to_vec()),
                _ => None,
            })
            .flatten()
            .collect::<Vec<_>>()
    }

    pub fn visible(&self, client_info: &ClientInfo) -> bool {
        self.visible && self.apply(client_info)
    }

    pub fn relevant(&self, client_info: &ClientInfo, selected: &Vec<String>) -> bool {
        self.apply(client_info) && (!self.visible || selected.contains(self.name.as_ref().unwrap()))
    }

    pub fn get_files(&self) -> HashMap<&Location, OptionalFiles> {
        let mut map = HashMap::new();
        for action in self.actions.iter().filter_map(|action| match action {
            Action::Files(files) => Some(files),
            _ => None,
        }) {
            let entry = map
                .entry(&action.location)
                .or_insert(OptionalFiles::default());
            entry
                .original_paths
                .append(&mut action.files.original_paths.clone());
            entry.rename_paths.extend(action.files.rename_paths.clone());
        }
        map
    }
}

impl Apply for Optional {
    fn apply(&self, client_info: &ClientInfo) -> bool {
        self.rules.iter().any(|rule| match rule {
            Rule::OsType(rule) => rule.apply(&client_info),
        })
    }
}