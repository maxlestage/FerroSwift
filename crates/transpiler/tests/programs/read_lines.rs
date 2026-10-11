use std::io;

fn main() {
    let mut line = String::new();
    let mut count = 0;
    loop {
        line.clear();
        let n = io::stdin().read_line(&mut line).expect("read failed");
        if n == 0 {
            break;
        }
        count += 1;
        let t = line.trim_end();
        println!("{count}: {t:?} ({n} bytes)");
    }
    println!("{count} lines");
}
