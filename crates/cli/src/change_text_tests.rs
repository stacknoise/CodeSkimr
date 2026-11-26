//! The dates of the text: the day of the epoch, a leap day, a date before 1970 and the turn of a
//! century.

use super::time_text;

#[test]
fn the_epoch_is_the_first_of_january_1970() {
    assert_eq!(time_text(0), "1970-01-01 00:00");
}

#[test]
fn a_leap_day_is_kept() {
    assert_eq!(time_text(951_782_400 + 3_600 + 120), "2000-02-29 01:02");
}

#[test]
fn a_date_before_1970_is_counted_backwards() {
    assert_eq!(time_text(-1), "1969-12-31 23:59");
    assert_eq!(time_text(-86_400 * 365), "1969-01-01 00:00");
}

#[test]
fn a_year_divisible_by_100_but_not_by_400_has_no_leap_day() {
    assert_eq!(time_text(4_107_542_400), "2100-03-01 00:00");
}

#[test]
fn the_last_day_of_a_year_and_a_current_date() {
    assert_eq!(time_text(1_798_761_600 - 60), "2026-12-31 23:59");
    assert_eq!(time_text(1_795_000_000), "2026-11-18 11:06");
}
