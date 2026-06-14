//! License type: marker enum for content provenance and attribution requirements.
//!
//! Every piece of content carries a license that determines:
//! - Attribution rendering (always mandatory, non-strippable)
//! - Reuse boundaries (per jurisdiction/Option A vs B)
//! - Derivative-work permissions
//!
//! Parse-Don't-Validate: only officially-assigned licenses can be constructed.
//! Exhaustive match is compiler-enforced; a catch-all arm is impossible.

use std::fmt;

/// A license marker for content provenance.
///
/// Each license variant indicates the source and legal constraints on content:
/// - `CcBySa4`: Stack Exchange and other CC BY-SA 4.0 sources (share-alike)
/// - `CcBy4`: Biostars and other CC BY 4.0 sources (attribution-only)
/// - `Native`: Content created within medstack (original user contributions)
/// - `LinkOnly`: FHIR Zulip and other sources where only links are mirrored (no copy)
///
/// Parse-Don't-Validate: invalid/unknown licenses cannot be constructed.
/// The enum is exhaustive; the compiler rejects any catch-all patterns.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum License {
    /// CC BY-SA 4.0: source-share-alike, requires attribution.
    CcBySa4,
    /// CC BY 4.0: attribution-only, no share-alike.
    CcBy4,
    /// Native: original content created within medstack.
    Native,
    /// Link-only: title + URL, no body copy mirrored.
    LinkOnly,
}

impl License {
    /// Parse a license from a string, case-insensitive.
    ///
    /// Accepts the canonical forms: "cc-by-sa-4.0", "cc-by-4.0", "native", "link-only"
    /// (hyphens, case-insensitive).
    ///
    /// # Errors
    ///
    /// Returns `Err(LicenseError::Unknown)` if the string is not a recognized
    /// license code.
    pub fn new(s: &str) -> Result<Self, LicenseError> {
        match s.trim().to_lowercase().as_str() {
            "cc-by-sa-4.0" | "cc by-sa 4.0" | "ccbysa4" => Ok(License::CcBySa4),
            "cc-by-4.0" | "cc by 4.0" | "ccby4" => Ok(License::CcBy4),
            "native" => Ok(License::Native),
            "link-only" | "linkonly" | "link only" => Ok(License::LinkOnly),
            _ => Err(LicenseError::Unknown),
        }
    }
}

impl fmt::Display for License {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CcBySa4 => write!(f, "CC BY-SA 4.0"),
            Self::CcBy4 => write!(f, "CC BY 4.0"),
            Self::Native => write!(f, "Native"),
            Self::LinkOnly => write!(f, "Link-only"),
        }
    }
}

/// Error when parsing a License.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LicenseError {
    /// The string is not a recognized license code.
    Unknown,
}

impl fmt::Display for LicenseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown => write!(
                f,
                "unknown license; expected one of: cc-by-sa-4.0, cc-by-4.0, native, link-only"
            ),
        }
    }
}

impl std::error::Error for LicenseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ccbysa4() {
        assert_eq!(License::new("cc-by-sa-4.0").unwrap(), License::CcBySa4);
        assert_eq!(License::new("CC-BY-SA-4.0").unwrap(), License::CcBySa4);
        assert_eq!(License::new("cc by-sa 4.0").unwrap(), License::CcBySa4);
        assert_eq!(License::new("CCBYSA4").unwrap(), License::CcBySa4);
    }

    #[test]
    fn parse_ccby4() {
        assert_eq!(License::new("cc-by-4.0").unwrap(), License::CcBy4);
        assert_eq!(License::new("CC-BY-4.0").unwrap(), License::CcBy4);
        assert_eq!(License::new("cc by 4.0").unwrap(), License::CcBy4);
        assert_eq!(License::new("CCBY4").unwrap(), License::CcBy4);
    }

    #[test]
    fn parse_native() {
        assert_eq!(License::new("native").unwrap(), License::Native);
        assert_eq!(License::new("NATIVE").unwrap(), License::Native);
        assert_eq!(License::new("  native  ").unwrap(), License::Native);
    }

    #[test]
    fn parse_linkonly() {
        assert_eq!(License::new("link-only").unwrap(), License::LinkOnly);
        assert_eq!(License::new("LINK-ONLY").unwrap(), License::LinkOnly);
        assert_eq!(License::new("linkonly").unwrap(), License::LinkOnly);
        assert_eq!(License::new("link only").unwrap(), License::LinkOnly);
    }

    #[test]
    fn parse_unknown() {
        assert_eq!(License::new("unknown").unwrap_err(), LicenseError::Unknown);
        assert_eq!(License::new("cc0").unwrap_err(), LicenseError::Unknown);
        assert_eq!(
            License::new("apache-2.0").unwrap_err(),
            LicenseError::Unknown
        );
        assert_eq!(License::new("").unwrap_err(), LicenseError::Unknown);
    }

    #[test]
    fn display() {
        assert_eq!(format!("{}", License::CcBySa4), "CC BY-SA 4.0");
        assert_eq!(format!("{}", License::CcBy4), "CC BY 4.0");
        assert_eq!(format!("{}", License::Native), "Native");
        assert_eq!(format!("{}", License::LinkOnly), "Link-only");
    }

    #[test]
    fn variants_are_distinct() {
        assert_ne!(License::CcBySa4, License::CcBy4);
        assert_ne!(License::CcBy4, License::Native);
        assert_ne!(License::Native, License::LinkOnly);
        assert_ne!(License::LinkOnly, License::CcBySa4);
    }

    #[test]
    fn variants_are_comparable() {
        let a = License::CcBySa4;
        let b = License::CcBySa4;
        let c = License::CcBy4;

        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn variants_are_hashable() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(License::CcBySa4);
        set.insert(License::CcBy4);

        assert!(set.contains(&License::CcBySa4));
        assert!(set.contains(&License::CcBy4));
        assert!(!set.contains(&License::Native));
    }

    #[test]
    fn exhaustive_match() {
        // This test documents that the enum is exhaustive.
        // If a new variant were added, this match would fail to compile
        // without handling the new case.
        let license = License::Native;
        let _result = match license {
            License::CcBySa4 => "cc-by-sa",
            License::CcBy4 => "cc-by",
            License::Native => "native",
            License::LinkOnly => "link-only",
        };
    }
}
