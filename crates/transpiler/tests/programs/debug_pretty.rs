use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
enum Shape {
    Dot,
    Circle { r: f64 },
    Rect(f64, f64),
    Group(Vec<Shape>, Option<Box<Shape>>),
}

#[derive(Debug)]
struct Unit;

#[derive(Debug)]
struct Meters(f32);

#[derive(Debug, Default)]
struct Scene {
    name: String,
    shapes: Vec<Shape>,
    tags: BTreeMap<String, Vec<(u8, char)>>,
    origin: (i8, i8),
    note: Option<Option<&'static str>>,
    unit: Unit,
    scale: Meters,
    flags: [bool; 2],
    nested: Vec<Vec<i64>>,
}

impl Default for Unit {
    fn default() -> Self {
        Unit
    }
}

impl Default for Meters {
    fn default() -> Self {
        Meters(1.0)
    }
}

fn main() {
    let mut scene = Scene { name: "demo \"quoted\"\n".into(), ..Default::default() };
    scene.shapes.push(Shape::Dot);
    scene.shapes.push(Shape::Circle { r: 0.1 + 0.2 });
    scene.shapes.push(Shape::Rect(1e21, -0.0));
    scene.shapes.push(Shape::Group(vec![Shape::Dot, Shape::Rect(2.0, 3.5)], Some(Box::new(Shape::Circle { r: 1.0 }))));
    scene.shapes.push(Shape::Group(vec![], None));
    scene.tags.insert("alpha".into(), vec![(1, 'a'), (2, '\'')]);
    scene.tags.insert("".into(), vec![]);
    scene.origin = (-128, 127);
    scene.note = Some(None);
    scene.flags = [true, false];
    scene.nested = vec![vec![], vec![i64::MIN, 0, i64::MAX]];
    println!("{:?}", scene);
    println!("{:#?}", scene);
    println!("{:?} {:?} {:?}", Unit, Meters(2.5), Shape::Rect(1.0, 2.0));
    println!("{:#?}", (Unit, Meters(0.0), Some(Some(3))));
    println!("{:#?}", Scene::default());
    println!("{:?}", scene.shapes[3] == Shape::Group(vec![Shape::Dot, Shape::Rect(2.0, 3.5)], Some(Box::new(Shape::Circle { r: 1.0 }))));
    println!("{:?} {:?}", scene.note, Some(Some("x")));
    println!("{:#?}", vec![("k", 1u8)].into_iter().collect::<BTreeMap<_, _>>());
    println!("{:?} {:?} {:?}", '\u{7f}', "tab\tnull\0bell\x07é", '\u{200b}');
    println!("{:#?}", [Some(1.5f32), None]);
}
