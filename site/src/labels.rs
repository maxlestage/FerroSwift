//! The few texts the playground island needs in the browser. Kept apart from the pages'
//! dictionary (server/i18n.rs) so the WebAssembly bundle only carries these.

use crate::routes::Lang;

pub struct Labels {
    pub example: &'static str,
    pub editor: &'static str,
    pub output: &'static str,
    pub copy: &'static str,
    pub copied: &'static str,
    pub reset: &'static str,
    pub open: &'static str,
    pub server: &'static str,
    pub wasm: &'static str,
    pub problems: &'static str,
    pub no_problems: &'static str,
    pub error: &'static str,
    pub warning: &'static str,
    pub one_error: &'static str,
    pub errors: &'static str,
    pub one_warning: &'static str,
    pub warnings: &'static str,
    pub stale: &'static str,
    pub crash: &'static str,
    pub support: &'static str,
    pub lines: &'static str,
    pub hint: &'static str,
    pub go_to: &'static str,
    /// Decimal separator of the translation time.
    pub decimal: char,
}

pub fn labels(lang: Lang) -> &'static Labels {
    match lang {
        Lang::Fr => &FR,
        Lang::En => &EN,
    }
}

static FR: Labels = Labels {
    example: "Exemple",
    editor: "Code Rust (modifiable)",
    output: "Swift généré",
    copy: "Copier le Swift",
    copied: "Copié ✓",
    reset: "Réinitialiser",
    open: "Ouvrir en plein écran",
    server: "rendu sur le serveur",
    wasm: "WebAssembly",
    problems: "Diagnostics",
    no_problems: "Aucun problème : tout est traduit.",
    error: "erreur",
    warning: "avertissement",
    one_error: "1 erreur",
    errors: "erreurs",
    one_warning: "1 avertissement",
    warnings: "avertissements",
    stale: "Erreur de syntaxe : dernière traduction valide affichée.",
    crash: "Erreur interne du traducteur",
    support: "Code de support FerroSwift",
    lines: "lignes",
    hint: "Tab : 4 espaces · Échap puis Tab : sortir de l’éditeur",
    go_to: "Aller à la ligne",
    decimal: ',',
};

static EN: Labels = Labels {
    example: "Example",
    editor: "Rust code (editable)",
    output: "Generated Swift",
    copy: "Copy Swift",
    copied: "Copied ✓",
    reset: "Reset",
    open: "Open full screen",
    server: "rendered on the server",
    wasm: "WebAssembly",
    problems: "Diagnostics",
    no_problems: "No problems: everything was translated.",
    error: "error",
    warning: "warning",
    one_error: "1 error",
    errors: "errors",
    one_warning: "1 warning",
    warnings: "warnings",
    stale: "Syntax error: showing the last valid translation.",
    crash: "Internal translator error",
    support: "FerroSwift support code",
    lines: "lines",
    hint: "Tab: 4 spaces · Esc then Tab: leave the editor",
    go_to: "Go to line",
    decimal: '.',
};

/// `value` with two decimals, without the float formatting machinery:
/// `fixed2(0.745, '.') == "0.75"`.
pub fn fixed2(value: f64, decimal: char) -> String {
    let n = (value.max(0.0) * 100.0 + 0.5) as u64;
    format!("{}{decimal}{:02}", n / 100, n % 100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_times() {
        assert_eq!(fixed2(0.745, '.'), "0.75");
        assert_eq!(fixed2(12.0, ','), "12,00");
        assert_eq!(fixed2(-1.0, '.'), "0.00");
    }
}
