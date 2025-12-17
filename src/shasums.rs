#[derive(Debug, Clone)]
pub struct ShasumsText(String);

const PREFERRED_ARCHIVE_EXTENSION: &str = ".tar.gz";

impl From<String> for ShasumsText {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl ShasumsText {
    pub fn entries<'a>(&'a self) -> impl Iterator<Item = ShasumsTextEntry<'a>> {
        self.0
            .split('\n')
            .into_iter()
            .filter_map(|line| {
                let mut parts = line.split_whitespace();
                let checksum = parts.next()?;
                let filepath = parts.next()?;
                let (filestem, _) = filepath.rsplit_once(PREFERRED_ARCHIVE_EXTENSION)?;
                let mut target_parts = filestem.rsplitn(3, '-');
                let arch = target_parts.next()?;
                let system = target_parts.next()?;
                let target = Target::from_system_arch(system, arch)?;

                Some(ShasumsTextEntry {
                    filepath,
                    target,
                    checksum,
                })
            })
            .into_iter()
    }
}

pub struct ShasumsTextEntry<'a> {
    pub filepath: &'a str,
    pub target: Target,
    pub checksum: &'a str,
}

pub enum Target {
    DarwinArm64,
    DarwinX64,
    LinuxArm64,
    LinuxX64,
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DarwinArm64 => write!(f, "aarch64-darwin"),
            Self::DarwinX64 => write!(f, "x86_64-darwin"),
            Self::LinuxArm64 => write!(f, "aarch64-linux"),
            Self::LinuxX64 => write!(f, "x86_64-linux"),
        }
    }
}

impl Target {
    fn from_system_arch(system: &str, arch: &str) -> Option<Self> {
        match (system, arch) {
            ("linux", "x64") => Some(Self::LinuxX64),
            ("linux", "arm64") => Some(Self::LinuxArm64),
            ("darwin", "x64") => Some(Self::DarwinX64),
            ("darwin", "arm64") => Some(Self::DarwinArm64),
            _ => None,
        }
    }
}
