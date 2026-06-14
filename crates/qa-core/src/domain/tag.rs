//! Tag value object: immutable tag with date and jurisdiction facets.
//!
//! Tags carry a date (YYYY-MM-DD) and jurisdiction scope, enabling filtering
//! and staleness tracking. Parse-Don't-Validate: invalid dates or jurisdictions
//! are rejected at construction.

use std::fmt;

/// The complete set of officially-assigned ISO 3166-1 alpha-2 country codes,
/// stored sorted to permit binary search at parse time.
///
/// This is the canonical standard for representing countries/jurisdictions.
/// Reserved, user-assigned, and exceptionally-reserved codes (e.g. `EU`, `UK`)
/// are deliberately excluded: only officially-assigned codes are valid.
const ISO_3166_1_ALPHA2: &[[u8; 2]] = &[
    *b"AD", *b"AE", *b"AF", *b"AG", *b"AI", *b"AL", *b"AM", *b"AO", *b"AQ", *b"AR", *b"AS", *b"AT",
    *b"AU", *b"AW", *b"AX", *b"AZ", *b"BA", *b"BB", *b"BD", *b"BE", *b"BF", *b"BG", *b"BH", *b"BI",
    *b"BJ", *b"BL", *b"BM", *b"BN", *b"BO", *b"BQ", *b"BR", *b"BS", *b"BT", *b"BV", *b"BW", *b"BY",
    *b"BZ", *b"CA", *b"CC", *b"CD", *b"CF", *b"CG", *b"CH", *b"CI", *b"CK", *b"CL", *b"CM", *b"CN",
    *b"CO", *b"CR", *b"CU", *b"CV", *b"CW", *b"CX", *b"CY", *b"CZ", *b"DE", *b"DJ", *b"DK", *b"DM",
    *b"DO", *b"DZ", *b"EC", *b"EE", *b"EG", *b"EH", *b"ER", *b"ES", *b"ET", *b"FI", *b"FJ", *b"FK",
    *b"FM", *b"FO", *b"FR", *b"GA", *b"GB", *b"GD", *b"GE", *b"GF", *b"GG", *b"GH", *b"GI", *b"GL",
    *b"GM", *b"GN", *b"GP", *b"GQ", *b"GR", *b"GS", *b"GT", *b"GU", *b"GW", *b"GY", *b"HK", *b"HM",
    *b"HN", *b"HR", *b"HT", *b"HU", *b"ID", *b"IE", *b"IL", *b"IM", *b"IN", *b"IO", *b"IQ", *b"IR",
    *b"IS", *b"IT", *b"JE", *b"JM", *b"JO", *b"JP", *b"KE", *b"KG", *b"KH", *b"KI", *b"KM", *b"KN",
    *b"KP", *b"KR", *b"KW", *b"KY", *b"KZ", *b"LA", *b"LB", *b"LC", *b"LI", *b"LK", *b"LR", *b"LS",
    *b"LT", *b"LU", *b"LV", *b"LY", *b"MA", *b"MC", *b"MD", *b"ME", *b"MF", *b"MG", *b"MH", *b"MK",
    *b"ML", *b"MM", *b"MN", *b"MO", *b"MP", *b"MQ", *b"MR", *b"MS", *b"MT", *b"MU", *b"MV", *b"MW",
    *b"MX", *b"MY", *b"MZ", *b"NA", *b"NC", *b"NE", *b"NF", *b"NG", *b"NI", *b"NL", *b"NO", *b"NP",
    *b"NR", *b"NU", *b"NZ", *b"OM", *b"PA", *b"PE", *b"PF", *b"PG", *b"PH", *b"PK", *b"PL", *b"PM",
    *b"PN", *b"PR", *b"PS", *b"PT", *b"PW", *b"PY", *b"QA", *b"RE", *b"RO", *b"RS", *b"RU", *b"RW",
    *b"SA", *b"SB", *b"SC", *b"SD", *b"SE", *b"SG", *b"SH", *b"SI", *b"SJ", *b"SK", *b"SL", *b"SM",
    *b"SN", *b"SO", *b"SR", *b"SS", *b"ST", *b"SV", *b"SX", *b"SY", *b"SZ", *b"TC", *b"TD", *b"TF",
    *b"TG", *b"TH", *b"TJ", *b"TK", *b"TL", *b"TM", *b"TN", *b"TO", *b"TR", *b"TT", *b"TV", *b"TW",
    *b"TZ", *b"UA", *b"UG", *b"UM", *b"US", *b"UY", *b"UZ", *b"VA", *b"VC", *b"VE", *b"VG", *b"VI",
    *b"VN", *b"VU", *b"WF", *b"WS", *b"YE", *b"YT", *b"ZA", *b"ZM", *b"ZW",
];

