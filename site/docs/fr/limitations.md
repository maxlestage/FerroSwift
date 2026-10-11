---
title: Limites
description: Ce que FerroSwift ne traduit pas, ou traduit autrement : sémantique de possession, mutabilité partagée, threads, async, code unsafe et autres crates.
order: 5
---

FerroSwift traduit le Rust que vous écrivez pour le modèle, la logique et les vues d’une app. Certaines parties de Rust n’ont pas d’équivalent Swift, ou ne sont pas encore traduites. La plupart sont signalées par un diagnostic ; celles-ci méritent d’être connues avant de commencer.

## Possession, emprunts et durées de vie

rustc vérifie la possession et les durées de vie ; Swift n’en a pas besoin. FerroSwift ignore les durées de vie (`&'a str` donne `String`) et traduit les références comme des valeurs :

- `&T` devient `T`, et un paramètre `&mut T` devient `inout T` : l’appelant voit les modifications, comme en Rust.
- Une référence partagée **stockée** dans une struct (`&'a [i32]`) devient une copie, ce qui ne change rien : Rust interdit de modifier l’original tant qu’il est emprunté. Une référence mutable (`count: &'a mut i32`) devient un `RsMut<Int>` du fichier de support, dont le getter et le setter vont à l’emplacement d’où elle a été prise : `Counter { count: &mut n }` puis `*self.count += 1` modifie `n`, comme en Rust. Une fonction qui garde une telle référence (`fn new(count: &'a mut i32) -> Counter<'a>`) prend un `RsMut` ; une fonction qui garde un paramètre `inout` le temps d’un bloc (`Writer { out: target }`) travaille sur une copie réécrite à la fin du bloc.
- Les déplacements (moves) et `Drop` n’ont pas d’effet : Swift copie les structs et compte les références vers les classes.
- Une méthode qui renvoie une référence mutable dans son receveur (`fn find_mut(&mut self, id) -> Option<&mut Task>`, `-> &mut T`, `-> Result<&mut T, E>`) renvoie à la place un `WritableKeyPath` dans le type, et ses appelants lisent et écrivent `self[keyPath: path]` : `self.find_mut(id)?.done = true`, `if let Some(t) = inv.find_mut(id) { t.qty += 1 }` et `let acct = bank.get(name)?;` modifient l’élément comme en Rust. Le corps doit se terminer par un emplacement de `self` (`&mut self.items[i]`, `self.map.get_mut(&k)`, `….ok_or(e)`, `self.map.entry(k).or_insert(v)`, `self.items.iter_mut().find(…)`) ; une telle méthode doit prendre `&mut self`. Un `&mut` dans une collection obtenu sur place (`v.last_mut().unwrap()`, `map.get_mut(&k)`, `v.iter_mut().find(…)`) modifie aussi l’élément : le nom désigne `v[i]`. Un curseur sur des nœuds chaînés (`let mut cur = &mut self.head;` puis `cur = &mut node.next`, `*cur = Some(…)`, `cur.take()`) est un petit `RsLink` du fichier de support qui retient le nœud et le champ visé, de sorte que les idiomes des listes et arbres de Rust gardent leur sens avec des nœuds que Swift fait classes.

Écrivez le modèle avec des valeurs (structs, enums, `Vec`, `HashMap`) et passez `&mut` aux fonctions qui le modifient : ce code a le même sens dans les deux langages.

## Mutabilité partagée

