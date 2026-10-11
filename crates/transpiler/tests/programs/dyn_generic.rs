trait Converter<T> {
    fn convert(&self, x: T) -> T;
    fn name(&self) -> String {
        "converter".to_string()
    }
}

struct Doubler;
struct Offset(f64);
struct Upper;

impl Converter<f64> for Doubler {
    fn convert(&self, x: f64) -> f64 {
        x * 2.0
    }
    fn name(&self) -> String {
        "doubler".into()
    }
}

impl Converter<f64> for Offset {
    fn convert(&self, x: f64) -> f64 {
        x + self.0
    }
}

impl Converter<String> for Upper {
    fn convert(&self, x: String) -> String {
        x.to_uppercase()
    }
}

fn apply_all(cs: &[Box<dyn Converter<f64>>], x: f64) -> f64 {
    cs.iter().fold(x, |acc, c| c.convert(acc))
}

fn run(c: &dyn Converter<String>, s: &str) -> String {
    c.convert(s.to_string())
}

fn main() {
    let cs: Vec<Box<dyn Converter<f64>>> = vec![Box::new(Doubler), Box::new(Offset(1.5))];
    println!("{}", apply_all(&cs, 2.0));
    for c in &cs {
        println!("{} {}", c.name(), c.convert(10.0));
    }
    println!("{}", run(&Upper, "abc"));
    let picked: &dyn Converter<f64> = if cs.len() > 1 { &Doubler } else { &Offset(0.0) };
    println!("{}", picked.convert(4.0));
}
