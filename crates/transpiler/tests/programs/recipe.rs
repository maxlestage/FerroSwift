use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Unit {
    Kg,
    G,
    Lb,
}

impl Unit {
    fn to_grams(self) -> f64 {
        match self {
            Unit::Kg => 1000.0,
            Unit::G => 1.0,
            Unit::Lb => 453.592,
        }
    }

    fn parse(s: &str) -> Option<Unit> {
        match s.to_ascii_lowercase().as_str() {
            "kg" => Some(Unit::Kg),
            "g" => Some(Unit::G),
            "lb" | "lbs" => Some(Unit::Lb),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Quantity {
    amount: f64,
    unit: Unit,
}

impl Quantity {
    fn grams(&self) -> f64 {
        self.amount * self.unit.to_grams()
    }

    fn convert(&self, unit: Unit) -> Quantity {
        Quantity { amount: self.grams() / unit.to_grams(), unit }
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.amount.fract() == 0.0 {
            write!(f, "{} {:?}", self.amount as i64, self.unit)
        } else {
            write!(f, "{:.2} {:?}", self.amount, self.unit)
        }
    }
}

impl std::str::FromStr for Quantity {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();
        let amount: f64 = parts
            .next()
            .ok_or("empty")?
            .parse()
            .map_err(|e| format!("bad amount: {e}"))?;
        let unit = parts.next().and_then(Unit::parse).ok_or_else(|| "bad unit".to_string())?;
        if parts.next().is_some() {
            return Err("trailing text".into());
        }
        Ok(Quantity { amount, unit })
    }
}

#[derive(Debug, Default)]
struct Recipe {
    name: String,
    ingredients: Vec<(String, Quantity)>,
    steps: Vec<String>,
}

impl Recipe {
    fn new(name: &str) -> Self {
        Recipe { name: name.to_string(), ..Default::default() }
    }

    fn ingredient(mut self, name: &str, q: &str) -> Self {
        match q.parse::<Quantity>() {
            Ok(q) => self.ingredients.push((name.to_string(), q)),
            Err(e) => self.steps.push(format!("skipped {name}: {e}")),
        }
        self
    }

    fn step(mut self, s: &str) -> Self {
        self.steps.push(s.to_string());
        self
    }

    fn total_grams(&self) -> f64 {
        self.ingredients.iter().map(|(_, q)| q.grams()).sum()
    }

    fn scaled(&self, factor: f64) -> Recipe {
        Recipe {
            name: format!("{} x{factor}", self.name),
            ingredients: self
                .ingredients
                .iter()
                .map(|(n, q)| (n.clone(), Quantity { amount: q.amount * factor, ..*q }))
                .collect(),
            steps: self.steps.clone(),
        }
    }
}

fn main() {
    let r = Recipe::new("bread")
        .ingredient("flour", "500 g")
        .ingredient("water", "0.35 kg")
        .ingredient("salt", "two g")
        .ingredient("yeast", "7 g extra")
        .ingredient("butter", "1 lb")
        .step("mix")
        .step("bake");
    println!("{} {} {:.1}", r.name, r.ingredients.len(), r.total_grams());
    for (name, q) in &r.ingredients {
        println!("{name}: {q} = {:.0} g = {}", q.grams(), q.convert(Unit::Kg));
    }
    println!("{:?}", r.steps);
    let double = r.scaled(2.0);
    println!("{} {}", double.name, double.ingredients.iter().map(|(_, q)| q.to_string()).collect::<Vec<_>>().join(", "));
    println!("{:?}", "3 lbs".parse::<Quantity>().map(|q| q.convert(Unit::G).amount.round()));
    println!("{:?} {:?}", "".parse::<Quantity>(), "x g".parse::<Quantity>());
    let by_unit: HashMap<Unit, Vec<&str>> = r.ingredients.iter().fold(HashMap::new(), |mut m, (n, q)| {
        m.entry(q.unit).or_insert_with(Vec::new).push(n.as_str());
        m
    });
    let mut units: Vec<_> = by_unit.iter().map(|(u, ns)| format!("{u:?}={}", ns.join("+"))).collect();
    units.sort();
    println!("{:?}", units);
    let heaviest = r.ingredients.iter().max_by(|a, b| a.1.grams().partial_cmp(&b.1.grams()).unwrap()).map(|(n, _)| n.as_str());
    println!("{:?}", heaviest);
    let q = Quantity { amount: 2.5, unit: Unit::Kg };
    println!("{q} {} {:?} {}", q.convert(Unit::G), q.convert(Unit::Lb).amount as i32, q == q.clone());
    println!("{}", matches!(Unit::parse("LB"), Some(Unit::Lb)));
}