`Box<T>` et `Rc<T>` deviennent simplement `T` : un `Rc` d’une valeur est copié, ce qui ne change rien tant que la valeur n’est pas modifiée à travers le `Rc`. Un programme qui compte ses références (`Rc::strong_count`, `Rc::weak_count`, `Rc::downgrade`, `Weak<T>`) reçoit à la place `RsRc<T>` et `RsWeak<T>`, qui les comptent comme Rust : un clone en ajoute une, un `drop` ou la fin d’une portée en retire une, la valeur disparaît avec la dernière référence forte, `upgrade()` donne alors `None`, et `{:?}` d’un `Weak` affiche `(Weak)`. `RefCell<T>`, `Cell<T>`, `Mutex<T>` et `RwLock<T>` deviennent `RsRefCell<T>`, une classe du fichier support qui porte la valeur : `Rc::clone`, `borrow_mut()`, `set()`, `lock().unwrap()`, `Rc::ptr_eq` et `{:?}` (`RefCell { value: 1 }`) se comportent comme en Rust, et un `let mut guard = cell.borrow_mut();` désigne la valeur elle-même. Ce qui n’est pas modélisé : les verrous ne bloquent jamais (le programme est mono-thread), un `Mutex` n’est jamais empoisonné, et cloner une structure qui contient un `RefCell` partage la cellule au lieu de la copier. Pour un état partagé entre écrans, un modèle avec `#[derive(Observable)]` devient une classe `@Observable` — voir [SwiftUI en Rust](../swiftui/#etat-et-bindings).

## Concurrence et async

- `async fn` devient `async func` et `.await` devient `await` ; les exécuteurs et bibliothèques async (Tokio, futures) sont d’autres crates, non traduites.
- `std::thread` (`spawn`, `scope`, `sleep`, `yield_now`, `JoinHandle::join`), les canaux `mpsc` (`channel`, `sync_channel`, `send`, `recv`, `try_recv`, `recv_timeout`, l’itération), les atomiques (`AtomicUsize`, `AtomicBool`… avec `load`, `store`, `swap`, `fetch_add` et les autres `fetch_…`, `compare_exchange`), `Arc`, `Mutex` et `RwLock` sont traduits avec des types du fichier de support. Un thread lancé est une tâche exécutée quand quelque chose l’attend : son `join`, un `recv` sur un canal vide, la fin de son `thread::scope`, un `sleep` ou un `yield_now`. Le programme affiche donc ce qu’affiche le programme Rust une fois ses threads rejoints, de façon déterministe et sans course de données ; ce qui diffère, c’est le temps : le travail n’est pas réparti sur les cœurs, et un thread jamais rejoint ni attendu ne s’exécute pas. Un thread qui panique affiche le message de Rust (`thread '<unnamed>' panicked at …`) et s’arrête seul : son `join` renvoie `Err`, et un `thread::scope` dont un thread a paniqué sans être rejoint panique avec `a scoped thread panicked`.

## Les macros

Les macros de la bibliothèque standard sont traduites : `println!`, `eprintln!`, `format!`, `write!`, `vec!`, `assert!`, `assert_eq!`, `matches!`, `panic!`, `todo!`, `unreachable!`, `stringify!`, `concat!`… Vos propres `macro_rules!` sont développées avant la traduction, comme rustc les développe : toutes les sortes de fragments (`expr`, `ident`, `ty`, `tt`, `literal`, `pat`, `block`, `stmt`, `path`, `item`…), les répétitions (`$(…),*`, `+`, `?`, imbriquées), plusieurs règles, la récursion, les usages en expression, en instruction, en élément, dans un bloc `impl` et dans les arguments d’autres macros. Les macros procédurales (le `#[derive(Serialize)]` d’une crate, les macros d’attribut) ne le sont pas : elles appartiennent à d’autres crates.

## Code unsafe, FFI et autres crates

- Les blocs `unsafe` sont traduits comme du code ordinaire, avec un avertissement ; les pointeurs bruts et le FFI n’ont pas d’équivalent.
- Seuls `std` et `ferroswift` sont connus. Les appels vers d’autres crates (`serde`, `regex`, `rand`…) sont recopiés tels quels, avec un avertissement, et ne compileront pas en Swift. Gardez le code traduit sans dépendances, ou derrière vos propres fonctions.

## Différences de comportement

- **Les paniques** sont celles de Rust : `unwrap()` sur `None` ou `Err`, `expect(…)`, un indice au-delà de la fin d’un `Vec`, une tranche hors limites, une clé absente dans `map[&k]`, une division par zéro, une chaîne découpée au milieu d’un caractère, `assert!`, `assert_eq!`, `panic!`, `unreachable!` et `todo!` affichent le message de Rust, avec les mêmes fichier, ligne et colonne (`thread 'main' (…) panicked at src/main.rs:4:13:`), et terminent avec le code 101. `std::panic::catch_unwind(|| …)` rattrape une panique de sa fermeture comme en Rust : le message est affiché et le résultat vaut `Err`.
- Un **dépassement d’entier** arrête un programme Swift dans tous les modes ; Rust panique en debug et boucle en release. `i32` est l’`Int` 64 bits de Swift : `+` déborde à 2⁶³ plutôt qu’à 2³¹ ; `wrapping_add`, `checked_add`, `saturating_add`, `overflowing_add`, `rotate_left`, `count_ones`, `s.parse::<i32>()`, `i32::try_from` et `x as i32` gardent les limites 32 bits. Les conversions `as` gardent le sens de Rust : `n as u8` tronque, un flottant converti en entier sature et `NaN` donne 0. `usize` est aussi `Int` : `usize::MAX` vaut 2⁶³ − 1, ce qui garde son usage de sentinelle (`best == usize::MAX`) mais pas ses chiffres ; une variable ou une fonction dont l’`usize` part d’un littéral au-delà d’`i64::MAX` (le `0xcbf29ce484222325` d’un hachage) est un `UInt64`, de sorte qu’un hachage avec `wrapping_mul`, `%` et `{:x}` donne les résultats de Rust.
- **`Deref` :** `*x` sur une valeur dont le type a une `impl Deref` donne la cible, et `*r` sur une référence `r` vers une telle valeur donne la valeur elle-même (`**r` la cible), comme en Rust. Un trait à paramètres de type devient un protocole à type associé primaire : `dyn Converter<f64>` est `any Converter<Double>`.
- **`i128` et `u128`** deviennent `RsInt128` et `RsUInt128`, des entiers 128 bits du fichier de support (l’`Int128` de Swift demande iOS 18 et macOS 15) : ils fonctionnent dès iOS 17 et macOS 14, avec l’arithmétique, les littéraux, le parsing, l’affichage et les méthodes des autres entiers.
- **Display et largeur :** comme en Rust, `{:>10}` est ignoré par une impl `Display` écrite avec `write!` (une impl qui passe par `f.pad(…)`, `f.pad_integral(…)`, `self.0.fmt(f)` ou les requêtes du formateur l’honore) ; `{:?}` applique une largeur aux nombres et booléens à l’intérieur d’une valeur, pas à ses chaînes. Largeurs et précisions comptent des scalaires, comme Rust compte des `char`.
- Les **tables de hachage et ensembles** sont parcourus dans un ordre non spécifié dans les deux langages, mais pas le même. Une `BTreeMap` est parcourue dans l’ordre des clés.
- Les **chaînes** sont indexées par octets en Rust et par caractères en Swift : le découpage (`&s[0..3]`) passe par une aide qui compte les octets, comme Rust.

## Ce qui est signalé

| Diagnostic | Signification |
| --- | --- |
| error | Le Swift produit est incomplet ou ne compilera pas (construction non prise en charge, macro inconnue…). |
| warning | Le Swift produit compile mais peut se comporter différemment, ou appelle quelque chose que FerroSwift ne connaît pas. |

Le [playground](../../playground/) affiche les diagnostics pendant que vous tapez : c’est le moyen le plus rapide de savoir si un morceau de code se traduit.
