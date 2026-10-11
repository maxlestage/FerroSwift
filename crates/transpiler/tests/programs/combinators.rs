#[derive(Debug, Clone, PartialEq)]
enum Priority { Low, Normal, Urgent }

#[derive(Debug, Clone)]
struct Task { id: u32, title: String, due: Option<(u16, u8, u8)>, prio: Priority, tags: Vec<String> }

fn parse_date(s: &str) -> Result<(u16, u8, u8), String> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return Err(format!("bad date '{s}'"));
    }
    let y = parts[0].parse::<u16>().map_err(|e| format!("year: {e}"))?;
    let m = parts[1].parse::<u8>().map_err(|e| format!("month: {e}"))?;
    let d = parts[2].parse::<u8>().map_err(|e| e.to_string())?;
    if m == 0 || m > 12 { Err(format!("month {m} out of range")) } else { Ok((y, m, d)) }
}

fn parse_task(id: u32, line: &str) -> Result<Task, String> {
    let mut fields = line.split('|').map(str::trim);
    let title = fields.next().filter(|t| !t.is_empty()).ok_or("missing title")?.to_string();
    let due = fields.next().filter(|s| !s.is_empty()).map(parse_date).transpose()?;
    let prio = match fields.next().unwrap_or("") {
        "!" => Priority::Urgent,
        "-" => Priority::Low,
        _ => Priority::Normal,
    };
    let tags = fields.next().map_or(vec![], |t| t.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect());
    Ok(Task { id, title, due, prio, tags })
}

fn main() {
    let input = ["Write report | 2024-05-01 | ! | work, urgent", "Buy milk | | - | home", "Call Bob | 2024-13-01 | |", "| 2024-01-01", "Plan trip | 2024-07-15 | | travel,fun", "Fix bike | 2024-x-02"];
    let results: Vec<Result<Task, String>> = input.iter().enumerate().map(|(i, l)| parse_task(i as u32 + 1, l)).collect();
    for r in &results {
        match r {
            Ok(t) => println!("ok #{} {:?} {:?} {:?} {:?}", t.id, t.title, t.due, t.prio, t.tags),
            Err(e) => println!("err {e}"),
        }
    }
    let good: Vec<Task> = results.iter().filter_map(|r| r.as_ref().ok()).cloned().collect();
    let all: Result<Vec<Task>, String> = input.iter().take(2).enumerate().map(|(i, l)| parse_task(i as u32, l)).collect();
    println!("{} {:?}", good.len(), all.map(|v| v.len()));
    let failed: Result<Vec<Task>, String> = input.iter().enumerate().map(|(i, l)| parse_task(i as u32, l)).collect();
    println!("{:?}", failed.err());
    let mut sorted = good.clone();
    sorted.sort_by(|a, b| b.due.cmp(&a.due).then_with(|| a.title.cmp(&b.title)));
    println!("{:?}", sorted.iter().map(|t| &t.title).collect::<Vec<_>>());
    let earliest = good.iter().filter_map(|t| t.due).min();
    println!("{:?} {:?}", earliest, earliest.map(|(y, m, _)| format!("{y}/{m:02}")));
    let first_tag = good.iter().find_map(|t| t.tags.first()).cloned().unwrap_or_default();
    let urgent = good.iter().any(|t| t.prio == Priority::Urgent);
    println!("{first_tag:?} {urgent} {:?}", good.iter().position(|t| t.tags.is_empty()));
    let opt_sum: Option<u32> = good.iter().map(|t| t.due.map(|d| d.2 as u32)).sum();
    println!("{:?}", opt_sum);
    let some_sum: Option<u32> = good.iter().filter(|t| t.due.is_some()).map(|t| t.due.map(|d| d.2 as u32)).sum();
    println!("{:?}", some_sum);
    let x: Option<i32> = Some(4);
    println!("{:?} {:?} {:?} {:?}", x.and_then(|v| if v > 3 { Some(v * 2) } else { None }), x.or(Some(1)), x.xor(None::<i32>), x.zip(Some('c')));
    println!("{} {:?} {:?}", x.map_or(0, |v| v + 1), x.filter(|v| v % 2 == 1), x.ok_or("none"));
    let r: Result<i32, String> = "12".parse::<i32>().map_err(|e| e.to_string());
    println!("{:?} {:?} {} {:?}", r.as_ref().map(|v| v * 2), r.clone().and_then(|v| if v > 100 { Ok(v) } else { Err(format!("{v} too small")) }), r.clone().unwrap_or_else(|_| -1), r.clone().ok());
}
