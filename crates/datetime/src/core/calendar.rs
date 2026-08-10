use arrayvec::ArrayVec;
use chrono::{Datelike as _, NaiveDate};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CalendarDay {
    pub number: u32,
    pub in_month: bool,
    pub today: bool,
}

/// Builds the fixed six-week grid used by the calendar presentation.
pub fn month_grid(date: NaiveDate) -> Option<ArrayVec<CalendarDay, 42>> {
    let first = date.with_day(1)?;
    let previous_month = first.pred_opt()?;
    let days_in_previous = previous_month.day();
    let next_month = if date.month() == 12 {
        NaiveDate::from_ymd_opt(date.year().checked_add(1)?, 1, 1)?
    } else {
        NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1)?
    };
    let days_in_month = next_month.pred_opt()?.day();
    let leading = first.weekday().num_days_from_sunday();

    Some(
        (0..42)
            .map(|cell| {
                if cell < leading {
                    CalendarDay {
                        number: days_in_previous - leading + cell + 1,
                        in_month: false,
                        today: false,
                    }
                } else {
                    let number = cell - leading + 1;
                    if number <= days_in_month {
                        CalendarDay {
                            number,
                            in_month: true,
                            today: number == date.day(),
                        }
                    } else {
                        CalendarDay {
                            number: number - days_in_month,
                            in_month: false,
                            today: false,
                        }
                    }
                }
            })
            .collect(),
    )
}
