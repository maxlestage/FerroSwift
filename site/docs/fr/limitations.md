---
title: Limites
description: Ce que FerroSwift ne traduit pas (encore) : sémantique de possession, mutabilité partagée, async et threads, macros, code unsafe et autres crates.
order: 5
---

FerroSwift traduit le Rust que vous écrivez pour le modèle, la logique et les vues d’une app. Certaines parties de Rust n’ont pas d’équivalent Swift, ou ne sont pas encore traduites. La plupart sont signalées par un diagnostic ; celles-ci méritent d’être connues avant de commencer.

## Possession, emprunts et durées de vie

rustc vérifie la possession et les durées de vie ; Swift n’en a pas besoin. FerroSwift ignore les durées de vie (`&'a str` donne `String`) et traduit les références comme des valeurs :

- `&T` devient `T`, et un paramètre `&mut T` devient `inout T` : l’appelant voit les modifications, comme en Rust.
- Une référence **stockée** dans une struct devient une copie. Les modifications faites à travers elle ne sont pas vues par l’original.
- Les déplacements (moves) et `Drop` n’ont pas d’effet : Swift copie les structs et compte les références vers les classes.
- Une méthode qui renvoie une référence mutable dans son receveur (`fn find_mut(&mut self, id) -> Option<&mut Task>`) est signalée : renvoyez l’indice de la tâche et modifiez `self.tasks[i]` à la place. Un `&mut` dans une collection obtenu sur place (`v.last_mut().unwrap()`, `map.get_mut(&k)`, `v.iter_mut().find(…)`) modifie bien l’élément : le nom désigne `v[i]`.

Écrivez le modèle avec des valeurs (structs, enums, `Vec`, `HashMap`) et passez `&mut` aux fonctions qui le modifient : ce code a le même sens dans les deux langages.

## Mutabilité partagée

`Rc<RefCell<T>>`, `Arc<Mutex<T>>`, `Cell<T>` et `Box<T>` deviennent simplement `T`. Le partage est perdu : deux clones `Rc` d’une même valeur deviennent deux copies indépendantes, et FerroSwift avertit à chaque `RefCell::new`, `Cell::new`, `Mutex::new` et `RwLock::new`. Pour un état partagé entre écrans, utilisez un modèle avec `#[derive(Observable)]`, qui devient une classe `@Observable` (un type référence) — voir [SwiftUI en Rust](../swiftui/#etat-et-bindings).

## Concurrence et async

- `async fn` devient `async func` et `.await` devient `await`, mais les exécuteurs et bibliothèques async (Tokio, futures, canaux) ne sont pas transposés.
- `std::thread`, `Mutex`, les atomiques et les canaux ne sont pas traduits : leurs appels sont signalés comme fonctions inconnues.

## Les macros

Les macros de la bibliothèque standard sont traduites : `println!`, `eprintln!`, `format!`, `write!`, `vec!`, `assert!`, `assert_eq!`, `matches!`, `panic!`, `todo!`, `unreachable!`… Vos propres `macro_rules!` et macros procédurales ne le sont pas : leurs définitions sont ignorées et leurs utilisations signalées comme erreurs.

## Code unsafe, FFI et autres crates

- Les blocs `unsafe` sont traduits comme du code ordinaire, avec un avertissement ; les pointeurs bruts et le FFI n’ont pas d’équivalent.
- Seuls `std` et `ferroswift` sont connus. Les appels vers d’autres crates (`serde`, `regex`, `rand`…) sont recopiés tels quels, avec un avertissement, et ne compileront pas en Swift. Gardez le code traduit sans dépendances, ou derrière vos propres fonctions.

## Différences de comportement

- Un **dépassement d’entier** arrête un programme Swift dans tous les modes ; Rust panique en debug et boucle en release. `i32` est l’`Int` 64 bits de Swift : `+` déborde à 2⁶³ plutôt qu’à 2³¹ ; `wrapping_add`, `checked_add`, `saturating_add`, `overflowing_add`, `rotate_left`, `count_ones`, `s.parse::<i32>()`, `i32::try_from` et `x as i32` gardent les limites 32 bits. Les conversions `as` gardent le sens de Rust : `n as u8` tronque, un flottant converti en entier sature et `NaN` donne 0. `usize` est aussi `Int` : `usize::MAX` vaut 2⁶³ − 1.
- **Display et largeur :** comme en Rust, `{:>10}` est ignoré par une impl `Display` écrite avec `write!` (seuls `f.pad(…)` et `self.0.fmt(f)` l’honorent) ; `{:?}` applique une largeur aux nombres et booléens à l’intérieur d’une valeur, pas à ses chaînes.
- Les **tables de hachage et ensembles** sont parcourus dans un ordre non spécifié dans les deux langages, mais pas le même. Une `BTreeMap` est parcourue dans l’ordre des clés.
- Les **chaînes** sont indexées par octets en Rust et par caractères en Swift : le découpage (`&s[0..3]`) passe par une aide qui compte les octets, comme Rust.

## Ce qui est signalé

| Diagnostic | Signification |
| --- | --- |
| error | Le Swift produit est incomplet ou ne compilera pas (construction non prise en charge, macro inconnue…). |
| warning | Le Swift produit compile mais peut se comporter différemment, ou appelle quelque chose que FerroSwift ne connaît pas. |

Le [playground](../../playground/) affiche les diagnostics pendant que vous tapez : c’est le moyen le plus rapide de savoir si un morceau de code se traduit.
