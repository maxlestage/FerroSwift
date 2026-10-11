struct Job(u32);

impl Drop for Job {
    fn drop(&mut self) {
        println!("done {}", self.0);
    }
}

struct Worker {
    current: Job,
    queue: Vec<Job>,
}

fn next_job(n: u32) -> Option<Job> {
    if n < 2 { Some(Job(100 + n)) } else { None }
}

fn main() {
    let mut queue = vec![Job(1), Job(2), Job(3)];
    while let Some(job) = queue.pop() {
        println!("run {}", job.0);
    }
    if let Some(j) = next_job(0) {
        println!("got {}", j.0);
    }
    println!("between");
    match next_job(1) {
        Some(j) => println!("matched {}", j.0),
        None => println!("none"),
    }
    let mut w = Worker { current: Job(7), queue: vec![Job(8), Job(9)] };
    w.current = Job(70);
    println!("replaced current");
    w.queue[0] = Job(80);
    println!("replaced item");
    w.queue.clear();
    println!("cleared");
    w.queue.push(Job(90));
    println!("end");
}
