//! Business-day counting under article 28 of the Ley Federal de Procedimiento
//! Administrativo, for commencement clauses such as `entrará en vigor a los
//! treinta días hábiles siguientes a su publicación`.
//!
//! JRH's direction (2026-10-27) is that article 28 governs the count. The
//! article names weekends and a fixed list of holidays, and it also excludes
//! "los días en que tengan vacaciones generales las autoridades competentes o
//! aquellos en que se suspendan las labores", which are made public by an
//! agreement published in the Diario Oficial. Those notices are inputs, not
//! something this module can assume away.
//!
//! A date is therefore produced only when every input it depends on is
//! evidenced:
//!
//! * a statute version applies on every day of the counting window (the text
//!   version carries its source and hash);
//! * the whole window, up to the date reached, lies inside a declared notice
//!   coverage range, so a missing suspension notice is a statement the
//!   operator made about a reviewed range, never an assumption.
//!
//! Otherwise the outcome is [`CountOutcome::Insufficient`] and the commencement
//! stays unresolved. No elapsed-time buffer is ever substituted for the
//! missing evidence.

use chrono::{Datelike, Days, NaiveDate, Weekday};

/// One annual holiday: month and day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedHoliday {
    pub month: u32,
    pub day: u32,
}

/// A holiday that falls only every six years, such as 1 December on the
/// transmission of the Federal Executive (1994, 2000, 2006, ...).
///
/// `last_year` is the last occurrence the entry is evidenced for. A later
/// occurrence year (`first_year + 6k > last_year`) is not counted as an
/// ordinary year: any window that touches it is insufficient, because which
/// day is then excluded is not evidenced (from 2024 the Executive changes on
/// 1 October). Choosing that day is a legal question, not a counting one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SexennialHoliday {
    pub month: u32,
    pub day: u32,
    pub first_year: i32,
    pub last_year: i32,
}

impl SexennialHoliday {
    fn occurs_in(&self, year: i32) -> bool {
        year >= self.first_year && (year - self.first_year) % 6 == 0
    }
}

/// A version of the statutory text that fixes which days are not business days.
#[derive(Debug, Clone)]
pub struct StatuteVersion {
    pub id: String,
    /// Where the text is recorded (record id and hash), for provenance.
    pub source: String,
    pub applies_from: NaiveDate,
    pub applies_to: Option<NaiveDate>,
    pub weekend_days: Vec<Weekday>,
    pub fixed_holidays: Vec<FixedHoliday>,
    pub sexennial_holidays: Vec<SexennialHoliday>,
}

/// An inclusive date range with its source.
#[derive(Debug, Clone)]
pub struct DateRange {
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub source: String,
}

/// The inputs a business-day count depends on. The default is empty, which
/// makes every count insufficient.
#[derive(Debug, Clone, Default)]
pub struct BusinessDayCalendar {
    pub rule_id: String,
    pub versions: Vec<StatuteVersion>,
    /// Published vacation or suspension periods that are not business days.
    pub suspensions: Vec<DateRange>,
    /// Ranges for which the suspension and vacation notices were collected.
    pub notice_coverage: Vec<DateRange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CountOutcome {
    /// Every input was evidenced; `date` is the Nth business day after publication.
    Date { date: NaiveDate, provenance: String },
    /// The count cannot be completed from evidenced inputs.
    Insufficient {
        /// The earliest the date could be, counting only weekends and the
        /// statutory holidays (suspensions can only delay it). Not a result.
        earliest_possible: Option<NaiveDate>,
        reason: String,
    },
}

impl BusinessDayCalendar {
    fn version_on(&self, day: NaiveDate) -> Option<&StatuteVersion> {
        self.versions.iter().find(|version| {
            version.applies_from <= day && version.applies_to.is_none_or(|end| day <= end)
        })
    }

