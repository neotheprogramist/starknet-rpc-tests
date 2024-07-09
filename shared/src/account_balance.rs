use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub enum Version {
    V0_0_6,
    V0_0_5,
}

impl std::str::FromStr for Version {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "v6" => Ok(Version::V0_0_6),
            "v5" => Ok(Version::V0_0_5),
            _ => Err(format!("'{}' is not a valid version", s)),
        }
    }
}