/// A jurisdiction, represented as an ISO 3166-1 alpha-2 country code.
///
/// This is the international standard for identifying countries. Parse-Don't-Validate:
/// only officially-assigned codes can be constructed, so an invalid jurisdiction
/// is unrepresentable. The code is stored canonically (uppercase ASCII).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Jurisdiction([u8; 2]);

impl Jurisdiction {
    /// Parse an ISO 3166-1 alpha-2 country code (case-insensitive on input,
    /// stored uppercase).
    ///
    /// # Errors
    ///
    /// Returns `Err(JurisdictionError::InvalidLength)` if the input is not
    /// exactly two ASCII letters.
    /// Returns `Err(JurisdictionError::Unassigned)` if the two letters are not
    /// an officially-assigned ISO 3166-1 alpha-2 code.
    pub fn new(s: &str) -> Result<Self, JurisdictionError> {
        let bytes = s.as_bytes();

        if bytes.len() != 2 || !bytes[0].is_ascii_alphabetic() || !bytes[1].is_ascii_alphabetic() {
            return Err(JurisdictionError::InvalidLength);
        }

        let code = [bytes[0].to_ascii_uppercase(), bytes[1].to_ascii_uppercase()];

        if ISO_3166_1_ALPHA2.binary_search(&code).is_err() {
            return Err(JurisdictionError::Unassigned);
        }

        Ok(Jurisdiction(code))
    }

    /// The ISO 3166-1 alpha-2 code as a string slice (always uppercase ASCII).
    pub fn as_str(&self) -> &str {
        // Safe: codes are validated ASCII letters at construction.
        std::str::from_utf8(&self.0).expect("ISO code is valid ASCII")
    }
}

impl fmt::Display for Jurisdiction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Error when parsing a Jurisdiction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JurisdictionError {
    /// Input is not exactly two ASCII letters.
    InvalidLength,
    /// The code is not an officially-assigned ISO 3166-1 alpha-2 code.
    Unassigned,
}

impl fmt::Display for JurisdictionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength => write!(
                f,
                "jurisdiction must be a two-letter ISO 3166-1 alpha-2 code"
            ),
            Self::Unassigned => {
                write!(f, "jurisdiction is not an assigned ISO 3166-1 alpha-2 code")
            }
        }
    }
}

impl std::error::Error for JurisdictionError {}

/// A date in YYYY-MM-DD format.
///
/// Validates both format and calendar validity at construction: the month must
/// be 01–12 and the day must exist in that month (Gregorian, leap-year aware).
/// No time-of-day or leap-second handling — this is a calendar date only.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    year: u16,
    month: u8,
    day: u8,
}

impl Date {
    /// Parse a date string in YYYY-MM-DD format.
    ///
    /// # Errors
    ///
    /// Returns `Err(DateError::InvalidFormat)` if the string is not exactly
    /// 10 bytes, has misplaced separators, or contains any non-ASCII-digit
    /// character in a digit position (including a `+`/`-` sign or whitespace).
    /// Returns `Err(DateError::InvalidMonth)` if month is not 01–12.
    /// Returns `Err(DateError::InvalidDay)` if day is not a valid day for the
    /// given month and year (Gregorian, leap-year aware).
    pub fn new(s: &str) -> Result<Self, DateError> {
        if s.len() != 10 {
            return Err(DateError::InvalidFormat);
        }

        let bytes = s.as_bytes();

        // Validate layout: YYYY-MM-DD with '-' separators and an ASCII digit in
        // every digit position. Rejecting non-digits up front closes the hole
        // where integer parsing silently accepts a leading '+' sign (e.g.
        // "+025-06-14" would otherwise parse the year as 25, not 2025).
        if bytes[4] != b'-' || bytes[7] != b'-' {
            return Err(DateError::InvalidFormat);
        }

        let digits_only = |slice: &[u8]| !slice.is_empty() && slice.iter().all(u8::is_ascii_digit);
        if !digits_only(&bytes[0..4]) || !digits_only(&bytes[5..7]) || !digits_only(&bytes[8..10]) {
            return Err(DateError::InvalidFormat);
        }

        // Every digit position is an ASCII digit, so these parses cannot fail
        // (4 digits ≤ 9999 < u16::MAX; 2 digits ≤ 99 < u8::MAX).
        let year: u16 = s[0..4].parse().map_err(|_| DateError::InvalidFormat)?;
        let month: u8 = s[5..7].parse().map_err(|_| DateError::InvalidFormat)?;
        let day: u8 = s[8..10].parse().map_err(|_| DateError::InvalidFormat)?;

        if !(1..=12).contains(&month) {
            return Err(DateError::InvalidMonth);
        }

        if day == 0 || day > days_in_month(year, month) {
            return Err(DateError::InvalidDay);
        }

        Ok(Date { year, month, day })
    }

