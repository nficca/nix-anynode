use std::collections::HashMap;

use askama::Template;

use crate::shasums::Target;

#[derive(Template, Default, Debug, Clone)]
#[template(path = "data.nix.template", escape = "none")]
pub struct DataNixTemplate {
    versions: Vec<VersionData>,
}

impl Extend<VersionData> for DataNixTemplate {
    fn extend<T: IntoIterator<Item = VersionData>>(&mut self, iter: T) {
        self.versions.extend(iter);
    }
}

#[derive(Debug, Clone)]
pub struct VersionData {
    pub directory: String,
    pub system_packages: HashMap<System, PackageData>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum System {
    DarwinArm64,
    DarwinX64,
    LinuxArm64,
    LinuxX64,
}

impl std::fmt::Display for System {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DarwinArm64 => write!(f, "aarch64-darwin"),
            Self::DarwinX64 => write!(f, "x86_64-darwin"),
            Self::LinuxArm64 => write!(f, "aarch64-linux"),
            Self::LinuxX64 => write!(f, "x86_64-linux"),
        }
    }
}

impl From<Target> for System {
    fn from(value: Target) -> Self {
        match value {
            Target::DarwinArm64 => Self::DarwinArm64,
            Target::DarwinX64 => Self::DarwinX64,
            Target::LinuxArm64 => Self::LinuxArm64,
            Target::LinuxX64 => Self::LinuxX64,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PackageData {
    url: String,
    sha256: String,
}

impl PackageData {
    pub fn new(url: &str, sha256: &str) -> Self {
        Self {
            url: url.to_string(),
            sha256: sha256.to_string(),
        }
    }
}
