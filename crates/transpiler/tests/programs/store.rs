use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, PartialEq)]
enum Value {
    Int(i64),
    Text(String),
    List(Vec<Value>),
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Int(n)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::Text(s.to_string())
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(items: Vec<T>) -> Self {
        Value::List(items.into_iter().map(Into::into).collect())
    }
}

impl Value {
    fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "int",
            Value::Text(_) => "text",
            Value::List(_) => "list",
        }
    }

    fn flatten(&self) -> Vec<&Value> {
        match self {
            Value::List(items) => items.iter().flat_map(|v| v.flatten()).collect(),
            other => vec![other],
        }
    }

    fn render(&self) -> String {
        match self {
            Value::Int(n) => n.to_string(),
            Value::Text(s) => format!("'{s}'"),
            Value::List(items) => format!("[{}]", items.iter().map(Value::render).collect::<Vec<_>>().join(", ")),
        }
    }
}

struct Store {
    data: BTreeMap<String, Value>,
    log: Vec<String>,
}

impl Store {
    fn set(&mut self, key: &str, value: impl Into<Value>) -> Option<Value> {
        let value = value.into();
        self.log.push(format!("set {key}={}", value.render()));
        self.data.insert(key.to_string(), value)
    }

    fn get_int(&self, key: &str) -> Option<i64> {
        match self.data.get(key)? {
            Value::Int(n) => Some(*n),
            _ => None,
        }
    }

    fn incr(&mut self, key: &str, by: i64) -> Result<i64, String> {
        match self.data.get_mut(key) {
            Some(Value::Int(n)) => {
                *n += by;
                Ok(*n)
            }
            Some(other) => Err(format!("{key} is a {}", other.type_name())),
            None => {
                self.data.insert(key.to_string(), Value::Int(by));
                Ok(by)
            }
        }
    }

    fn push(&mut self, key: &str, item: impl Into<Value>) -> Result<usize, String> {
        let entry = self.data.entry(key.to_string()).or_insert_with(|| Value::List(vec![]));
        match entry {
            Value::List(items) => {
                items.push(item.into());
                Ok(items.len())
            }
            other => Err(format!("{key} is a {}", other.type_name())),
        }
    }

    fn by_type(&self) -> HashMap<&'static str, Vec<&str>> {
        let mut out: HashMap<&'static str, Vec<&str>> = HashMap::new();
        for (k, v) in &self.data {
            out.entry(v.type_name()).or_default().push(k.as_str());
        }
        out
    }
}

fn main() {
    let mut store = Store { data: BTreeMap::new(), log: Vec::new() };
    println!("{:?}", store.set("a", 1));
    println!("{:?}", store.set("a", "one"));
    println!("{:?}", store.set("list", vec![1i64, 2, 3]));
    println!("{:?}", store.set("nested", vec![Value::from(1), Value::from(vec!["x", "y"]), Value::List(vec![])]));
    println!("{:?} {:?} {:?}", store.get_int("a"), store.get_int("missing"), store.incr("count", 5));
    println!("{:?} {:?} {:?}", store.incr("count", -2), store.incr("a", 1), store.incr("new", 7));
    println!("{:?} {:?} {:?}", store.push("list", 4i64), store.push("items", "first"), store.push("a", 1i64));
    let mut types: Vec<(&str, Vec<&str>)> = store.by_type().into_iter().collect();
    types.sort();
    println!("{:?}", types);
    for (k, v) in &store.data {
        println!("{k:<7} {:<5} {} {}", v.type_name(), v.render(), v.flatten().len());
    }
    println!("{}", store.log.join(" | "));
    let v: Value = vec![vec![1i64, 2], vec![3]].into();
    println!("{} {:?}", v.render(), v.flatten().iter().filter_map(|x| if let Value::Int(n) = x { Some(*n) } else { None }).sum::<i64>());
    println!("{}", Value::from("q") == Value::Text("q".into()));
}