    /// Year component.
    pub fn year(self) -> u16 {
        self.year
    }

    /// Month component (1–12).
    pub fn month(self) -> u8 {
        self.month
    }

    /// Day component (1–31).
    pub fn day(self) -> u8 {
        self.day
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// Number of days in the given month, accounting for leap years in February.
///
/// `month` must be 1–12; callers validate this before calling.
fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        // Unreachable: month is validated to 1..=12 before this is called.
        _ => 0,
    }
}

/// Whether `year` is a leap year in the proleptic Gregorian calendar.
fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Error when parsing a Date.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DateError {
    /// Input is not 10 characters or separators are wrong.
    InvalidFormat,
    /// Month is not 01–12.
    InvalidMonth,
    /// Day is not 01–31.
    InvalidDay,
}

impl fmt::Display for DateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFormat => write!(f, "date must be in YYYY-MM-DD format"),
            Self::InvalidMonth => write!(f, "month must be 01–12"),
            Self::InvalidDay => write!(f, "day must be 01–31"),
        }
    }
}

impl std::error::Error for DateError {}

/// A tag: immutable, labeled, with date and jurisdiction scope.
///
/// Tags enable filtering and staleness tracking. A tag must have a non-empty
/// label, a valid date, and a jurisdiction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tag {
    label: String,
    date: Date,
    jurisdiction: Jurisdiction,
}

impl Tag {
    /// Parse a new tag from label, date string, and jurisdiction.
    ///
    /// The label is normalized — surrounding whitespace trimmed and folded to
    /// lowercase — so that semantically identical tags compare equal and do not
    /// fragment the tag facet (e.g. `"Rust"`, `"rust"`, and `"  rust  "` all
    /// become the single tag `"rust"`).
    ///
    /// # Errors
    ///
    /// Returns `Err(TagError::EmptyLabel)` if the label is empty or
    /// whitespace-only.
    /// Returns `Err(TagError::InvalidDate)` if the date string is malformed.
    pub fn new(
        label: impl Into<String>,
        date_str: &str,
        jurisdiction: Jurisdiction,
    ) -> Result<Self, TagError> {
        let label = label.into().trim().to_lowercase();

        if label.is_empty() {
            return Err(TagError::EmptyLabel);
        }

        let date = Date::new(date_str).map_err(TagError::InvalidDate)?;

        Ok(Tag {
            label,
            date,
            jurisdiction,
        })
    }

    /// Tag label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Tag date.
    pub fn date(&self) -> Date {
        self.date
    }

    /// Tag jurisdiction.
    pub fn jurisdiction(&self) -> Jurisdiction {
        self.jurisdiction
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}#{}", self.label, self.date, self.jurisdiction)
    }
}

/// Error when parsing a Tag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TagError {
    /// Label is empty or whitespace-only.
    EmptyLabel,
    /// Date string is invalid.
    InvalidDate(DateError),
}

impl fmt::Display for TagError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyLabel => write!(f, "tag label cannot be empty or whitespace-only"),
            Self::InvalidDate(e) => write!(f, "invalid tag date: {}", e),
        }
    }
}

