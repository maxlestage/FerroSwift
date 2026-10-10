use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Order {
    customer: String,
    item: String,
    quantity: u32,
    unit_price: f64,
}

impl Order {
    fn new(customer: &str, item: &str, quantity: u32, unit_price: f64) -> Self {
        Order {
            customer: customer.to_string(),
            item: item.to_string(),
            quantity,
            unit_price,
        }
    }

    fn total(&self) -> f64 {
        self.quantity as f64 * self.unit_price
    }
}

fn main() {
    let orders = vec![
        Order::new("Ada", "keyboard", 1, 89.0),
        Order::new("Linus", "mouse", 2, 25.5),
        Order::new("Ada", "monitor", 2, 199.99),
        Order::new("Grace", "cable", 5, 4.25),
    ];

    // map + sum
    let revenue: f64 = orders.iter().map(|o| o.total()).sum();
    println!("Revenue: {revenue:.2}");

    // filter + map + collect
    let big: Vec<String> = orders
        .iter()
        .filter(|o| o.total() > 50.0)
        .map(|o| format!("{} × {}", o.quantity, o.item))
        .collect();
    println!("Big orders: {:?}", big);

    // fold
    let items = orders.iter().fold(0, |count, o| count + o.quantity);
    println!("{items} items sold");

    // HashMap: total spent per customer
    let mut spent: HashMap<String, f64> = HashMap::new();
    for order in &orders {
        *spent.entry(order.customer.clone()).or_insert(0.0) += order.total();
    }
    let mut names: Vec<&String> = spent.keys().collect();
    names.sort();
    for name in names {
        println!("{name}: {:.2}", spent[name]);
    }

    // enumerate, any, max_by
    for (i, order) in orders.iter().enumerate() {
        println!("#{} {} ({})", i + 1, order.item, order.customer);
    }
    let has_cables = orders.iter().any(|o| o.item == "cable");
    let priciest = orders
        .iter()
        .max_by(|a, b| a.unit_price.partial_cmp(&b.unit_price).unwrap())
        .map(|o| o.item.clone())
        .unwrap_or_default();
    println!("cables: {has_cables}, priciest: {priciest}");
}
