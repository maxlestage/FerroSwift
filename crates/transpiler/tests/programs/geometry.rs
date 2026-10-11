type P = (f64, f64);

fn cross(o: P, a: P, b: P) -> f64 {
    (a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0)
}

fn convex_hull(points: &[P]) -> Vec<P> {
    let mut pts: Vec<P> = points.to_vec();
    pts.sort_by(|a, b| a.partial_cmp(b).unwrap());
    pts.dedup_by(|a, b| (a.0 - b.0).abs() < f64::EPSILON && (a.1 - b.1).abs() < f64::EPSILON);
    if pts.len() < 3 {
        return pts;
    }
    let mut lower: Vec<P> = Vec::new();
    for &p in &pts {
        while lower.len() >= 2 && cross(lower[lower.len() - 2], lower[lower.len() - 1], p) <= 0.0 {
            lower.pop();
        }
        lower.push(p);
    }
    let mut upper: Vec<P> = Vec::new();
    for &p in pts.iter().rev() {
        while upper.len() >= 2 && cross(upper[upper.len() - 2], upper[upper.len() - 1], p) <= 0.0 {
            upper.pop();
        }
        upper.push(p);
    }
    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}

fn area(poly: &[P]) -> f64 {
    let n = poly.len();
    let mut sum = 0.0;
    for i in 0..n {
        let (x1, y1) = poly[i];
        let (x2, y2) = poly[(i + 1) % n];
        sum += x1 * y2 - x2 * y1;
    }
    sum.abs() / 2.0
}

fn perimeter(poly: &[P]) -> f64 {
    poly.iter().zip(poly.iter().cycle().skip(1)).map(|(a, b)| ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()).sum()
}

fn contains(poly: &[P], (px, py): P) -> bool {
    let mut inside = false;
    let n = poly.len();
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = poly[i];
        let (xj, yj) = poly[j];
        if (yi > py) != (yj > py) && px < (xj - xi) * (py - yi) / (yj - yi) + xi {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn centroid(poly: &[P]) -> P {
    let n = poly.len() as f64;
    let (sx, sy) = poly.iter().fold((0.0, 0.0), |(ax, ay), &(x, y)| (ax + x, ay + y));
    (sx / n, sy / n)
}

fn main() {
    let cloud = [(0.0, 0.0), (2.0, 0.0), (2.0, 2.0), (0.0, 2.0), (1.0, 1.0), (1.0, 0.5), (2.0, 2.0), (0.5, 1.5), (3.0, 1.0)];
    let hull = convex_hull(&cloud);
    println!("{:?}", hull);
    println!("{:.3} {:.3} {:?}", area(&hull), perimeter(&hull), centroid(&hull));
    for p in [(1.0, 1.0), (2.5, 1.0), (3.0, 1.0), (-0.1, 0.5), (2.9, 1.0)] {
        print!("{} ", contains(&hull, p));
    }
    println!();
    println!("{:?} {:?}", convex_hull(&[(1.0, 1.0), (1.0, 1.0)]), convex_hull(&[]));
    let tri = [(0.0, 0.0), (4.0, 0.0), (0.0, 3.0)];
    println!("{} {} {:?}", area(&tri), perimeter(&tri), centroid(&tri));
    let mut far: Vec<(f64, usize)> = cloud.iter().enumerate().map(|(i, &(x, y))| ((x * x + y * y).sqrt(), i)).collect();
    far.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap().then(a.1.cmp(&b.1)));
    println!("{:?}", far.iter().take(3).map(|(d, i)| format!("{i}:{d:.2}")).collect::<Vec<_>>());
    let angles: Vec<f64> = hull.iter().map(|&(x, y)| (y - 1.0).atan2(x - 1.0).to_degrees()).collect();
    println!("{:?}", angles.iter().map(|a| format!("{a:.1}")).collect::<Vec<_>>());
    let scaled: Vec<P> = hull.iter().map(|&(x, y)| (x * 0.5 + 1.0, y * 0.5 + 1.0)).collect();
    println!("{:.4} {}", area(&scaled), area(&scaled) < area(&hull));
    println!("{}", (1e-17f64 + 1.0 == 1.0) as u8 + (f64::EPSILON / 2.0 + 1.0 == 1.0) as u8);
}
