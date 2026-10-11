---
title: Rust → Swift
description: Le sous-ensemble de Rust pris en charge et ce qu’il devient en Swift : types, structs, enums, traits, génériques, closures, filtrage, erreurs et formatage.
order: 3
---

FerroSwift traduit du Rust ordinaire — le code que vous écrivez pour le modèle et la logique d’une app — en Swift tel que vous l’écririez à la main. Les types sont suivis expression par expression, pour que chaque construction reçoive le bon Swift : `Int` ou `Double`, un optionnel, un `throws`, un binding. Essayez tout ceci dans le [playground](../../playground/).

## Les types

| Rust | Swift |
| --- | --- |
| `i32`, `i64`, `isize`, `usize` | `Int` |
| `i8`, `i16`, `u8`, `u16`, `u32`, `u64` | `Int8`, `Int16`, `UInt8`, `UInt16`, `UInt32`, `UInt64` |
| `f64`, `f32` | `Double`, `Float` |
| `bool`, `char` | `Bool`, `Character` |
| `String`, `&str` | `String` |
| `Vec<T>`, `[T; N]`, `&[T]` | `[T]` |
| `HashMap<K, V>`, `BTreeMap<K, V>` | `[K: V]` (une `BTreeMap` est parcourue dans l’ordre des clés) |
| `HashSet<T>`, `BTreeSet<T>` | `Set<T>` |
| `HashMap<(A, B), V>`, `HashSet<(A, B)>` | `RsTupleMap2<A, B, V>`, `RsTupleSet2<A, B>` : des types de support aux mêmes méthodes, Swift ne pouvant pas hacher un tuple (paires et triplets) |
| `Option<T>` | `T?` |
| `Result<T, E>` | une fonction qui `throws(E)` et renvoie `T` |
| `(A, B)`, `()` | `(A, B)`, `Void` |
| `(A,)` | `RsTuple1<A>`, une struct du fichier de support (Swift n’a pas de tuple à un élément) |
| `BinaryHeap<T>` | `RsBinaryHeap<T>`, un tas du fichier de support, disposé comme celui de Rust ; un tuple dans le tas, ou dans un `Reverse`, est rangé en `RsOrd2` et rendu comme tuple |
| `Box<T>`, `Rc<T>`, `Arc<T>`, `Cow<T>` | `T` |
| `RefCell<T>`, `Cell<T>`, `Mutex<T>`, `RwLock<T>` | `RsRefCell<T>`, une classe qui porte la valeur : les clones d’un `Rc` la partagent |
| `&T`, `&mut T` | `T` (`inout T` pour un paramètre `&mut` ; une méthode qui renvoie `&mut T` renvoie un `WritableKeyPath<Self, T>`) |
| `impl Trait`, `dyn Trait` | `some Proto`, `any Proto` |
| `fn(A) -> B`, closures | `(A) -> B` |

Les longueurs `usize` sont des `Int` Swift : `v.len()` donne `v.count`. Le `len()` d’une chaîne compte les octets UTF-8, comme en Rust : `s.utf8.count`.

## Les éléments

