use chrono::{NaiveDateTime, TimeZone, Utc};
use chrono_tz::Europe::Helsinki;


pub fn fi_datetime(date: NaiveDateTime) -> String {
    let fi_time = Utc.from_utc_datetime(&date).with_timezone(&Helsinki);
    fi_time.to_string()
}