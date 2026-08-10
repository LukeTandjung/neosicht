use chrono::NaiveDate;
use datetime::core::calendar::month_grid;

#[test]
fn month_grid_contains_six_weeks_and_marks_today() {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let days = month_grid(date).unwrap();

    assert_eq!(days.len(), 42);
    assert_eq!(days.iter().filter(|day| day.today).count(), 1);
    assert!(days.iter().any(|day| day.today && day.number == 15));
    assert!(days.iter().any(|day| !day.in_month));
}