- Les **structs** deviennent des `struct` avec leur initialiseur membre à membre ; les structs tuples reçoivent des champs `_0`, `_1`… `#[derive(Default)]` donne à chaque champ sa valeur par défaut. Une struct qui se contient elle-même (`next: Option<Box<Node>>`) devient une `final class`, un type valeur Swift ne pouvant pas se contenir ; son `clone()` copie toute la structure et `==` la compare.
- **Derives :** `PartialEq`/`Eq` → `Equatable`, `Hash` → `Hashable`, `PartialOrd`/`Ord` → `Comparable`. `Clone` et `Copy` n’ont besoin de rien : les structs Swift sont des valeurs. `Debug` est pris en charge par `{:?}` (voir plus bas). Pour les enums à données, `==`, `hash(into:)` et `<` sont écrits quand Swift ne peut pas les synthétiser : les variantes se comparent dans l’ordre de déclaration, puis champ par champ, comme avec les derives de Rust.
- Les **enums** deviennent des `enum` ; les variantes à données reçoivent des valeurs associées (`Card { last4: String }` → `case card(last4: String)`). Modifier les champs d’une variante par un `match` sur un emplacement (`Shape::Circle { r } => *r *= k`) range la variante à la fin du bras.
- Les blocs **`impl`** deviennent des `extension`. `fn new(…) -> Self` devient un `init`, les méthodes `&mut self` des `mutating func`, les `const` associées des `static let`. Une méthode qui renvoie `&mut Self` (un builder) est appelée comme une instruction : `a.b().c()` devient `a.b()` puis `a.c()`. Une méthode qui porte le nom d’un champ (`fn count(&self)` à côté de `count`) devient une propriété calculée, le champ étant stocké sous `_count`.
- **`impl Iterator for Type`** fait du type une `Sequence` : `for x in Counter { … }`, `.filter`, `.sum()` fonctionnent dessus. Ses adaptateurs sont paresseux, comme ceux de `(1..)`, donc un itérateur sans fin s’arrête avec `take(5)` ; `let mut it = …` garde un itérateur que `next()`, `peek()` et `nth()` font avancer.
- Les **traits** deviennent des `protocol` ; les méthodes par défaut vont dans une extension du protocole, les types associés dans des `associatedtype`. `impl Trait for Type` ajoute la conformité.
- Les **traits d’opérateurs** (`Add`, `Sub`, `Mul`, `Neg`…) deviennent des fonctions d’opérateur, `impl Index` un `subscript` (avec un `set` quand `IndexMut` est implémenté aussi) ; `impl Display` devient `CustomStringConvertible`.
- Les **bornes génériques** deviennent des contraintes de protocole (`T: Display` → `T: CustomStringConvertible`, `T: PartialOrd` → `T: Comparable`) ; `T: Default` et `T: Into<f64>` passent par deux petits protocoles du fichier de support (`RsDefault`, `RsIntoDouble`), que les types standard adoptent, et une structure générique qui dérive `Default` reçoit son `init()` sous `where T: RsDefault`.
- Les **noms** suivent les conventions de Swift (`my_field` → `myField`, `MAX_SIZE` → `maxSize`) ; un type nommé comme l’un de ceux de Swift (`enum Error`, `struct Task`) reçoit le suffixe `Type` (`ErrorType`), et `type Result<T> = std::result::Result<T, Error>` devient un `typealias` générique.
- **Conversions :** `impl From<A> for B` devient un `init` ; `impl FromStr for B` et `impl TryFrom<A> for B` deviennent les fonctions statiques `fromStr` et `tryFrom`, qu’appellent `s.parse::<B>()`, `B::from_str(s)`, `B::try_from(x)` et `x.try_into()`.
- Les **génériques** gardent leurs bornes : `fn largest<T: Describe>(items: &[T])` → `func largest<T: Describe>(_ items: [T])`. Les clauses `where` sont conservées.
- Les **fonctions** prennent des paramètres sans étiquette (`_ name: Type`), pour que les appels ressemblent à ceux de Rust.
- `const` et `static` deviennent des `let` globaux, les alias `type` des `typealias`, les blocs `mod` sont mis à plat.

## Expressions et contrôle de flux

| Rust | Swift |
| --- | --- |
| `match x { A => …, B(v) if v > 0 => …, _ => … }` | `switch x { case .a: … case .b(let v) where v > 0: … default: … }` |
| `if let Some(x) = opt { … }` | `if let x = opt { … }` |
| `let Some(x) = opt else { return };` | `guard let x = opt else { return }` |
| `while let Some(x) = stack.pop()` | `while let x = stack.popLast()` |
| `for (i, x) in v.iter().enumerate()` | `for (i, x) in v.enumerated()` |
| `for i in 0..n`, `0..=n`, `(0..n).rev()` | `for i in 0..<n`, `0...n`, `(0..<n).reversed()` |
| `if c { a } else { b }` comme valeur | `c ? a : b` |
| `v.push(match op { '+' => a + b, … })` | `v.append({ () -> Int in switch op { … } }())` |
| `x as f64`, `n as u8` | `Double(x)`, `UInt8(truncatingIfNeeded: n)` |
| `matches!(x, Some(1..=5))` | un `switch` qui renvoie un `Bool` |
| `assert!(c, "…")`, `panic!("…")` | `precondition(c, "…")`, `fatalError("…")` |

Les closures deviennent des closures Swift (`|x| x * 2` → `{ x in x * 2 }`), et les chaînes d’itérateurs les méthodes Swift correspondantes : `map`, `filter`, `fold` (→ `reduce`), `sum`, `any` (→ `contains(where:)`), `find` (→ `first(where:)`), `position` (→ `firstIndex`), `enumerate`, `zip`, `rev`, `max_by`, `collect`… Les quelques méthodes sans équivalent direct viennent du fichier de support.

