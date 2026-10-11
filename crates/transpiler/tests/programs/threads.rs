use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

fn parallel_sum(data: Vec<u64>, workers: usize) -> u64 {
    let chunk = (data.len() + workers - 1) / workers;
    let data = Arc::new(data);
    let mut handles = Vec::new();
    for w in 0..workers {
        let data = Arc::clone(&data);
        handles.push(thread::spawn(move || {
            let start = w * chunk;
            let end = (start + chunk).min(data.len());
            data[start..end].iter().sum::<u64>()
        }));
    }
    handles.into_iter().map(|h| h.join().unwrap()).sum()
}

fn main() {
    println!("{}", parallel_sum((1..=100).collect(), 4));

    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    for i in 0..5 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            let mut n = counter.lock().unwrap();
            *n += i;
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    println!("counter {}", *counter.lock().unwrap());

    let (tx, rx) = mpsc::channel();
    for id in 0..3 {
        let tx = tx.clone();
        thread::spawn(move || {
            for j in 0..2 {
                tx.send(id * 10 + j).unwrap();
            }
        });
    }
    drop(tx);
    let mut got: Vec<i32> = rx.iter().collect();
    got.sort();
    println!("{:?}", got);

    let (tx, rx) = mpsc::channel::<String>();
    let worker = thread::spawn(move || {
        let mut out = Vec::new();
        for msg in rx {
            out.push(msg.to_uppercase());
        }
        out
    });
    for w in ["a", "b", "c"] {
        tx.send(w.to_string()).unwrap();
    }
    drop(tx);
    println!("{:?}", worker.join().unwrap());

    let (tx, rx) = mpsc::channel();
    tx.send(1).unwrap();
    println!("{:?} {:?}", rx.try_recv(), rx.try_recv());
    drop(tx);
    println!("{:?} {:?}", rx.try_recv(), rx.recv());
    match rx.try_recv() {
        Err(mpsc::TryRecvError::Disconnected) => println!("closed"),
        Err(mpsc::TryRecvError::Empty) => println!("empty"),
        Ok(v) => println!("value {v}"),
    }

    let hits = Arc::new(AtomicUsize::new(0));
    let flag = Arc::new(AtomicBool::new(false));
    thread::scope(|s| {
        for _ in 0..4 {
            let hits = &hits;
            s.spawn(move || {
                hits.fetch_add(2, Ordering::SeqCst);
            });
        }
        s.spawn(|| flag.store(true, Ordering::Release));
    });
    println!("{} {} {:?}", hits.load(Ordering::SeqCst), flag.load(Ordering::Acquire), hits);
    println!("{:?} {:?}", hits.compare_exchange(8, 1, Ordering::SeqCst, Ordering::SeqCst), hits.compare_exchange(8, 2, Ordering::SeqCst, Ordering::SeqCst));
    println!("{} {}", hits.swap(5, Ordering::Relaxed), hits.fetch_sub(1, Ordering::Relaxed));

    let cache: Arc<RwLock<HashMap<String, i32>>> = Arc::new(RwLock::new(HashMap::new()));
    let writer = {
        let cache = Arc::clone(&cache);
        thread::spawn(move || {
            cache.write().unwrap().insert("k".to_string(), 7);
        })
    };
    writer.join().unwrap();
    println!("{:?}", cache.read().unwrap().get("k"));
    let slow = thread::spawn(|| 42);
    thread::sleep(Duration::from_millis(5));
    println!("{} {}", slow.is_finished(), slow.join().unwrap());
}