    fn covered(&self, from: NaiveDate, through: NaiveDate) -> bool {
        let mut day = from;
        while day <= through {
            if !self
                .notice_coverage
                .iter()
                .any(|range| range.from <= day && day <= range.to)
            {
                return false;
            }
            let Some(next) = day.checked_add_days(Days::new(1)) else {
                return false;
            };
            day = next;
        }
        true
    }
}

fn statutory_non_business(version: &StatuteVersion, day: NaiveDate) -> bool {
    version.weekend_days.contains(&day.weekday())
        || version
            .fixed_holidays
            .iter()
            .any(|holiday| holiday.month == day.month() && holiday.day == day.day())
        || version.sexennial_holidays.iter().any(|holiday| {
            holiday.month == day.month()
                && holiday.day == day.day()
                && holiday.occurs_in(day.year())
        })
}

/// A sexennial occurrence year after the entry's evidenced range, if `day`
/// falls in one.
fn unevidenced_sexennial_year(version: &StatuteVersion, day: NaiveDate) -> Option<i32> {
    let year = day.year();
    version
        .sexennial_holidays
        .iter()
        .any(|holiday| holiday.occurs_in(year) && year > holiday.last_year)
        .then_some(year)
}

/// Upper limit on the days scanned, so a pathological input cannot loop.
const SCAN_LIMIT_DAYS: u64 = 4000;

/// The Nth business day after `publication` (the day of publication excluded),
/// counted under the supplied calendar.
#[must_use]
pub fn count_business_days_after(
    publication: NaiveDate,
    count: u64,
    calendar: &BusinessDayCalendar,
) -> CountOutcome {
    if count == 0 {
        return CountOutcome::Insufficient {
            earliest_possible: None,
            reason: "a count of zero business days is not a counting case".to_owned(),
        };
    }
    if calendar.versions.is_empty() {
        return CountOutcome::Insufficient {
            earliest_possible: None,
            reason: "no statutory text version is supplied for the counting rule".to_owned(),
        };
    }
    let earliest = walk(publication, count, calendar, false);
    let (date, window_start) = match walk(publication, count, calendar, true) {
        Walk::Reached(date) => (date, publication),
        Walk::NoVersion(day) => {
            return CountOutcome::Insufficient {
                earliest_possible: reached(&earliest),
                reason: format!("no statutory text version applies on {day}"),
            };
        }
        Walk::UnevidencedSexennial(year) => {
            return CountOutcome::Insufficient {
                earliest_possible: None,
                reason: format!(
                    "the counting window reaches {year}, a sexennial holiday year after the recorded entry's last evidenced year"
                ),
            };
        }
        Walk::Exhausted => {
            return CountOutcome::Insufficient {
                earliest_possible: reached(&earliest),
                reason: "the counting window could not be completed".to_owned(),
            };
        }
    };
    let first = window_start
        .checked_add_days(Days::new(1))
        .unwrap_or(window_start);
    if !calendar.covered(first, date) {
        return CountOutcome::Insufficient {
            earliest_possible: reached(&earliest),
            reason: format!(
                "suspension and vacation notices are not evidenced for the whole window {first} to {date}"
            ),
        };
    }
    let versions = calendar
        .versions
        .iter()
        .filter(|version| {
            version.applies_from <= date && version.applies_to.is_none_or(|end| first <= end)
        })
        .map(|version| format!("{} ({})", version.id, version.source))
        .collect::<Vec<_>>()
        .join("; ");
    let coverage = calendar
        .notice_coverage
        .iter()
        .filter(|range| range.from <= date && range.to >= first)
        .map(|range| format!("{} to {} ({})", range.from, range.to, range.source))
        .collect::<Vec<_>>()
        .join("; ");
    CountOutcome::Date {
        date,
        provenance: format!(
            "{}: business day {count} after publication {publication}, counted with {versions}; notice coverage {coverage}",
            calendar.rule_id
        ),
    }
}

enum Walk {
    Reached(NaiveDate),
    NoVersion(NaiveDate),
    UnevidencedSexennial(i32),
    Exhausted,
}

fn reached(walk: &Walk) -> Option<NaiveDate> {
    match walk {
        Walk::Reached(date) => Some(*date),
        _ => None,
    }
}

fn walk(
    publication: NaiveDate,
    count: u64,
    calendar: &BusinessDayCalendar,
    with_suspensions: bool,
) -> Walk {
    let mut day = publication;
    let mut seen = 0u64;
    for _ in 0..SCAN_LIMIT_DAYS {
        let Some(next) = day.checked_add_days(Days::new(1)) else {
            return Walk::Exhausted;
        };
        day = next;
        let Some(version) = calendar.version_on(day) else {
            return Walk::NoVersion(day);
        };
        if let Some(year) = unevidenced_sexennial_year(version, day) {
            return Walk::UnevidencedSexennial(year);
        }
        let suspended = with_suspensions
            && calendar
                .suspensions
                .iter()
                .any(|range| range.from <= day && day <= range.to);
        if statutory_non_business(version, day) || suspended {
            continue;
        }
        seen += 1;
        if seen == count {
            return Walk::Reached(day);
        }
    }
    Walk::Exhausted
}

/// The closed shape of `adapters/shared/_business-day-calendar.json`.
const CALENDAR_SCHEMA: &str = include_str!("../../../schemas/business-day-calendar-v1.schema.json");

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CalendarFile {
    #[serde(rename = "schema_version")]
    _schema_version: String,
    #[serde(rename = "description")]
    _description: String,
    rule_id: String,
    #[serde(rename = "counting_rule")]
    _counting_rule: serde_json::Value,
    statute_versions: Vec<VersionFile>,
    suspension_periods: Vec<RangeFile>,
    notice_coverage: Vec<RangeFile>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionFile {
    id: String,
    source: String,
    #[serde(rename = "evidence")]
    _evidence: String,
    applies_from: NaiveDate,
    applies_to: Option<NaiveDate>,
    weekend_days: Vec<String>,
    fixed_holidays: Vec<FixedFile>,
    sexennial_holidays: Vec<SexennialFile>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FixedFile {
    month: u32,
    day: u32,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SexennialFile {
    month: u32,
    day: u32,
    first_year: i32,
    last_year: i32,
    #[serde(rename = "reason")]
    _reason: String,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RangeFile {
    from: NaiveDate,
    to: NaiveDate,
    source: String,
}

fn range(file: RangeFile, list: &str) -> Result<DateRange, String> {
    if file.from > file.to {
        return Err(format!(
            "{list}: range {} to {} ends before it starts",
            file.from, file.to
        ));
    }
    Ok(DateRange {
        from: file.from,
        to: file.to,
        source: file.source,
    })
}

fn real_day(month: u32, day: u32) -> bool {
    // 2000 is a leap year, so 29 February is accepted.
    NaiveDate::from_ymd_opt(2000, month, day).is_some()
}

fn version(file: VersionFile) -> Result<StatuteVersion, String> {
    if file.applies_to.is_some_and(|end| end < file.applies_from) {
        return Err(format!(
            "statute version {}: applies_to precedes applies_from",
            file.id
        ));
    }
    let weekend_days = file
        .weekend_days
        .iter()
        .map(|name| {
            name.parse::<Weekday>()
                .map_err(|_| format!("statute version {}: unknown weekday {name}", file.id))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut fixed_holidays = Vec::new();
    for holiday in file.fixed_holidays {
        if !real_day(holiday.month, holiday.day) {
            return Err(format!(
                "statute version {}: {}-{} is not a calendar day",
                file.id, holiday.month, holiday.day
            ));
        }
        fixed_holidays.push(FixedHoliday {
            month: holiday.month,
            day: holiday.day,
        });
    }
    let mut sexennial_holidays = Vec::new();
    for holiday in file.sexennial_holidays {
        if !real_day(holiday.month, holiday.day)
            || holiday.last_year < holiday.first_year
            || (holiday.last_year - holiday.first_year) % 6 != 0
        {
            return Err(format!(
                "statute version {}: sexennial holiday {}-{} needs a real day and a last_year that is an occurrence year at or after first_year",
                file.id, holiday.month, holiday.day
            ));
        }
        sexennial_holidays.push(SexennialHoliday {
            month: holiday.month,
            day: holiday.day,
            first_year: holiday.first_year,
            last_year: holiday.last_year,
        });
    }
    Ok(StatuteVersion {
        id: file.id,
        source: file.source,
        applies_from: file.applies_from,
        applies_to: file.applies_to,
        weekend_days,
        fixed_holidays,
        sexennial_holidays,
    })
}

/// Parse `adapters/shared/_business-day-calendar.json`.
///
/// The file must match `schemas/business-day-calendar-v1.schema.json`
/// exactly: every key is required, unknown keys are rejected, and an
/// intentionally empty list is written as `[]`. Ranges must be ordered, statute
/// versions must not overlap, and holidays must be real days. Nothing is
/// defaulted, so a typo can never silently drop evidence.
///
/// # Errors
/// Returns a description of the first schema or semantic violation.
pub fn parse_calendar(bytes: &[u8]) -> Result<BusinessDayCalendar, String> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|error| format!("invalid JSON: {error}"))?;
    let schema: serde_json::Value =
        serde_json::from_str(CALENDAR_SCHEMA).map_err(|error| error.to_string())?;
    jsonschema::validator_for(&schema)
        .map_err(|error| error.to_string())?
        .validate(&value)
        .map_err(|error| {
            format!(
                "does not match business-day-calendar-v1 at {}: {error}",
                error.instance_path
            )
        })?;
    let file: CalendarFile = serde_json::from_value(value).map_err(|error| error.to_string())?;
    let mut versions = file
        .statute_versions
        .into_iter()
        .map(version)
        .collect::<Result<Vec<_>, _>>()?;
    versions.sort_by_key(|version| version.applies_from);
    for pair in versions.windows(2) {
        if pair[0]
            .applies_to
            .is_none_or(|end| end >= pair[1].applies_from)
        {
            return Err(format!(
                "statute versions {} and {} overlap",
                pair[0].id, pair[1].id
            ));
        }
    }
    Ok(BusinessDayCalendar {
        rule_id: file.rule_id,
        versions,
        suspensions: file
            .suspension_periods
            .into_iter()
            .map(|item| range(item, "suspension_periods"))
            .collect::<Result<_, _>>()?,
        notice_coverage: file
            .notice_coverage
            .into_iter()
            .map(|item| range(item, "notice_coverage"))
            .collect::<Result<_, _>>()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
    }

    fn version() -> StatuteVersion {
        // The unamended text of article 28 recorded in the corpus.
        let fixed = [
            (1, 1),
            (2, 5),
            (3, 21),
            (5, 1),
            (5, 5),
            (9, 1),
            (9, 16),
            (11, 20),
            (12, 25),
        ];
        StatuteVersion {
            id: "lfpa-art28-original".to_owned(),
            source: "test record".to_owned(),
            applies_from: date(1995, 6, 1),
            applies_to: None,
            weekend_days: vec![Weekday::Sat, Weekday::Sun],
            fixed_holidays: fixed
                .iter()
                .map(|&(month, day)| FixedHoliday { month, day })
                .collect(),
            sexennial_holidays: vec![SexennialHoliday {
                month: 12,
                day: 1,
                first_year: 1994,
                last_year: 2018,
            }],
        }
    }

    fn calendar(
        coverage: &[(NaiveDate, NaiveDate)],
        suspensions: &[(NaiveDate, NaiveDate)],
    ) -> BusinessDayCalendar {
        let range = |&(from, to): &(NaiveDate, NaiveDate)| DateRange {
            from,
            to,
            source: "test notice review".to_owned(),
        };
        BusinessDayCalendar {
            rule_id: "lfpa-art-28".to_owned(),
            versions: vec![version()],
            suspensions: suspensions.iter().map(range).collect(),
            notice_coverage: coverage.iter().map(range).collect(),
        }
    }

    fn whole_year(y: i32) -> (NaiveDate, NaiveDate) {
        (date(y, 1, 1), date(y, 12, 31))
    }

    fn date_of(outcome: &CountOutcome) -> Option<NaiveDate> {
        match outcome {
            CountOutcome::Date { date, .. } => Some(*date),
            CountOutcome::Insufficient { .. } => None,
        }
    }

    #[test]
    fn weekends_are_skipped() {
        // Thursday 2023-06-01: Fri 2 (1), Mon 5 (2), Tue 6 (3), Wed 7 (4), Thu 8 (5)
        let outcome =
            count_business_days_after(date(2023, 6, 1), 5, &calendar(&[whole_year(2023)], &[]));
        assert_eq!(date_of(&outcome), Some(date(2023, 6, 8)));
    }

    #[test]
    fn a_statutory_holiday_is_skipped() {
        // Friday 2023-11-17: Monday 20 November is a listed holiday, so the
        // second business day is Wednesday 22 and not Tuesday 21.
        let outcome =
            count_business_days_after(date(2023, 11, 17), 2, &calendar(&[whole_year(2023)], &[]));
        assert_eq!(date_of(&outcome), Some(date(2023, 11, 22)));
        // the same count, one year earlier shape, with no holiday in the way
        let plain =
            count_business_days_after(date(2023, 6, 16), 2, &calendar(&[whole_year(2023)], &[]));
        assert_eq!(date_of(&plain), Some(date(2023, 6, 20)));
    }

    #[test]
    fn the_first_of_december_is_a_holiday_only_when_the_executive_changes() {
        // 2000 is a transmission year (1994 + 6 + 6), 2004 is not.
        let transmission =
            count_business_days_after(date(2000, 11, 30), 1, &calendar(&[whole_year(2000)], &[]));
        assert_eq!(date_of(&transmission), Some(date(2000, 12, 4)));
        let ordinary =
            count_business_days_after(date(2004, 11, 30), 1, &calendar(&[whole_year(2004)], &[]));
        assert_eq!(date_of(&ordinary), Some(date(2004, 12, 1)));
    }

    #[test]
    fn explicitly_supplied_suspension_periods_are_skipped() {
        // Mon 5 and Tue 6 June 2023 suspended: Fri 2 (1), Wed 7 (2), Thu 8 (3), Fri 9 (4), Mon 12 (5)
        let c = calendar(&[whole_year(2023)], &[(date(2023, 6, 5), date(2023, 6, 6))]);
        assert_eq!(
            date_of(&count_business_days_after(date(2023, 6, 1), 5, &c)),
            Some(date(2023, 6, 12))
        );
    }

    #[test]
    fn a_suspension_that_pushes_the_date_past_the_covered_range_is_insufficient() {
        // coverage ends on 2023-06-09 but the suspension moves the 5th business day to the 12th
        let c = calendar(
            &[(date(2023, 1, 1), date(2023, 6, 9))],
            &[(date(2023, 6, 5), date(2023, 6, 6))],
        );
        let outcome = count_business_days_after(date(2023, 6, 1), 5, &c);
        assert!(
            matches!(outcome, CountOutcome::Insufficient { .. }),
            "{outcome:?}"
        );
    }

    #[test]
    fn missing_notice_coverage_is_never_read_as_no_suspension() {
        for coverage in [
            vec![],
            vec![(date(2023, 6, 1), date(2023, 6, 6))],
            vec![(date(2023, 6, 3), date(2023, 12, 31))],
        ] {
            let outcome = count_business_days_after(date(2023, 6, 1), 5, &calendar(&coverage, &[]));
            match outcome {
                CountOutcome::Insufficient {
                    earliest_possible,
                    reason,
                } => {
                    // only a lower bound is reported, and it is not a result
                    assert_eq!(earliest_possible, Some(date(2023, 6, 8)));
                    assert!(reason.contains("not evidenced"), "{reason}");
                }
                CountOutcome::Date { .. } => panic!("a date was produced without coverage"),
            }
        }
    }

    #[test]
    fn a_window_before_every_statute_version_is_insufficient() {
        // the recorded text applies from 1995-06-01; a window in 1993 has no evidenced text
        let c = calendar(&[whole_year(1993)], &[]);
        match count_business_days_after(date(1993, 5, 3), 5, &c) {
            CountOutcome::Insufficient { reason, .. } => {
                assert!(reason.contains("no statutory text version"), "{reason}");
            }
            other @ CountOutcome::Date { .. } => panic!("{other:?}"),
        }
        // and a window that starts inside the version's range works
        let ok = calendar(&[whole_year(1996)], &[]);
        assert!(date_of(&count_business_days_after(date(1996, 5, 3), 5, &ok)).is_some());
    }

    #[test]
    fn an_empty_calendar_and_a_zero_count_resolve_nothing() {
        assert!(
            date_of(&count_business_days_after(
                date(2023, 6, 1),
                5,
                &BusinessDayCalendar::default()
            ))
            .is_none()
        );
        assert!(
            date_of(&count_business_days_after(
                date(2023, 6, 1),
                0,
                &calendar(&[whole_year(2023)], &[])
            ))
            .is_none()
        );
    }

    #[test]
    fn the_reviews_counterexample_is_not_resolved_without_evidence() {
        // Codex's synthetic case: publication 2020-03-20, N = 30. The old rule claimed
        // 2020-06-18 as a proven latest date. With weekdays 23 March to 30 June suspended,
        // the 30th business day is 2020-08-11. Supplied as evidence, that is what is counted.
        let suspended = [(date(2020, 3, 23), date(2020, 6, 30))];
        let c = calendar(&[(date(2020, 1, 1), date(2020, 12, 31))], &suspended);
        assert_eq!(
            date_of(&count_business_days_after(date(2020, 3, 20), 30, &c)),
            Some(date(2020, 8, 11))
        );
        // without that notice evidenced, nothing is claimed
        let none = calendar(&[], &[]);
        assert!(date_of(&count_business_days_after(date(2020, 3, 20), 30, &none)).is_none());
    }

    #[test]
    fn provenance_names_the_rule_the_version_and_the_coverage() {
        match count_business_days_after(date(2023, 6, 1), 5, &calendar(&[whole_year(2023)], &[])) {
            CountOutcome::Date { provenance, .. } => {
                assert!(provenance.contains("lfpa-art-28"));
                assert!(provenance.contains("lfpa-art28-original"));
                assert!(provenance.contains("test notice review"));
            }
            other @ CountOutcome::Insufficient { .. } => panic!("{other:?}"),
        }
    }

    const SHIPPED: &str = include_str!("../../../adapters/shared/_business-day-calendar.json");

    /// The shipped calendar with one edit applied to its JSON value.
    fn edited(edit: impl FnOnce(&mut serde_json::Value)) -> Vec<u8> {
        let mut value: serde_json::Value = serde_json::from_str(SHIPPED).expect("shipped JSON");
        edit(&mut value);
        serde_json::to_vec(&value).expect("serialize")
    }

    #[test]
    fn the_shipped_calendar_parses_and_resolves_nothing_without_coverage() {
        let parsed = parse_calendar(SHIPPED.as_bytes()).expect("shipped calendar is valid");
        assert_eq!(parsed.rule_id, "lfpa-art-28");
        assert_eq!(parsed.versions.len(), 1);
        assert_eq!(parsed.versions[0].sexennial_holidays[0].last_year, 2018);
        assert!(parsed.notice_coverage.is_empty() && parsed.suspensions.is_empty());
        assert!(date_of(&count_business_days_after(date(2005, 3, 1), 10, &parsed)).is_none());
    }

    #[test]
    fn a_misspelled_missing_or_extra_calendar_key_is_an_error() {
        // Codex audit 2026-11-17, C2: `suspension_period` used to be read as no
        // suspensions, so the count ended two business days early.
        let cases: Vec<(&str, Vec<u8>)> = vec![
            (
                "misspelled list",
                edited(|v| {
                    let periods = v
                        .as_object_mut()
                        .unwrap()
                        .remove("suspension_periods")
                        .unwrap();
                    v["suspension_period"] = periods;
                }),
            ),
            (
                "missing list",
                edited(|v| {
                    v.as_object_mut().unwrap().remove("notice_coverage");
                }),
            ),
            (
                "missing nested list",
                edited(|v| {
                    v["statute_versions"][0]
                        .as_object_mut()
                        .unwrap()
                        .remove("fixed_holidays");
                }),
            ),
            (
                "missing applies_to key",
                edited(|v| {
                    v["statute_versions"][0]
                        .as_object_mut()
                        .unwrap()
                        .remove("applies_to");
                }),
            ),
            (
                "unknown nested key",
                edited(|v| {
                    v["statute_versions"][0]["weekend_day"] = serde_json::json!(["sunday"]);
                }),
            ),
            (
                "wrong schema version",
                edited(|v| v["schema_version"] = serde_json::json!("2")),
            ),
            (
                "empty provenance",
                edited(|v| {
                    v["statute_versions"][0]["source"] = serde_json::json!("");
                }),
            ),
            (
                "uncapped sexennial entry",
                edited(|v| {
                    v["statute_versions"][0]["sexennial_holidays"][0]
                        .as_object_mut()
                        .unwrap()
                        .remove("last_year");
                }),
            ),
        ];
        for (name, bytes) in cases {
            assert!(parse_calendar(&bytes).is_err(), "{name} was accepted");
        }
    }

    #[test]
    fn calendar_semantics_are_checked_after_the_schema() {
        let range =
            |from: &str, to: &str| serde_json::json!({"from": from, "to": to, "source": "test"});
        let cases: Vec<(&str, Vec<u8>)> = vec![
            (
                "reversed coverage",
                edited(|v| {
                    v["notice_coverage"] = serde_json::json!([range("2020-12-31", "2020-01-01")]);
                }),
            ),
            (
                "reversed suspension",
                edited(|v| {
                    v["suspension_periods"] =
                        serde_json::json!([range("2020-06-06", "2020-06-05")]);
                }),
            ),
            (
                "not a calendar day",
                edited(|v| {
                    v["statute_versions"][0]["fixed_holidays"][0] =
                        serde_json::json!({"month": 2, "day": 30});
                }),
            ),
            (
                "not a real date",
                edited(|v| {
                    v["notice_coverage"] = serde_json::json!([range("2020-02-30", "2020-03-01")]);
                }),
            ),
            (
                "last_year off the cycle",
                edited(|v| {
                    v["statute_versions"][0]["sexennial_holidays"][0]["last_year"] =
                        serde_json::json!(2019);
                }),
            ),
            (
                "overlapping statute versions",
                edited(|v| {
                    let mut second = v["statute_versions"][0].clone();
                    second["id"] = serde_json::json!("second");
                    second["applies_from"] = serde_json::json!("2000-01-01");
                    v["statute_versions"].as_array_mut().unwrap().push(second);
                }),
            ),
        ];
        for (name, bytes) in cases {
            assert!(parse_calendar(&bytes).is_err(), "{name} was accepted");
        }
        // two versions that meet without overlapping are accepted
        let ordered = edited(|v| {
            v["statute_versions"][0]["applies_to"] = serde_json::json!("1999-12-31");
            let mut second = v["statute_versions"][0].clone();
            second["id"] = serde_json::json!("second");
            second["applies_from"] = serde_json::json!("2000-01-01");
            second["applies_to"] = serde_json::Value::Null;
            v["statute_versions"].as_array_mut().unwrap().push(second);
        });
        assert_eq!(
            parse_calendar(&ordered)
                .expect("ordered versions")
                .versions
                .len(),
            2
        );
    }

    #[test]
    fn a_window_reaching_an_unevidenced_transmission_year_is_insufficient() {
        // Codex review 2026-10-28, deferred limit 3: from 2024 the Executive
        // changes on 1 October, so 2024 is not counted as an ordinary year.
        let c = calendar(&[whole_year(2024)], &[]);
        match count_business_days_after(date(2024, 3, 1), 5, &c) {
            CountOutcome::Insufficient { reason, .. } => {
                assert!(reason.contains("2024"), "{reason}");
            }
            other @ CountOutcome::Date { .. } => panic!("{other:?}"),
        }
        // a window ending in 2023 is unaffected; 2018 is still evidenced
        let earlier = calendar(&[whole_year(2023)], &[]);
        assert_eq!(
            date_of(&count_business_days_after(date(2023, 6, 1), 5, &earlier)),
            Some(date(2023, 6, 8))
        );
        let last = calendar(&[whole_year(2018)], &[]);
        assert_eq!(
            date_of(&count_business_days_after(date(2018, 11, 30), 1, &last)),
            Some(date(2018, 12, 3))
        );
    }
}
