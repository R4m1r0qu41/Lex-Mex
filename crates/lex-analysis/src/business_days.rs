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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SexennialHoliday {
    pub month: u32,
    pub day: u32,
    pub first_year: i32,
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
                && day.year() >= holiday.first_year
                && (day.year() - holiday.first_year) % 6 == 0
        })
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
}
