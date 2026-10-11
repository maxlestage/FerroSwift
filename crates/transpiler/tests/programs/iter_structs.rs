struct Pairs<I: Iterator<Item = u8>> {
    inner: I,
    count: usize,
}

impl<I: Iterator<Item = u8>> Pairs<I> {
    fn new(inner: I) -> Self {
        Pairs { inner, count: 0 }
    }
}

impl<I: Iterator<Item = u8>> Iterator for Pairs<I> {
    type Item = (u8, u8);
    fn next(&mut self) -> Option<(u8, u8)> {
        let a = self.inner.next()?;
        let b = self.inner.next().unwrap_or(0);
        self.count += 1;
        Some((a, b))
    }
}

struct Words<I>
where
    I: Iterator<Item = char>,
{
    chars: I,
    done: bool,
}

impl<I> Iterator for Words<I>
where
    I: Iterator<Item = char>,
{
    type Item = String;
    fn next(&mut self) -> Option<String> {
        if self.done {
            return None;
        }
        let mut word = String::new();
        loop {
            match self.chars.next() {
                Some(' ') if !word.is_empty() => return Some(word),
                Some(' ') => continue,
                Some(c) => word.push(c),
                None => {
                    self.done = true;
                    return if word.is_empty() { None } else { Some(word) };
                }
            }
        }
    }
}

fn pairs_of<I: Iterator<Item = u8>>(it: I) -> Pairs<I> {
    Pairs::new(it)
}

fn main() {
    let mut p: Pairs<std::vec::IntoIter<u8>> = Pairs::new(vec![9, 8, 7].into_iter());
    println!("{:?} {:?} {:?} {}", p.next(), p.next(), p.next(), p.count);
    let total: u32 = pairs_of((1..=6u8).filter(|x| x % 2 == 1)).map(|(a, b)| a as u32 * b as u32).sum();
    println!("{total}");
    let w = Words { chars: "  hello  big world ".chars(), done: false };
    println!("{:?}", w.collect::<Vec<_>>());
    let lens: Vec<usize> = Words { chars: "a bb ccc".chars().rev(), done: false }.map(|s| s.len()).collect();
    println!("{:?}", lens);
    let mut zipped = pairs_of("xyz".bytes()).zip(Words { chars: "one two".chars(), done: false });
    println!("{:?} {:?}", zipped.next(), zipped.next());
}
