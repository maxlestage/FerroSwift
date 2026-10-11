//! Interface texts in French and English.

use crate::routes::Lang;

pub struct Dict {
    pub lang_name: &'static str,
    pub locale: &'static str,
    pub skip: &'static str,
    pub nav: Nav,
    pub home: Home,
    pub playground: PlaygroundPage,
    pub docs: DocsText,
    pub not_found: NotFound,
    pub offline: Offline,
    pub footer: Footer,
}

pub struct Nav {
    pub playground: &'static str,
    pub docs: &'static str,
    pub main: &'static str,
    pub language: &'static str,
    pub switch_to: &'static str,
}

pub struct Item {
    pub title: &'static str,
    pub text: &'static str,
    /// A short code token illustrating the item.
    pub code: &'static str,
}

pub struct Section {
    pub kicker: &'static str,
    pub title: &'static str,
    pub text: &'static str,
}

pub struct Home {
    pub title: &'static str,
    pub description: &'static str,
    pub eyebrow: &'static str,
    pub hero: [&'static str; 2],
    pub lead: &'static str,
    pub cta_play: &'static str,
    pub cta_start: &'static str,
    pub hero_rust: &'static str,
    pub hero_swift: &'static str,
    pub play: Section,
    pub features: Section,
    pub items: [Item; 9],
    pub how: Section,
    pub steps: [Item; 5],
    pub compare: Section,
    pub tabs: [&'static str; 4],
    pub tab_notes: [&'static str; 4],
    pub install: Section,
    pub commands: [Item; 4],
    pub install_more: &'static str,
}

pub struct PlaygroundPage {
    pub title: &'static str,
    pub description: &'static str,
    pub lead: &'static str,
    pub tips_title: &'static str,
    pub tips: [&'static str; 5],
}

pub struct DocsText {
    pub title: &'static str,
    pub description: &'static str,
    pub intro: &'static str,
    pub menu: &'static str,
    pub on_this_page: &'static str,
    pub prev: &'static str,
    pub next: &'static str,
    pub copy: &'static str,
}

pub struct NotFound {
    pub title: &'static str,
    pub text: &'static str,
    pub back: &'static str,
}

/// The page the service worker shows without network, when a page is not cached.
pub struct Offline {
    pub title: &'static str,
    pub text: &'static str,
}

pub struct Footer {
    pub tagline: &'static str,
    /// The call to action, followed by the project name: "Start building with *FerroSwift*".
    pub start: &'static str,
    /// Accessible name of the install command chip.
    pub install: &'static str,
    pub rust_only: &'static str,
    pub product: &'static str,
    pub project: &'static str,
    pub made_with: &'static str,
    pub license: &'static str,
    /// The sister projects: title, one-line introduction, the "you are here" mark and one
    /// line per project, in the order of `pages::ECOSYSTEM`.
    pub ecosystem: &'static str,
    pub ecosystem_text: &'static str,
    pub here: &'static str,
    pub projects: [&'static str; 6],
    /// The signature: "Designed and built by", then where it was made.
    pub by: &'static str,
    pub crafted: &'static str,
    pub rights: &'static str,
    pub proprietary: &'static str,
    pub top: &'static str,
}

pub fn t(lang: Lang) -> &'static Dict {
    match lang {
        Lang::Fr => &FR,
        Lang::En => &EN,
    }
}

static FR: Dict = Dict {
    lang_name: "Français",
    locale: "fr_FR",
    skip: "Aller au contenu",
    nav: Nav {
        playground: "Playground",
        docs: "Documentation",
        main: "Navigation principale",
        language: "Langue",
        switch_to: "Read in English",
    },
    home: Home {
        title: "FerroSwift — écrivez du Rust, livrez du SwiftUI",
        description: "FerroSwift traduit votre app Rust — la logique et les vues SwiftUI — en Swift idiomatique à ouvrir dans Xcode. Vérifiée par rustc, à essayer dans le navigateur.",
        eyebrow: "Transpileur Rust → Swift · 100 % Rust",
        hero: ["Écrivez du Rust.", "Livrez du SwiftUI."],
        lead: "Écrivez toute votre app iOS ou macOS en Rust — la logique et les vues SwiftUI. rustc la vérifie, FerroSwift la traduit en Swift idiomatique que vous ouvrez dans Xcode.",
        cta_play: "Essayer le playground",
        cta_start: "Commencer",
        hero_rust: "hello.rs · vérifié par rustc",
        hero_swift: "Hello.swift · généré par FerroSwift",
        play: Section {
            kicker: "Playground",
            title: "Traduit pendant que vous tapez.",
            text: "Le traducteur tourne ici, dans votre navigateur : le même code Rust, compilé en WebAssembly. Modifiez l’exemple, le Swift suit.",
        },
        features: Section {
            kicker: "Fonctionnalités",
            title: "Le Rust que vous connaissez, le Swift que vous auriez écrit.",
            text: "FerroSwift ne cache rien derrière un runtime : le résultat est du Swift ordinaire, lisible, que vous pouvez relire et garder.",
        },
        items: [
            Item {
                title: "Du Swift idiomatique",
                text: "Des structs, enums, protocoles et extensions comme on les écrit à la main. Pas de runtime ni de pont : juste du Swift.",
                code: "impl → extension",
            },
            Item {
                title: "Des vues SwiftUI en Rust",
                text: "vstack(), text(), button() et leurs modificateurs se combinent comme des builders et deviennent VStack, Text et Button.",
                code: "vstack().padding()",
            },
            Item {
                title: "État et bindings",
                text: "#[state] devient @State et &mut self.x devient $x : les liaisons SwiftUI, vérifiées par le borrow checker.",
                code: "#[state] → @State",
            },
            Item {
                title: "Erreurs et optionnels",
                text: "Result devient throws, ? devient try, Option un optionnel, match un switch et let-else un guard.",
                code: "Result → throws",
            },
            Item {
                title: "Un formatage identique à Rust",
                text: "{}, {:?} et {:.2} affichent exactement la même chose qu’en Rust, grâce à un petit fichier de support.",
                code: "{:?} → rsDebug",
            },
            Item {
                title: "Commentaires conservés",
                text: "Les commentaires et la documentation /// suivent le code jusque dans Swift : le résultat reste lisible.",
                code: "/// doc",
            },
            Item {
                title: "Des diagnostics précis",
                text: "Ce qui ne se traduit pas est signalé avec sa ligne et sa colonne, comme une erreur du compilateur.",
                code: "12:5: error",
            },
            Item {
                title: "Une CLI avec mode watch",
                text: "ferroswift build traduit le crate, ferroswift watch retraduit à chaque sauvegarde pendant que vous codez.",
                code: "ferroswift watch",
            },
            Item {
                title: "Dans votre navigateur",
                text: "Le playground, c’est le transpileur compilé en WebAssembly : votre code ne quitte jamais votre machine.",
                code: "wasm32",
            },
        ],
        how: Section {
            kicker: "Fonctionnement",
            title: "Du crate Rust au projet Xcode.",
            text: "Une chaîne courte et transparente : chaque étape produit quelque chose que vous pouvez lire.",
        },
        steps: [
            Item {
                title: "Source Rust",
                text: "Votre crate, vérifié par cargo check et rust-analyzer : types, emprunts, tout.",
                code: "src/*.rs",
            },
            Item {
                title: "AST syn",
                text: "Chaque fichier est analysé par syn, avec la position exacte de chaque nœud.",
                code: "syn::File",
            },
            Item {
                title: "Traduction typée",
                text: "Les types sont suivis expression par expression pour choisir le bon Swift : Int ou Double, optionnel, throws, binding.",
                code: "Ty::Option",
            },
            Item {
                title: "Fichiers Swift",
                text: "Un fichier .swift par fichier .rs, plus un petit fichier de support si nécessaire.",
                code: "Sources/*.swift",
            },
            Item {
                title: "Xcode",
                text: "Ajoutez le dossier généré à une cible iOS, ou lancez swift run sur macOS.",
                code: "⌘R",
            },
        ],
        compare: Section {
            kicker: "Côte à côte",
            title: "Le même programme, deux langages.",
            text: "Ces traductions sont produites par FerroSwift au moment où ce site est généré : elles sont toujours exactes.",
        },
        tabs: ["Vue SwiftUI", "Enums et match", "Erreurs", "Formatage"],
        tab_notes: [
            "#[state] devient @State, &mut self.name devient $name, et les modificateurs restent dans l’ordre.",
            "Les enums à données deviennent des enums à valeurs associées, match devient switch, les gardes deviennent where.",
            "Result<T, E> devient throws(E), ? devient try, et let-else devient guard.",
            "{:?} et {:.2} passent par de petites fonctions de support pour afficher exactement comme Rust.",
        ],
        install: Section {
            kicker: "Démarrer",
            title: "Quatre commandes.",
            text: "Il vous faut Rust pour écrire l’app, et Xcode (ou Swift sur macOS) pour la lancer.",
        },
        commands: [
            Item {
                title: "Installez la CLI",
                text: "Depuis crates.io, avec cargo.",
                code: "cargo install --git https://github.com/maxlestage/FerroSwift ferroswift-cli",
            },
            Item {
                title: "Créez une app",
                text: "Un crate Rust prêt à l’emploi, avec une première vue.",
                code: "ferroswift new MyApp",
            },
            Item {
                title: "Traduisez en Swift",
                text: "Les fichiers Swift sont écrits à côté du crate.",
                code: "ferroswift build",
            },
            Item {
                title: "Gardez-le à jour",
                text: "Chaque sauvegarde retraduit les fichiers modifiés.",
                code: "ferroswift watch",
            },
        ],
        install_more: "Lire le guide de démarrage",
    },
    playground: PlaygroundPage {
        title: "Playground",
        description: "Écrivez du Rust, lisez le Swift et le SwiftUI qu’en tire FerroSwift, en direct dans votre navigateur grâce à WebAssembly.",
        lead: "Écrivez du Rust à gauche, lisez le Swift à droite. La traduction se fait dans votre navigateur, par FerroSwift compilé en WebAssembly : rien ne quitte votre machine.",
        tips_title: "Bon à savoir",
        tips: [
            "Cinq exemples pour commencer : un compteur et une liste de tâches en SwiftUI, des formes (enums, match, traits), des erreurs (Result et ?) et des itérateurs.",
            "Tab insère quatre espaces, Maj+Tab les retire ; Échap puis Tab sort de l’éditeur.",
            "Cliquez sur un diagnostic pour sélectionner la ligne concernée.",
            "Le code de support (formatage à la Rust, petites extensions) n’est ajouté que si la traduction en a besoin.",
            "Le sous-ensemble de Rust pris en charge est décrit dans la documentation : Langage, SwiftUI et Limites.",
        ],
    },
    docs: DocsText {
        title: "Documentation",
        description: "Tout pour écrire une app SwiftUI en Rust avec FerroSwift : installation, vues, sous-ensemble de Rust pris en charge, CLI et limites.",
        intro: "Tout pour écrire une app SwiftUI en Rust avec FerroSwift : l’installation, les vues, le sous-ensemble de Rust pris en charge, la CLI et les limites actuelles.",
        menu: "Pages de la documentation",
        on_this_page: "Sur cette page",
        prev: "Précédent",
        next: "Suivant",
        copy: "Copier",
    },
    not_found: NotFound {
        title: "Page introuvable",
        text: "Cette page n’existe pas, ou plus. Le playground et la documentation, eux, sont toujours là.",
        back: "Retour à l’accueil",
    },
    offline: Offline {
        title: "Vous êtes hors ligne",
        text: "Cette page n’est pas encore enregistrée sur cet appareil. L’accueil, le playground et la documentation, eux, restent disponibles sans réseau.",
    },
    footer: Footer {
        tagline: "Écrivez du Rust. Livrez du SwiftUI.",
        start: "Commencez avec",
        install: "S'installe en une ligne",
        rust_only: "100 % Rust",
        product: "Produit",
        project: "Projet",
        made_with: "Site fait avec Active",
        license: "Licence",
        ecosystem: "Écosystème",
        ecosystem_text: "Six projets Rust d'un même auteur, faits pour travailler ensemble : du front-end à la base de données, et jusqu'aux apps iPhone.",
        here: "Vous êtes ici",
        projects: [
            "Front-end : composants, SSR, îlots WebAssembly",
            "Serveur web façon Express, HTTP/3",
            "Middlewares de sécurité pour Vitesse et tower",
            "ORM asynchrone typé, SQL et NoSQL",
            "Le framework full-stack qui les réunit",
            "Des apps SwiftUI écrites en Rust",
        ],
        by: "Conçu et développé par",
        crafted: "Fabriqué en Rust, en France",
        rights: "© 2026 Maxime Nathan Lestage. Tous droits réservés.",
        proprietary: "Logiciel propriétaire",
        top: "Haut de page",
    },
};

static EN: Dict = Dict {
    lang_name: "English",
    locale: "en_US",
    skip: "Skip to content",
    nav: Nav {
        playground: "Playground",
        docs: "Docs",
        main: "Main navigation",
        language: "Language",
        switch_to: "Lire en français",
    },
    home: Home {
        title: "FerroSwift — write Rust, ship SwiftUI",
        description: "FerroSwift translates your Rust app — logic and SwiftUI views — into idiomatic Swift you open in Xcode. Type-checked by rustc, try it in your browser.",
        eyebrow: "Rust → Swift transpiler · 100 % Rust",
        hero: ["Write Rust.", "Ship SwiftUI."],
        lead: "Write your whole iOS or macOS app in Rust — the logic and the SwiftUI views. rustc type-checks it, FerroSwift translates it into idiomatic Swift you open in Xcode.",
        cta_play: "Try the playground",
        cta_start: "Get started",
        hero_rust: "hello.rs · checked by rustc",
        hero_swift: "Hello.swift · written by FerroSwift",
        play: Section {
            kicker: "Playground",
            title: "Translated as you type.",
            text: "The translator runs right here, in your browser: the same Rust code, compiled to WebAssembly. Edit the example, the Swift follows.",
        },
        features: Section {
            kicker: "Features",
            title: "The Rust you know, the Swift you would have written.",
            text: "FerroSwift hides nothing behind a runtime: the result is ordinary, readable Swift you can review and keep.",
        },
        items: [
            Item {
                title: "Idiomatic Swift",
                text: "Structs, enums, protocols and extensions as you would write them by hand. No runtime, no bridge: just Swift.",
                code: "impl → extension",
            },
            Item {
                title: "SwiftUI views from Rust",
                text: "vstack(), text(), button() and their modifiers chain like builders and become VStack, Text and Button.",
                code: "vstack().padding()",
            },
            Item {
                title: "State and bindings",
                text: "#[state] becomes @State and &mut self.x becomes $x: SwiftUI bindings, checked by the borrow checker.",
                code: "#[state] → @State",
            },
            Item {
                title: "Errors and optionals",
                text: "Result becomes throws, ? becomes try, Option an optional, match a switch and let-else a guard.",
                code: "Result → throws",
            },
            Item {
                title: "Formatting identical to Rust",
                text: "{}, {:?} and {:.2} print exactly what Rust prints, thanks to a tiny support file.",
                code: "{:?} → rsDebug",
            },
            Item {
                title: "Comments kept",
                text: "Comments and /// documentation follow the code into Swift: the result stays readable.",
                code: "/// doc",
            },
            Item {
                title: "Precise diagnostics",
                text: "Whatever cannot be translated is reported with its line and column, like a compiler error.",
                code: "12:5: error",
            },
            Item {
                title: "A CLI with watch mode",
                text: "ferroswift build translates the crate, ferroswift watch translates again on every save while you code.",
                code: "ferroswift watch",
            },
            Item {
                title: "Runs in your browser",
                text: "The playground is the transpiler compiled to WebAssembly: your code never leaves your machine.",
                code: "wasm32",
            },
        ],
        how: Section {
            kicker: "How it works",
            title: "From a Rust crate to an Xcode project.",
            text: "A short, transparent pipeline: every step produces something you can read.",
        },
        steps: [
            Item {
                title: "Rust source",
                text: "Your crate, checked by cargo check and rust-analyzer: types, borrows, everything.",
                code: "src/*.rs",
            },
            Item {
                title: "syn AST",
                text: "Each file is parsed by syn, with the exact position of every node.",
                code: "syn::File",
            },
            Item {
                title: "Typed translation",
                text: "Types are tracked expression by expression to pick the right Swift: Int or Double, optional, throws, binding.",
                code: "Ty::Option",
            },
            Item {
                title: "Swift files",
                text: "One .swift file per .rs file, plus a small support file when needed.",
                code: "Sources/*.swift",
            },
            Item {
                title: "Xcode",
                text: "Add the generated folder to an iOS target, or swift run it on macOS.",
                code: "⌘R",
            },
        ],
        compare: Section {
            kicker: "Side by side",
            title: "Same program, two languages.",
            text: "These translations are produced by FerroSwift when this site is built: they are always accurate.",
        },
        tabs: ["SwiftUI view", "Enums & match", "Errors", "Formatting"],
        tab_notes: [
            "#[state] becomes @State, &mut self.name becomes $name, and modifiers keep their order.",
            "Enums with data become enums with associated values, match becomes switch, guards become where.",
            "Result<T, E> becomes throws(E), ? becomes try, and let-else becomes guard.",
            "{:?} and {:.2} go through small support functions to print exactly like Rust.",
        ],
        install: Section {
            kicker: "Get started",
            title: "Four commands.",
            text: "You need Rust to write the app, and Xcode (or Swift on macOS) to run it.",
        },
        commands: [
            Item {
                title: "Install the CLI",
                text: "From crates.io, with cargo.",
                code: "cargo install --git https://github.com/maxlestage/FerroSwift ferroswift-cli",
            },
            Item {
                title: "Create an app",
                text: "A ready-to-use Rust crate with a first view.",
                code: "ferroswift new MyApp",
            },
            Item {
                title: "Translate to Swift",
                text: "The Swift files are written next to the crate.",
                code: "ferroswift build",
            },
            Item {
                title: "Keep it in sync",
                text: "Every save translates the changed files again.",
                code: "ferroswift watch",
            },
        ],
        install_more: "Read the getting started guide",
    },
    playground: PlaygroundPage {
        title: "Playground",
        description: "Write Rust and read the Swift and SwiftUI FerroSwift makes of it, live in your browser thanks to WebAssembly.",
        lead: "Write Rust on the left, read Swift on the right. The translation happens in your browser, by FerroSwift compiled to WebAssembly: nothing leaves your machine.",
        tips_title: "Good to know",
        tips: [
            "Five examples to start with: a counter and a todo list in SwiftUI, shapes (enums, match, traits), errors (Result and ?) and iterators.",
            "Tab inserts four spaces, Shift+Tab removes them; Esc then Tab leaves the editor.",
            "Click a diagnostic to select its line.",
            "The support code (Rust-like formatting, small extensions) is only added when the translation needs it.",
            "The supported Rust subset is described in the docs: Language, SwiftUI and Limitations.",
        ],
    },
    docs: DocsText {
        title: "Documentation",
        description: "Everything to write a SwiftUI app in Rust with FerroSwift: installation, views, the supported Rust subset, the CLI and limitations.",
        intro: "Everything to write a SwiftUI app in Rust with FerroSwift: installation, views, the supported Rust subset, the CLI and the current limitations.",
        menu: "Documentation pages",
        on_this_page: "On this page",
        prev: "Previous",
        next: "Next",
        copy: "Copy",
    },
    not_found: NotFound {
        title: "Page not found",
        text: "This page does not exist, or no longer does. The playground and the docs are still here.",
        back: "Back to the home page",
    },
    offline: Offline {
        title: "You are offline",
        text: "This page is not saved on this device yet. The home page, the playground and the docs stay available without network.",
    },
    footer: Footer {
        tagline: "Write Rust. Ship SwiftUI.",
        start: "Start building with",
        install: "Install in one line",
        rust_only: "100 % Rust",
        product: "Product",
        project: "Project",
        made_with: "Site made with Active",
        license: "License",
        ecosystem: "Ecosystem",
        ecosystem_text: "Six Rust projects by one author, made to work together: from the front end to the database, and even to iPhone apps.",
        here: "You are here",
        projects: [
            "Front-end: components, SSR, WebAssembly islands",
            "Express-style web server, HTTP/3",
            "Security middlewares for Vitesse & tower",
            "Async, type-safe ORM, SQL & NoSQL",
            "The full-stack framework that unites them",
            "SwiftUI apps written in Rust",
        ],
        by: "Designed & built by",
        crafted: "Crafted in Rust, in France",
        rights: "© 2026 Maxime Nathan Lestage. All rights reserved.",
        proprietary: "Proprietary software",
        top: "Back to top",
    },
};
