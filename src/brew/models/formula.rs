pub enum MacOSVersion {
    Ventura,
    BigSur,
    ElCapitan,
    Sierra,
    HighSierra,
    Mojave,
    SemVer(String),
}
impl std::fmt::Display for MacOSVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MacOSVersion::Ventura => write!(f, ":ventura"),
            MacOSVersion::BigSur => write!(f, ":big_sur"),
            MacOSVersion::ElCapitan => write!(f, ":el_capitan"),
            MacOSVersion::Sierra => write!(f, ":sierra"),
            MacOSVersion::HighSierra => write!(f, ":high_sierra"),
            MacOSVersion::Mojave => write!(f, ":mojave"),
            MacOSVersion::SemVer(version) => write!(f, "\"{}\"", version),
        }
    }
}
pub enum DependsOn {
    MacOS(MacOSVersion),
    Formula(String),
    Cask(String),
}
impl std::fmt::Display for DependsOn {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DependsOn::MacOS(version) => write!(f, "  depends_on macos: {}", version),
            DependsOn::Formula(name) => write!(f, "  depends_on formula: \"{}\"", name),
            DependsOn::Cask(name) => write!(f, "  depends_on cask: \"{}\"", name),
        }
    }
}
pub struct Head {
    uri: String,
    branch: String,
}
pub struct Formula {
    body: String,
    depends_on: DependsOn,
    desc: String,
    head: Head,
    homepage: String,
    license: Vec<String>,
}