## Les erreurs

Une fonction qui renvoie `Result<T, E>` devient une fonction qui `throws(E)` ; `?` devient `try`, `Err(e)` un `throw`, et un `match` sur un `Result` attrape l’erreur. Une erreur qui est une `String`, un nombre, un `bool` ou un `char` est levée telle quelle (le fichier de support fait du type une `Error`). Un `main` qui renvoie une `Err` affiche `Error: {:?}` sur la sortie d’erreur et termine avec le code 1, comme en Rust :

```rust
fn check(name: &str) -> Result<&str, LoginError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(LoginError::EmptyName);
    }
    Ok(name)
}
```

```swift
func check(_ name: String) throws(LoginError) -> String {
    let name = name.trimmingCharacters(in: .whitespacesAndNewlines)
    if name.isEmpty {
        throw .emptyName
    }
    return name
}
```

Les enums d’erreur se conforment à `Error`. Le `throws` typé demande Swift 6 (Xcode 16).

`s.parse::<i32>()` (la cible peut aussi venir d’un usage ultérieur de la valeur, comme `let n = s.parse().ok()?;` suivi de `Point { x: n }`) et les autres conversions de la bibliothèque standard (`i32::from_str_radix`, `u8::try_from`, `x.try_into()`, `String::from_utf8`, `char::try_from`) renvoient des `Result` dont les erreurs sont celles de Rust, en `{:?}` comme en `{}` : `Err(ParseIntError { kind: InvalidDigit })`, `invalid digit found in string`, `PosOverflow` pour un nombre trop grand pour le type, `TryFromIntError(NegOverflow)`. Un `?` sur une erreur qui se convertit en celle de la fonction (`impl From<ParseIntError> for AppError`) la convertit de la même façon. `v.last_mut().unwrap().done = true`, `map.get_mut(&k).unwrap().push(x)`, `if let Some(t) = v.iter_mut().find(…)` et `v.iter_mut().for_each(…)` modifient l’élément sur place, comme en Rust.

## Chaînes et formatage

`format!`, `println!`, `eprintln!` et `write!` deviennent de l’interpolation de chaînes. Les valeurs s’affichent exactement comme en Rust grâce à de petites fonctions du fichier de support : `{}` de `1.0` donne `1`, `{:.2}` arrondit de la même façon, `{:?}` écrit `Some(Point { x: 1.0, y: 2.5 })`, et largeurs, alignements et remplissages (`{:>8}`, `{:05}`) sont conservés — dans `{:>7?}` ils atteignent les nombres à l’intérieur de la valeur, comme avec le `Debug` dérivé de Rust. Les méthodes de chaînes courantes ont leur équivalent Swift (`trim` → `trimmingCharacters(in:)`, `split` → `components(separatedBy:)`, `to_uppercase` → `uppercased()`, `starts_with` → `hasPrefix`…). `s.chars()` parcourt les scalaires Unicode, comme Rust (`s.rsChars`) : `chars().count()`, `rev()` et `nth()` donnent le résultat de Rust sur les accents combinants et les séquences d’emojis, que le parcours propre à Swift regroupe en un seul caractère. Le motif de `starts_with`, `split`, `find`, `trim_matches`… peut être un texte, un caractère, une fermeture ou un tableau de caractères ; `s.get(a..b)` vaut `None` hors d’une frontière de caractère ; `to_ascii_uppercase` ne change que les lettres ASCII, comme en Rust.

## Les commentaires

Les commentaires et la documentation `///` sont conservés, à côté du code qu’ils décrivent : le Swift généré reste lisible. Avant et après les instructions, entre les enfants et les modificateurs d’une vue, et les lignes vides entre groupes d’instructions restent. Les commentaires placés au milieu d’une autre chaîne d’appels de méthodes (`v.iter() // … .map(…)`) ne sont pas conservés.

## Le fichier de support

Quand la traduction a besoin d’une aide — formatage Rust, `Ordering`, découpage de chaînes par octets, quelques méthodes de collections (`retain`, `dedup`, `windows`…) — FerroSwift l’écrit une fois dans `FerroSwiftSupport.swift` (ou à la fin du fichier dans le playground). Seules les aides utilisées sont incluses.
