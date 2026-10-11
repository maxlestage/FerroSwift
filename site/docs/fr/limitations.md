---
title: Limites
description: Ce que FerroSwift ne traduit pas, ou traduit autrement : sémantique de possession, mutabilité partagée, threads, async, code unsafe et autres crates.
order: 5
---

FerroSwift traduit le Rust que vous écrivez pour le modèle, la logique et les vues d’une app. Certaines parties de Rust n’ont pas d’équivalent Swift, ou ne sont pas encore traduites. La plupart sont signalées par un diagnostic ; celles-ci méritent d’être connues avant de commencer.

## Possession, emprunts et durées de vie

rustc vérifie la possession et les durées de vie ; Swift n’en a pas besoin. FerroSwift ignore les durées de vie (`&'a str` donne `String`) et traduit les références comme des valeurs :

- `&T` devient `T`, et un paramètre `&mut T` devient `inout T` : l’appelant voit les modifications, comme en Rust.
- Une référence **stockée** dans une struct devient une copie. Les modifications faites à travers elle ne sont pas vues par l’original.
- Les déplacements (moves) et `Drop` n’ont pas d’effet : Swift copie les structs et compte les références vers les classes.
- Une méthode qui renvoie une référence mutable dans son receveur (`fn find_mut(&mut self, id) -> Option<&mut Task>`, `-> &mut T`, `-> Result<&mut T, E>`) renvoie à la place un `WritableKeyPath` dans le type, et ses appelants lisent et écrivent `self[keyPath: path]` : `self.find_mut(id)?.done = true`, `if let Some(t) = inv.find_mut(id) { t.qty += 1 }` et `let acct = bank.get(name)?;` modifient l’élément comme en Rust. Le corps doit se terminer par un emplacement de `self` (`&mut self.items[i]`, `self.map.get_mut(&k)`, `….ok_or(e)`, `self.map.entry(k).or_insert(v)`, `self.items.iter_mut().find(…)`) ; une telle méthode doit prendre `&mut self`. Un `&mut` dans une collection obtenu sur place (`v.last_mut().unwrap()`, `map.get_mut(&k)`, `v.iter_mut().find(…)`) modifie aussi l’élément : le nom désigne `v[i]`. Un curseur sur des nœuds chaînés (`let mut cur = &mut self.head;` puis `cur = &mut node.next`, `*cur = Some(…)`, `cur.take()`) est un petit `RsLink` du fichier de support qui retient le nœud et le champ visé, de sorte que les idiomes des listes et arbres de Rust gardent leur sens avec des nœuds que Swift fait classes.

Écrivez le modèle avec des valeurs (structs, enums, `Vec`, `HashMap`) et passez `&mut` aux fonctions qui le modifient : ce code a le même sens dans les deux langages.

## Mutabilité partagée

