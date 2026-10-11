use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Date {
    year: i32,
    month: u32,
    day: u32,
}

const MONTH_DAYS: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const WEEKDAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: i32, month: u32) -> u32 {
    if month == 2 && is_leap(year) { 29 } else { MONTH_DAYS[(month - 1) as usize] }
}

impl Date {
    fn new(year: i32, month: u32, day: u32) -> Result<Date, String> {
        if !(1..=12).contains(&month) {
            return Err(format!("month {month} out of range"));
        }
        if day == 0 || day > days_in_month(year, month) {
            return Err(format!("day {day} out of range for {year}-{month:02}"));
        }
        Ok(Date { year, month, day })
    }

    /// Days since 1970-01-01 (may be negative).
    fn ordinal(&self) -> i64 {
        let y = if self.month <= 2 { self.year - 1 } else { self.year } as i64;
        let era = y.div_euclid(400);
        let yoe = y.rem_euclid(400);
        let m = self.month as i64;
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + self.day as i64 - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146097 + doe - 719468
    }

    fn from_ordinal(days: i64) -> Date {
        let z = days + 719468;
        let era = z.div_euclid(146097);
        let doe = z.rem_euclid(146097);
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
        let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
        Date { year: (if m <= 2 { y + 1 } else { y }) as i32, month: m, day: d }
    }

    fn weekday(&self) -> &'static str {
        WEEKDAYS[(self.ordinal() + 3).rem_euclid(7) as usize]
    }

    fn add_days(&self, n: i64) -> Date {
        Date::from_ordinal(self.ordinal() + n)
    }

    fn days_until(&self, other: &Date) -> i64 {
        other.ordinal() - self.ordinal()
    }

    fn add_months(&self, n: i32) -> Date {
        let total = self.year * 12 + (self.month as i32 - 1) + n;
        let year = total.div_euclid(12);
        let month = (total.rem_euclid(12) + 1) as u32;
        let day = self.day.min(days_in_month(year, month));
        Date { year, month, day }
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl FromStr for Date {
    type Err = String;
    fn from_str(s: &str) -> Result<Date, String> {
        let parts: Vec<&str> = s.trim().split('-').collect();
        if parts.len() != 3 {
            return Err(format!("expected y-m-d, got {s:?}"));
        }
        let year = parts[0].parse::<i32>().map_err(|e| format!("year: {e}"))?;
        let month = parts[1].parse::<u32>().map_err(|e| format!("month: {e}"))?;
        let day = parts[2].parse::<u32>().map_err(|e| format!("day: {e}"))?;
        Date::new(year, month, day)
    }
}

fn main() {
    let epoch = Date::new(1970, 1, 1).unwrap();
    println!("{} {} {}", epoch, epoch.ordinal(), epoch.weekday());
    for s in ["2024-02-29", "2023-02-29", "2000-13-01", "1999-12-31", "x-1-1", "2024-1", " 1969-07-20 "] {
        match s.parse::<Date>() {
            Ok(d) => println!("{s:?} -> {d} ord={} {} leap={}", d.ordinal(), d.weekday(), is_leap(d.year)),
            Err(e) => println!("{s:?} -> error: {e}"),
        }
    }
    let d = "2024-01-31".parse::<Date>().unwrap();
    println!("{} {} {} {}", d.add_days(1), d.add_days(-31), d.add_days(366), d.add_months(1));
    println!("{} {} {}", d.add_months(13), d.add_months(-2), d.add_months(-13));
    let later = Date::new(2025, 3, 1).unwrap();
    println!("{} {} {}", d.days_until(&later), later.days_until(&d), later > d);
    let mut dates: Vec<Date> = ["2024-03-01", "1999-12-31", "2024-02-29", "2024-03-01"].iter().map(|s| s.parse().unwrap()).collect();
    dates.sort();
    dates.dedup();
    println!("{}", dates.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(", "));
    println!("{:?} {:?}", dates.iter().max(), dates.binary_search(&Date::new(2024, 2, 29).unwrap()));
    let mut cur = Date::new(2024, 12, 29).unwrap();
    let mut log = Vec::new();
    while cur.year == 2024 {
        log.push(format!("{}/{}", cur.day, cur.weekday()));
        cur = cur.add_days(1);
    }
    println!("{} -> {cur}", log.join(" "));
    println!("{:?}", Date::from_ordinal(-1));
    println!("{}", (0..7).map(|i| epoch.add_days(i * 365 + i * 100).weekday()).collect::<Vec<_>>().join(""));
}
