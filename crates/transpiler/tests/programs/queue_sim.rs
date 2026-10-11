use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    Regular = 1,
    Member = 3,
    Vip = 5,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Customer {
    id: u32,
    kind: Kind,
    arrival: u32,
    service: u32,
}

impl Customer {
    const MAX_WAIT: u32 = 30;

    const fn weight(kind: Kind) -> u32 {
        kind as u32 * 10
    }

    fn priority(&self) -> u32 {
        Self::weight(self.kind) + self.service / 2
    }

    fn too_late(&self, now: u32) -> bool {
        now.saturating_sub(self.arrival) > Self::MAX_WAIT
    }
}

impl Ord for Customer {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority()
            .cmp(&other.priority())
            .then_with(|| other.arrival.cmp(&self.arrival))
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl PartialOrd for Customer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

static KINDS: [Kind; 3] = [Kind::Regular, Kind::Member, Kind::Vip];

fn main() {
    let mut arrivals: VecDeque<Customer> = VecDeque::new();
    let mut seed = 7u32;
    for id in 1..=12 {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let kind = KINDS[(seed >> 16) as usize % 3];
        let service = 2 + (seed >> 8) % 7;
        arrivals.push_back(Customer { id, kind, arrival: id * 3, service });
    }
    println!("{:?}", arrivals.iter().map(|c| format!("{}:{:?}:{}", c.id, c.kind, c.service)).collect::<Vec<_>>());
    let mut queue: BinaryHeap<Customer> = BinaryHeap::new();
    let mut now = 0;
    let mut served: Vec<(u32, u32)> = Vec::new();
    let mut abandoned = Vec::new();
    while !arrivals.is_empty() || !queue.is_empty() {
        while let Some(c) = arrivals.front() {
            if c.arrival <= now {
                queue.push(arrivals.pop_front().unwrap());
            } else {
                break;
            }
        }
        match queue.pop() {
            Some(c) if c.too_late(now) => abandoned.push(c.id),
            Some(c) => {
                let wait = now - c.arrival;
                served.push((c.id, wait));
                now += c.service;
                continue;
            }
            None => {}
        }
        now += 1;
        assert!(now < 1000, "runaway simulation");
    }
    println!("{:?}", served);
    println!("{:?} {} {}", abandoned, now, served.len() + abandoned.len());
    let waits: Vec<u32> = served.iter().map(|&(_, w)| w).collect();
    let mean = f64::from(waits.iter().sum::<u32>()) / waits.len() as f64;
    let worst = served.iter().max_by_key(|&&(id, w)| (w, Reverse(id)));
    println!("{mean:>6.2} {:?} {:?} {}", worst, waits.iter().max(), Customer::MAX_WAIT);
    let mut heap: BinaryHeap<Reverse<(u32, u32)>> = BinaryHeap::from(vec![Reverse((5, 1)), Reverse((2, 9)), Reverse((2, 3))]);
    heap.push(Reverse((1, 7)));
    println!("{:?} {} {:?}", heap.peek(), heap.len(), heap.iter().map(|r| r.0.0).max());
    let drained: Vec<(u32, u32)> = std::iter::from_fn(|| heap.pop().map(|r| r.0)).collect();
    println!("{:?}", drained);
    let sorted = BinaryHeap::from(vec![3, 1, 4, 1, 5]).into_sorted_vec();
    println!("{:?} {} {} {}", sorted, Customer::weight(Kind::Vip), Kind::Member as i32 + 1, KINDS.iter().map(|k| *k as u32).sum::<u32>());
    debug_assert_eq!(sorted.len(), 5);
    assert_ne!(sorted[0], sorted[4], "distinct ends");
    println!("{}", matches!(Kind::Vip, Kind::Vip | Kind::Member) && !matches!(Kind::Regular, Kind::Vip));
}