`Box<T>` et `Rc<T>` deviennent simplement `T` : un `Rc` d’une valeur est copié, ce qui ne change rien tant que la valeur n’est pas modifiée à travers le `Rc`. `RefCell<T>`, `Cell<T>`, `Mutex<T>` et `RwLock<T>` deviennent `RsRefCell<T>`, une classe du fichier support qui porte la valeur : `Rc::clone`, `borrow_mut()`, `set()`, `lock().unwrap()`, `Rc::ptr_eq` et `{:?}` (`RefCell { value: 1 }`) se comportent comme en Rust, et un `let mut guard = cell.borrow_mut();` désigne la valeur elle-même. Ce qui n’est pas modélisé : les verrous ne bloquent jamais (le programme est mono-thread), un `Mutex` n’est jamais empoisonné, `Rc::strong_count` et `Weak` n’ont pas d’équivalent, et cloner une structure qui contient un `RefCell` partage la cellule au lieu de la copier. Pour un état partagé entre écrans, un modèle avec `#[derive(Observable)]` devient une classe `@Observable` — voir [SwiftUI en Rust](../swiftui/#etat-et-bindings).

## Concurrence et async

- `async fn` devient `async func` et `.await` devient `await` ; les exécuteurs et bibliothèques async (Tokio, futures) sont d’autres crates, non traduites.
- `std::thread` (`spawn`, `scope`, `sleep`, `yield_now`, `JoinHandle::join`), les canaux `mpsc` (`channel`, `sync_channel`, `send`, `recv`, `try_recv`, `recv_timeout`, l’itération), les atomiques (`AtomicUsize`, `AtomicBool`… avec `load`, `store`, `swap`, `fetch_add` et les autres `fetch_…`, `compare_exchange`), `Arc`, `Mutex` et `RwLock` sont traduits avec des types du fichier de support. Un thread lancé est une tâche exécutée quand quelque chose l’attend : son `join`, un `recv` sur un canal vide, la fin de son `thread::scope`, un `sleep` ou un `yield_now`. Le programme affiche donc ce qu’affiche le programme Rust une fois ses threads rejoints, de façon déterministe et sans course de données ; ce qui diffère, c’est le temps : le travail n’est pas réparti sur les cœurs, un thread jamais rejoint ni attendu ne s’exécute pas, et un thread qui panique arrête le programme au lieu de faire renvoyer une erreur à `join`.

## Les macros

Les macros de la bibliothèque standard sont traduites : `println!`, `eprintln!`, `format!`, `write!`, `vec!`, `assert!`, `assert_eq!`, `matches!`, `panic!`, `todo!`, `unreachable!`, `stringify!`, `concat!`… Vos propres `macro_rules!` sont développées avant la traduction, comme rustc les développe : toutes les sortes de fragments (`expr`, `ident`, `ty`, `tt`, `literal`, `pat`, `block`, `stmt`, `path`, `item`…), les répétitions (`$(…),*`, `+`, `?`, imbriquées), plusieurs règles, la récursion, les usages en expression, en instruction, en élément, dans un bloc `impl` et dans les arguments d’autres macros. Les macros procédurales (le `#[derive(Serialize)]` d’une crate, les macros d’attribut) ne le sont pas : elles appartiennent à d’autres crates.

## Code unsafe, FFI et autres crates

- Les blocs `unsafe` sont traduits comme du code ordinaire, avec un avertissement ; les pointeurs bruts et le FFI n’ont pas d’équivalent.
- Seuls `std` et `ferroswift` sont connus. Les appels vers d’autres crates (`serde`, `regex`, `rand`…) sont recopiés tels quels, avec un avertissement, et ne compileront pas en Swift. Gardez le code traduit sans dépendances, ou derrière vos propres fonctions.

## Différences de comportement

- Un **dépassement d’entier** arrête un programme Swift dans tous les modes ; Rust panique en debug et boucle en release. `i32` est l’`Int` 64 bits de Swift : `+` déborde à 2⁶³ plutôt qu’à 2³¹ ; `wrapping_add`, `checked_add`, `saturating_add`, `overflowing_add`, `rotate_left`, `count_ones`, `s.parse::<i32>()`, `i32::try_from` et `x as i32` gardent les limites 32 bits. Les conversions `as` gardent le sens de Rust : `n as u8` tronque, un flottant converti en entier sature et `NaN` donne 0. `usize` est aussi `Int` : `usize::MAX` vaut 2⁶³ − 1.
- **`Deref` :** `*x` sur une valeur dont le type a une `impl Deref` donne la cible (aussi quand `x` est une référence vers une telle valeur : écrire `x.clone()` pour la valeur elle-même). Un trait à paramètres de type ne peut servir de valeur `dyn Converter<f64>` : son type associé n’a pas d’existentiel Swift.
- **`i128` et `u128`** deviennent `Int128` et `UInt128`, que les plateformes Apple fournissent à partir d’iOS 18 et macOS 15.
- **Display et largeur :** comme en Rust, `{:>10}` est ignoré par une impl `Display` écrite avec `write!` (une impl qui passe par `f.pad(…)`, `f.pad_integral(…)`, `self.0.fmt(f)` ou les requêtes du formateur l’honore) ; `{:?}` applique une largeur aux nombres et booléens à l’intérieur d’une valeur, pas à ses chaînes. Largeurs et précisions comptent des scalaires, comme Rust compte des `char`.
- Les **tables de hachage et ensembles** sont parcourus dans un ordre non spécifié dans les deux langages, mais pas le même. Une `BTreeMap` est parcourue dans l’ordre des clés.
- Les **chaînes** sont indexées par octets en Rust et par caractères en Swift : le découpage (`&s[0..3]`) passe par une aide qui compte les octets, comme Rust.

## Ce qui est signalé

| Diagnostic | Signification |
| --- | --- |
| error | Le Swift produit est incomplet ou ne compilera pas (construction non prise en charge, macro inconnue…). |
| warning | Le Swift produit compile mais peut se comporter différemment, ou appelle quelque chose que FerroSwift ne connaît pas. |

Le [playground](../../playground/) affiche les diagnostics pendant que vous tapez : c’est le moyen le plus rapide de savoir si un morceau de code se traduit.
