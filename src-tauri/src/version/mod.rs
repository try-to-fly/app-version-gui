pub mod comparator;
pub mod parser;

pub use comparator::{compare_versions, has_update, is_prerelease, VersionComparison};
pub use parser::{clean_version_prefix, parse_version, ParsedVersion};