impl std::error::Error for TagError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_valid_format() {
        let d = Date::new("2025-06-14").expect("valid");
        assert_eq!(d.year(), 2025);
        assert_eq!(d.month(), 6);
        assert_eq!(d.day(), 14);
    }

    #[test]
    fn date_display() {
        let d = Date::new("2025-06-14").expect("valid");
        assert_eq!(format!("{}", d), "2025-06-14");
    }

    #[test]
    fn date_rejects_short_format() {
        assert_eq!(
            Date::new("2025-06-1").unwrap_err(),
            DateError::InvalidFormat
        );
    }

    #[test]
    fn date_rejects_long_format() {
        assert_eq!(
            Date::new("2025-06-145").unwrap_err(),
            DateError::InvalidFormat
        );
    }

    #[test]
    fn date_rejects_wrong_separators() {
        assert_eq!(
            Date::new("2025/06/14").unwrap_err(),
            DateError::InvalidFormat
        );
    }

    #[test]
    fn date_rejects_invalid_month() {
        assert_eq!(
            Date::new("2025-13-01").unwrap_err(),
            DateError::InvalidMonth
        );
        assert_eq!(
            Date::new("2025-00-01").unwrap_err(),
            DateError::InvalidMonth
        );
    }

    #[test]
    fn date_rejects_invalid_day() {
        assert_eq!(Date::new("2025-06-32").unwrap_err(), DateError::InvalidDay);
        assert_eq!(Date::new("2025-06-00").unwrap_err(), DateError::InvalidDay);
    }

    #[test]
    fn date_rejects_plus_sign_in_components() {
        // Integer FromStr accepts a leading '+', which would silently corrupt
        // values (e.g. "+025-06-14" -> year 25). All must be rejected as format.
        assert_eq!(
            Date::new("2025-+5-14").unwrap_err(),
            DateError::InvalidFormat
        );
        assert_eq!(
            Date::new("+025-06-14").unwrap_err(),
            DateError::InvalidFormat
        );
        assert_eq!(
            Date::new("2025-06-+9").unwrap_err(),
            DateError::InvalidFormat
        );
    }

    #[test]
    fn date_rejects_whitespace_in_components() {
        assert_eq!(
            Date::new("2025-06- 9").unwrap_err(),
            DateError::InvalidFormat
        );
        assert_eq!(
            Date::new(" 025-06-14").unwrap_err(),
            DateError::InvalidFormat
        );
    }

    #[test]
    fn date_rejects_impossible_calendar_days() {
        // 31-day check is month-aware, not a flat 1..=31.
        assert_eq!(Date::new("2025-02-31").unwrap_err(), DateError::InvalidDay);
        assert_eq!(Date::new("2025-04-31").unwrap_err(), DateError::InvalidDay);
        assert_eq!(Date::new("2025-06-31").unwrap_err(), DateError::InvalidDay);
    }

    #[test]
    fn date_handles_february_leap_years() {
        // Divisible by 4 but not 100 -> leap.
        assert!(Date::new("2024-02-29").is_ok());
        // Not divisible by 4 -> common year.
        assert_eq!(Date::new("2025-02-29").unwrap_err(), DateError::InvalidDay);
        // Divisible by 400 -> leap.
        assert!(Date::new("2000-02-29").is_ok());
        // Divisible by 100 but not 400 -> common year.
        assert_eq!(Date::new("1900-02-29").unwrap_err(), DateError::InvalidDay);
        // 28 is always valid in February.
        assert!(Date::new("2025-02-28").is_ok());
    }

    #[test]
    fn date_accepts_month_boundary_days() {
        assert!(Date::new("2025-01-31").is_ok());
        assert!(Date::new("2025-04-30").is_ok());
        assert!(Date::new("2025-12-31").is_ok());
    }

    #[test]
    fn date_comparison() {
        let d1 = Date::new("2025-06-14").expect("valid");
        let d2 = Date::new("2025-06-15").expect("valid");
        let d3 = Date::new("2025-06-14").expect("valid");

        assert!(d1 < d2);
        assert!(d1 == d3);
        assert!(d2 > d1);
    }

    #[test]
    fn jurisdiction_valid_codes() {
        assert_eq!(Jurisdiction::new("US").unwrap().as_str(), "US");
        assert_eq!(Jurisdiction::new("ID").unwrap().as_str(), "ID");
        assert_eq!(Jurisdiction::new("JP").unwrap().as_str(), "JP");
        assert_eq!(Jurisdiction::new("GB").unwrap().as_str(), "GB");
    }

    #[test]
    fn jurisdiction_canonicalizes_to_uppercase() {
        assert_eq!(Jurisdiction::new("us").unwrap().as_str(), "US");
        assert_eq!(Jurisdiction::new("id").unwrap().as_str(), "ID");
        assert_eq!(Jurisdiction::new("Jp").unwrap().as_str(), "JP");
    }

    #[test]
    fn jurisdiction_display() {
        assert_eq!(format!("{}", Jurisdiction::new("US").unwrap()), "US");
        assert_eq!(format!("{}", Jurisdiction::new("FR").unwrap()), "FR");
    }

    #[test]
    fn jurisdiction_rejects_wrong_length() {
        assert_eq!(
            Jurisdiction::new("U").unwrap_err(),
            JurisdictionError::InvalidLength
        );
        assert_eq!(
            Jurisdiction::new("USA").unwrap_err(),
            JurisdictionError::InvalidLength
        );
        assert_eq!(
            Jurisdiction::new("").unwrap_err(),
            JurisdictionError::InvalidLength
        );
    }

    #[test]
    fn jurisdiction_rejects_non_alphabetic() {
        assert_eq!(
            Jurisdiction::new("U1").unwrap_err(),
            JurisdictionError::InvalidLength
        );
        assert_eq!(
            Jurisdiction::new("-1").unwrap_err(),
            JurisdictionError::InvalidLength
        );
    }

    #[test]
    fn jurisdiction_rejects_unassigned_codes() {
        // ZZ is reserved for user-assignment, never officially assigned.
        assert_eq!(
            Jurisdiction::new("ZZ").unwrap_err(),
            JurisdictionError::Unassigned
        );
        // EU and UK are exceptionally-reserved, not officially-assigned alpha-2.
        assert_eq!(
            Jurisdiction::new("EU").unwrap_err(),
            JurisdictionError::Unassigned
        );
        assert_eq!(
            Jurisdiction::new("UK").unwrap_err(),
            JurisdictionError::Unassigned
        );
    }

    #[test]
    fn iso_table_is_sorted() {
        // Binary search at parse time depends on the table being sorted.
        assert!(ISO_3166_1_ALPHA2.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn tag_valid_construction() {
        let tag = Tag::new(
            "clinical-software",
            "2025-06-14",
            Jurisdiction::new("US").unwrap(),
        )
        .expect("valid");

        assert_eq!(tag.label(), "clinical-software");
        assert_eq!(tag.date(), Date::new("2025-06-14").unwrap());
        assert_eq!(tag.jurisdiction(), Jurisdiction::new("US").unwrap());
    }

    #[test]
    fn tag_rejects_empty_label() {
        assert_eq!(
            Tag::new("", "2025-06-14", Jurisdiction::new("US").unwrap()).unwrap_err(),
            TagError::EmptyLabel
        );
    }

    #[test]
    fn tag_rejects_whitespace_label() {
        assert_eq!(
            Tag::new("   ", "2025-06-14", Jurisdiction::new("US").unwrap()).unwrap_err(),
            TagError::EmptyLabel
        );
    }

    #[test]
    fn tag_rejects_invalid_date() {
        let err = Tag::new("tag", "2025-13-01", Jurisdiction::new("US").unwrap()).unwrap_err();
        assert!(matches!(
            err,
            TagError::InvalidDate(DateError::InvalidMonth)
        ));
    }

    #[test]
    fn tag_display() {
        let tag = Tag::new("rust", "2024-01-15", Jurisdiction::new("US").unwrap()).expect("valid");
        assert_eq!(format!("{}", tag), "rust@2024-01-15#US");
    }

    #[test]
    fn tag_normalizes_label() {
        // Surrounding whitespace is trimmed and the label is folded to lowercase
        // so semantically identical tags do not fragment the facet.
        let tag = Tag::new(
            "  Clinical-Software  ",
            "2025-06-14",
            Jurisdiction::new("US").unwrap(),
        )
        .expect("valid");
        assert_eq!(tag.label(), "clinical-software");
    }

    #[test]
    fn tag_normalization_makes_variants_equal() {
        let j = Jurisdiction::new("US").unwrap();
        let a = Tag::new("Rust", "2025-06-14", j).unwrap();
        let b = Tag::new("rust", "2025-06-14", j).unwrap();
        let c = Tag::new("  RUST  ", "2025-06-14", j).unwrap();

        assert_eq!(a, b);
        assert_eq!(b, c);
    }

    #[test]
    fn multiple_jurisdictions() {
        let us = Tag::new("testing", "2025-06-14", Jurisdiction::new("US").unwrap()).unwrap();
        let id = Tag::new("testing", "2025-06-14", Jurisdiction::new("ID").unwrap()).unwrap();
        let jp = Tag::new("testing", "2025-06-14", Jurisdiction::new("JP").unwrap()).unwrap();

        assert_ne!(us, id);
        assert_ne!(id, jp);
        assert_eq!(us.jurisdiction(), Jurisdiction::new("US").unwrap());
    }
}
