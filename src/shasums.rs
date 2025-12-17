const PREFERRED_ARCHIVE_EXTENSION: &str = ".tar.gz";

/// Simple wrapper around the contents of a SHASUMS256.txt file.
///
/// Should contain something like this:
/// ```not_rust
/// d86280574658364f8acea579c430b65fe4ab71138039904739df830943ca4859  node-v4.9.1-darwin-x64.tar.gz
/// 36b3ae387c242f609d91ac78c8804846af568a14e5b733cda633382a1d284d8d  node-v4.9.1-darwin-x64.tar.xz
/// d5fe1c36f64fe7548060baef58ebe9e47cf281868108753dbaf97413edf6004f  node-v4.9.1-headers.tar.gz
/// 2b071b6d6bbe8c323fafd40a0c5111b331a49522e439a2761de9f1bf8d2ac188  node-v4.9.1-headers.tar.xz
/// b61b9b19f584cdd198a7342966a269393e6ef79e1273e4f4940d872b929d8403  node-v4.9.1-linux-arm64.tar.gz
/// 7203f9693e06ad220cfde2e5d70778cb021a176ecab1bdfee0bf546363333f26  node-v4.9.1-linux-arm64.tar.xz
/// 151bfd51cdb18e2404c83ab9e2c2775262fc8fcfe823cca3ce2a707de49e81ea  node-v4.9.1-linux-armv6l.tar.gz
/// a56808a59f132f134919e72961bc568226f47b8c752b8fd4ba06e66d07788e84  node-v4.9.1-linux-armv6l.tar.xz
/// ...(and more)
/// ```
#[derive(Debug, Clone)]
pub struct ShasumsText(String);

impl From<String> for ShasumsText {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl ShasumsText {
    /// Iterates over the lines in the shasums text file, yielding
    /// [`ShasumsTextEntry`] values. This will only include lines that:
    ///   - actually have a checksum and a filename
    ///   - the filename is for a tarball (.tar.gz)
    ///   - the filename includes a target system that can be parsed into [`Target`]
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

/// A reference to a relevant entry in a [`ShasumsText`] file.
pub struct ShasumsTextEntry<'a> {
    pub filepath: &'a str,
    pub target: Target,
    pub checksum: &'a str,
}

/// The target of a distributed nodejs binary.
///
/// This does not include all of the possible targets that nodejs is
/// built for. It only includes the common Linux and MacOS targets but could
/// be expanded later.
pub enum Target {
    DarwinArm64,
    DarwinX64,
    LinuxArm64,
    LinuxX64,
}

impl Target {
    /// Parses the target from the system and arch string pair.
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
