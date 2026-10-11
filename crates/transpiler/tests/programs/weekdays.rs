use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Weekday {
    Mon = 1,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}

impl Weekday {
    const ALL: [Weekday; 7] = [Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Thu, Weekday::Fri, Weekday::Sat, Weekday::Sun];

    fn from_index(i: u8) -> Option<Weekday> {
        Weekday::ALL.iter().copied().find(|d| *d as u8 == i)
    }

    fn is_weekend(self) -> bool {
        matches!(self, Weekday::Sat | Weekday::Sun)
    }

    fn next(self) -> Weekday {
        Weekday::from_index(self as u8 % 7 + 1).unwrap()
    }
}

impl fmt::Display for Weekday {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = format!("{self:?}");
        f.pad(&name)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Shift {
    day: Weekday,
    start: u8,
    hours: f32,
}

#[derive(Debug, Default)]
struct Roster {
    shifts: HashMap<String, Vec<Shift>>,
}

impl Roster {
    fn assign(&mut self, who: &str, day: Weekday, start: u8, hours: f32) -> Result<(), String> {
        if start >= 24 || hours <= 0.0 || start as f32 + hours > 24.0 {
            return Err(format!("{who}: bad shift {day} {start}h+{hours}"));
        }
        let shifts = self.shifts.entry(who.to_string()).or_default();
        if shifts.iter().any(|s| s.day == day && (start as f32) < s.start as f32 + s.hours && (s.start as f32) < start as f32 + hours) {
            return Err(format!("{who}: overlap on {day}"));
        }
        shifts.push(Shift { day, start, hours });
        shifts.sort_by(|a, b| a.day.cmp(&b.day).then(a.start.cmp(&b.start)));
        Ok(())
    }

    fn hours(&self, who: &str) -> f32 {
        self.shifts.get(who).map_or(0.0, |v| v.iter().map(|s| s.hours).sum())
    }

    fn weekend_workers(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self
            .shifts
            .iter()
            .filter(|(_, v)| v.iter().any(|s| s.day.is_weekend()))
            .map(|(k, _)| k.as_str())
            .collect();
        names.sort_unstable();
        names
    }

    fn busiest_day(&self) -> Option<(Weekday, f32)> {
        let mut per_day: HashMap<Weekday, f32> = HashMap::new();
        for s in self.shifts.values().flatten() {
            *per_day.entry(s.day).or_insert(0.0) += s.hours;
        }
        per_day.into_iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap().then(b.0.cmp(&a.0)))
    }
}

fn main() {
    println!("{:?} {} {} {}", Weekday::Wed, Weekday::Wed as i32, Weekday::Sun.next(), Weekday::Fri.next().is_weekend());
    println!("[{:>5}] [{:<5}] [{:^7}]", Weekday::Mon, Weekday::Tue, Weekday::Sat);
    println!("{:?} {:?}", Weekday::from_index(3), Weekday::from_index(9));
    println!("{}", Weekday::ALL.iter().filter(|d| !d.is_weekend()).count());
    println!("{:?}", Weekday::ALL.iter().map(|d| d.to_string().to_lowercase()).collect::<Vec<_>>().join(","));
    println!("{} {}", Weekday::Mon < Weekday::Sun, Weekday::ALL.iter().max().unwrap());
    let mut roster = Roster::default();
    let attempts = [
        roster.assign("ana", Weekday::Mon, 9, 8.0),
        roster.assign("ana", Weekday::Mon, 16, 2.0),
        roster.assign("ana", Weekday::Mon, 10, 1.0),
        roster.assign("bo", Weekday::Sat, 20, 5.0),
        roster.assign("bo", Weekday::Sun, 23, 2.0),
        roster.assign("cy", Weekday::Sat, 8, 4.5),
        roster.assign("cy", Weekday::Fri, 25, 1.0),
    ];
    for a in &attempts {
        match a {
            Ok(()) => println!("ok"),
            Err(e) => println!("error: {e}"),
        }
    }
    println!("{} {} {} {}", roster.hours("ana"), roster.hours("bo"), roster.hours("cy"), roster.hours("dee"));
    println!("{:?}", roster.weekend_workers());
    println!("{:?}", roster.busiest_day());
    println!("{:?}", roster.shifts.get("ana").map(|v| v.iter().map(|s| (s.day, s.start)).collect::<Vec<_>>()));
    let total: f32 = roster.shifts.values().map(|v| v.iter().map(|s| s.hours).sum::<f32>()).sum();
    println!("{total} {:.1} {}", total / 3.0, (total * 10.0).round() / 10.0);
    let mut names: Vec<&String> = roster.shifts.keys().collect();
    names.sort();
    println!("{:?} {}", names, roster.shifts.contains_key("bo"));
}
