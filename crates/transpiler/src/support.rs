//! Swift support code: small helpers giving Swift the behavior of Rust where the standard
//! libraries differ (number formatting, `{:?}`, `Ordering`, string slicing by bytes…). Only
//! the helpers a translation uses are written.

use std::collections::BTreeSet;

/// (name, other helpers it needs, Swift code).
const HELPERS: &[(&str, &[&str], &str)] = &[
    ("display", &[], DISPLAY),
    ("fixed", &[], FIXED),
    ("debug_custom", &[], DEBUG_CUSTOM),
    ("less_any", &[], LESS_ANY),
    (
        "debug",
        &["debug_custom", "display", "fixed", "pad", "less_any"],
        DEBUG,
    ),
    ("pad", &[], PAD),
    (
        "formatter",
        &["debug", "pad", "fixed", "display"],
        FORMATTER,
    ),
    ("Ordering", &[], ORDERING),
    ("string", &[], STRING),
    ("char", &[], CHAR),
    ("vec", &["Ordering"], VEC),
    ("binary_search", &["Ordering", "IntError"], BINARY_SEARCH),
    ("reverse", &[], REVERSE),
    ("try_from", &["debug", "int_bounds"], TRY_FROM),
    ("int_bounds", &[], INT_BOUNDS),
    ("parse", &["debug", "int_bounds"], PARSE),
    ("utf8", &["debug"], UTF8),
    ("tuple1", &[], TUPLE1),
    ("heap", &[], HEAP),
    ("duration", &["debug_custom", "display"], DURATION),
    ("tuple_keys", &[], TUPLE_KEYS),
    ("ord_tuple", &[], ORD_TUPLE),
    ("chars", &[], CHARS),
    ("exp", &[], EXP),
    ("cbrt", &[], CBRT),
    ("map", &[], MAP),
    ("iter", &[], ITER),
    ("option", &[], OPTION),
    ("option_cmp", &[], OPTION_CMP),
    ("array_cmp", &[], ARRAY_CMP),
    ("link", &[], LINK),
    ("rc", &["debug"], RC),
    ("panic", &["debug"], PANIC),
    ("thread", &["debug", "duration"], THREAD),
    ("cow", &["debug"], COW),
    ("result", &[], RESULT),
    ("int", &[], INT),
    ("mem", &[], MEM),
    ("refcell", &["debug_custom", "mem"], REFCELL),
    ("default", &[], DEFAULT),
    ("into_double", &[], INTO_DOUBLE),
    ("async_result", &[], ASYNC_RESULT),
    ("eprint", &[], EPRINT),
    ("StringError", &[], STRING_ERROR),
    ("IntError", &[], INT_ERROR),
    ("FloatError", &[], FLOAT_ERROR),
    ("BoolError", &[], BOOL_ERROR),
    ("CharError", &[], CHAR_ERROR),
    ("ParseError", &[], PARSE_ERROR),
    ("parse_any", &["parse", "ParseError", "debug"], PARSE_ANY),
    ("color_hex", &[], COLOR_HEX),
];

/// The support code for `used` helpers, in a stable order. `ui`: SwiftUI is imported.
pub fn render(used: &BTreeSet<&'static str>, ui: bool) -> Option<String> {
    let mut all: BTreeSet<&str> = BTreeSet::new();
    let mut stack: Vec<&str> = used.iter().copied().collect();
    while let Some(h) = stack.pop() {
        if all.insert(h) {
            if let Some((_, deps, _)) = HELPERS.iter().find(|(n, _, _)| *n == h) {
                stack.extend(deps.iter().copied());
            }
        }
    }
    if !ui {
        all.remove("color_hex");
    }
    if all.is_empty() {
        return None;
    }
    let mut out = String::new();
    for (name, _, code) in HELPERS {
        if all.contains(name) {
            out.push_str(code.trim_start());
            out.push('\n');
        }
    }
    Some(out)
}

const DISPLAY: &str = r#"
/// A number as Rust's `{}` writes it: `1` for 1.0, never an exponent.
func rsDisplay<F: BinaryFloatingPoint & LosslessStringConvertible>(_ x: F) -> String {
    if x.isNaN { return "NaN" }
    if x.isInfinite { return x < 0 ? "-inf" : "inf" }
    var s = x.description
    var exponent = 0
    if let e = s.firstIndex(where: { $0 == "e" || $0 == "E" }) {
        exponent = Int(s[s.index(after: e)...]) ?? 0
        s = String(s[..<e])
    }
    let negative = s.hasPrefix("-")
    if negative { s.removeFirst() }
    let parts = s.split(separator: ".", omittingEmptySubsequences: false)
    let integer = String(parts[0])
    var fraction = parts.count > 1 ? String(parts[1]) : ""
    if fraction == "0" { fraction = "" }
    var digits = integer + fraction
    var point = integer.count + exponent
    if point <= 0 {
        digits = String(repeating: "0", count: 1 - point) + digits
        point = 1
    }
    if point > digits.count { digits += String(repeating: "0", count: point - digits.count) }
    var whole = String(digits.prefix(point))
    var decimals = String(digits.dropFirst(point))
    while decimals.hasSuffix("0") { decimals.removeLast() }
    while whole.count > 1 && whole.hasPrefix("0") { whole.removeFirst() }
    return (negative ? "-" : "") + whole + (decimals.isEmpty ? "" : "." + decimals)
}

/// `{}` of a value of a generic type: a float as Rust writes it, anything else as itself.
func rsDisplay<T>(_ value: T) -> String {
    switch value {
    case let d as Double: return rsDisplay(d)
    case let f as Float: return rsDisplay(f)
    default: return "\(value)"
    }
}
"#;

const FIXED: &str = r#"
/// `{:.2}`: a number with a fixed count of decimals.
func rsFixed<F: BinaryFloatingPoint>(_ x: F, _ decimals: Int) -> String {
    if x.isNaN { return "NaN" }
    return String(format: "%.\(decimals)f", Double(x))
}
"#;

const LESS_ANY: &str = r#"
private func rsLessOpened<T: Comparable>(_ x: T, _ y: Any) -> Bool? {
    (y as? T).map { x < $0 }
}

/// `a < b` as Rust orders values of any type: numbers and texts by value, `false < true`,
/// tuples and arrays element by element, `None` before `Some`.
func rsLessAny(_ a: Any, _ b: Any) -> Bool {
    func attempt<T: Comparable>(_: T.Type) -> Bool? {
        if let x = a as? T, let y = b as? T { return x < y }
        return nil
    }
    if let r = attempt(String.self) { return r }
    if let r = attempt(Int.self) { return r }
    if let r = attempt(Double.self) { return r }
    if let r = attempt(Character.self) { return r }
    if let r = attempt(UInt.self) { return r }
    if let r = attempt(Int32.self) { return r }
    if let r = attempt(UInt32.self) { return r }
    if let r = attempt(Int64.self) { return r }
    if let r = attempt(UInt64.self) { return r }
    if let r = attempt(Int8.self) { return r }
    if let r = attempt(UInt8.self) { return r }
    if let r = attempt(Int16.self) { return r }
    if let r = attempt(UInt16.self) { return r }
    if let r = attempt(Float.self) { return r }
    if let x = a as? Bool, let y = b as? Bool { return !x && y }
    // A type of the program ordered by its derived `Ord` (an enum, a struct).
    if let x = a as? any Comparable, let r = rsLessOpened(x, b) { return r }
    let ma = Mirror(reflecting: a), mb = Mirror(reflecting: b)
    if ma.displayStyle == .optional || mb.displayStyle == .optional {
        switch (ma.children.first?.value, mb.children.first?.value) {
        case (nil, nil): return false
        case (nil, _): return true
        case (_, nil): return false
        case let (x?, y?): return rsLessAny(x, y)
        }
    }
    if ma.displayStyle == .tuple && mb.displayStyle == .tuple
        || ma.displayStyle == .collection && mb.displayStyle == .collection
    {
        let xs = ma.children.map(\.value), ys = mb.children.map(\.value)
        for (x, y) in zip(xs, ys) {
            if rsLessAny(x, y) { return true }
            if rsLessAny(y, x) { return false }
        }
        return xs.count < ys.count
    }
    return false
}
"#;

const DEBUG: &str = r#"
/// A value as Rust's `{:?}` writes it: `Some(1)`, `Point { x: 1, y: 2 }`, `"text"`.
/// `{:?}` of a `BTreeSet`, given its elements in order.
func rsDebugSet<T>(_ items: [T], pretty: Bool = false) -> String {
    let inner = rsDebug(items, pretty: pretty)
    return "{" + inner.dropFirst().dropLast() + "}"
}

/// The width, fill and precision of a `{:?}`, which Rust's derived `Debug` passes down to
/// the numbers and booleans inside (`{:^7?}` of `Some(1)` is `Some(   1   )`).
struct RsDebugStyle {
    var width: Int? = nil
    var align: Character? = nil
    var fill: Character = " "
    var zero = false
    var precision: Int? = nil
    /// `{:x?}` or `{:X?}`: the integers inside in hexadecimal.
    var hex: Character? = nil

    func pad(_ s: String, numeric: Bool = true) -> String {
        guard let width else { return s }
        if zero && numeric { return rsPadZero(s, width) }
        return rsPad(s, width, align ?? (numeric ? ">" : "<"), fill)
    }
}

func rsDebug<T>(_ value: T, pretty: Bool = false, style: RsDebugStyle = RsDebugStyle()) -> String {
    var out = ""
    rsDebugWrite(value as Any, pretty, 0, &out, style)
    return out
}

/// A type whose `{:?}` is written by its own `fmt` with a formatter (`f.debug_struct(…)`).
protocol RsDebugText {
    func rsDebugText(pretty: Bool, style: RsDebugStyle) -> String
}

/// A value written bare by `{:?}` (`InvalidDigit`, the variant of a support enum).
struct RsDebugBare: RsDebugCustom {
    let rsDebugText: String
}

/// A support value as Rust's derived `Debug` writes a struct (`Name { a: 1 }`), a tuple
/// struct (`Name(1)`, its labels empty) or a unit struct (no fields), in `{:?}` and `{:#?}`.
func rsDebugRecord(_ name: String, _ fields: [(String, Any)], pretty: Bool, style: RsDebugStyle) -> String {
    if fields.isEmpty { return name }
    let tuple = fields.allSatisfy { $0.0.isEmpty }
    var out = name + (tuple ? "(" : " {")
    for (i, (label, value)) in fields.enumerated() {
        var text = ""
        rsDebugWrite(value, pretty, pretty ? 1 : 0, &text, style)
        let head = label.isEmpty ? "" : label + ": "
        if pretty {
            out += "\n    " + head + text + ","
        } else {
            out += (i == 0 ? (tuple ? "" : " ") : ", ") + head + text
        }
    }
    if pretty { return out + "\n" + (tuple ? ")" : "}") }
    return out + (tuple ? ")" : " }")
}

/// An integer in `{:?}`; in `{:x?}`, its two's complement in the width of its type.
private func rsDebugInt(_ value: Any, _ hex: Character?, prefix: Bool) -> String {
    guard let hex else { return "\(value)" }
    let upper = hex == "X"
    let digits: String
    switch value {
    case let v as Int: digits = String(UInt(bitPattern: v), radix: 16, uppercase: upper)
    case let v as Int8: digits = String(UInt8(bitPattern: v), radix: 16, uppercase: upper)
    case let v as Int16: digits = String(UInt16(bitPattern: v), radix: 16, uppercase: upper)
    case let v as Int32: digits = String(UInt32(bitPattern: v), radix: 16, uppercase: upper)
    case let v as Int64: digits = String(UInt64(bitPattern: v), radix: 16, uppercase: upper)
    case let v as UInt: digits = String(v, radix: 16, uppercase: upper)
    case let v as UInt8: digits = String(v, radix: 16, uppercase: upper)
    case let v as UInt16: digits = String(v, radix: 16, uppercase: upper)
    case let v as UInt32: digits = String(v, radix: 16, uppercase: upper)
    case let v as UInt64: digits = String(v, radix: 16, uppercase: upper)
    default: digits = "\(value)"
    }
    return prefix ? "0x" + digits : digits
}

private func rsSnake(_ name: String) -> String {
    var out = ""
    for c in name {
        if c.isUppercase {
            if !out.isEmpty { out.append("_") }
            out.append(contentsOf: c.lowercased())
        } else {
            out.append(c)
        }
    }
    return out
}

/// `a < b` for two values of the same comparable type, tuples element by element.
/// Values of a comparable type, in order; anything else as given.
private func rsSorted(_ values: [Any]) -> [Any] {
    func attempt<T: Comparable>(_: T.Type) -> [Any]? {
        let typed = values.compactMap { $0 as? T }
        return typed.count == values.count ? typed.sorted() : nil
    }
    // Tuples (the keys of a `BTreeMap<(A, B), V>`): element by element.
    if !values.isEmpty && values.allSatisfy({ Mirror(reflecting: $0).displayStyle == .tuple }) {
        return values.sorted { rsLessAny($0, $1) }
    }
    if let s = attempt(String.self) { return s }
    if let s = attempt(Int.self) { return s }
    if let s = attempt(Double.self) { return s }
    if let s = attempt(Character.self) { return s }
    if let s = attempt(UInt.self) { return s }
    if let s = attempt(Int32.self) { return s }
    if let s = attempt(UInt32.self) { return s }
    if let s = attempt(Int64.self) { return s }
    if let s = attempt(UInt64.self) { return s }
    if let s = attempt(Int8.self) { return s }
    if let s = attempt(UInt8.self) { return s }
    if let s = attempt(Int16.self) { return s }
    if let s = attempt(UInt16.self) { return s }
    if let s = attempt(Float.self) { return s }
    // Keys of a type of the program ordered by its derived `Ord` (an enum, a struct).
    if !values.isEmpty && values.allSatisfy({ $0 is any Comparable }) {
        return values.sorted { rsLessAny($0, $1) }
    }
    return values
}

private func rsSameKey(_ a: Any, _ b: Any) -> Bool {
    func same<T: Equatable>(_: T.Type) -> Bool? {
        guard let x = a as? T, let y = b as? T else { return nil }
        return x == y
    }
    // Tuple keys: element by element.
    let ma = Mirror(reflecting: a), mb = Mirror(reflecting: b)
    if ma.displayStyle == .tuple && mb.displayStyle == .tuple {
        let xs = ma.children.map(\.value), ys = mb.children.map(\.value)
        return xs.count == ys.count && zip(xs, ys).allSatisfy { rsSameKey($0, $1) }
    }
    if let r = same(String.self) { return r }
    if let r = same(Int.self) { return r }
    if let r = same(Double.self) { return r }
    if let r = same(Character.self) { return r }
    if let r = same(UInt.self) { return r }
    if let r = same(Int32.self) { return r }
    if let r = same(UInt32.self) { return r }
    if let r = same(Int64.self) { return r }
    if let r = same(UInt64.self) { return r }
    if let r = same(Int8.self) { return r }
    if let r = same(UInt8.self) { return r }
    if let r = same(Int16.self) { return r }
    if let r = same(UInt16.self) { return r }
    if let r = same(Float.self) { return r }
    return String(reflecting: a) == String(reflecting: b)
}

private func rsTypeName(_ value: Any) -> String {
    let name = String(describing: type(of: value))
    return String(name.prefix { $0 != "<" })
}

/// One scalar as Rust's `{:?}` writes it between quotes: control, format and other invisible
/// characters as `\u{…}`, a combining mark too when nothing comes before it.
private func rsEscapeDebug(_ c: Unicode.Scalar, quote: Unicode.Scalar, graphemeExtended: Bool) -> String {
    switch c {
    case "\t": return "\\t"
    case "\r": return "\\r"
    case "\n": return "\\n"
    case "\\": return "\\\\"
    case "\0": return "\\0"
    case quote: return "\\" + String(c)
    default: break
    }
    let p = c.properties
    let printable: Bool
    switch p.generalCategory {
    case .control, .format, .surrogate, .privateUse, .unassigned, .lineSeparator, .paragraphSeparator:
        printable = false
    case .spaceSeparator:
        printable = c == " "
    default:
        printable = true
    }
    if !printable || (graphemeExtended && p.isGraphemeExtend) {
        return "\\u{" + String(c.value, radix: 16) + "}"
    }
    return String(c)
}

private func rsDebugWrite(_ value: Any, _ pretty: Bool, _ level: Int, _ out: inout String, _ style: RsDebugStyle = RsDebugStyle()) {
    let mirror = Mirror(reflecting: value)
    // Before the casts below, which look through an optional.
    if mirror.displayStyle == .optional {
        if let some = mirror.children.first {
            out += "Some("
            if pretty {
                out += "\n" + String(repeating: "    ", count: level + 1)
                rsDebugWrite(some.value, pretty, level + 1, &out, style)
                out += ",\n" + String(repeating: "    ", count: level) + ")"
            } else {
                rsDebugWrite(some.value, pretty, level, &out, style)
                out += ")"
            }
        } else {
            out += "None"
        }
        return
    }
    if let custom = value as? RsDebugCustom {
        out += custom.rsDebugText
        return
    }
    if let custom = value as? RsDebugText {
        let text = custom.rsDebugText(pretty: pretty, style: style)
        let indent = "\n" + String(repeating: "    ", count: level)
        out += level == 0 ? text : text.split(separator: "\n", omittingEmptySubsequences: false).joined(separator: indent)
        return
    }
    let typeName = String(describing: type(of: value))
    if typeName.hasPrefix("Range<") || typeName.hasPrefix("ClosedRange<") {
        var lo = "", hi = ""
        for child in mirror.children {
            if child.label == "lowerBound" { rsDebugWrite(child.value, false, 0, &lo, style) }
            if child.label == "upperBound" { rsDebugWrite(child.value, false, 0, &hi, style) }
        }
        out += lo + (typeName.hasPrefix("Closed") ? "..=" : "..") + hi
        return
    }
    let pad = String(repeating: "    ", count: level + 1)
    let end = String(repeating: "    ", count: level)
    func list(_ open: String, _ close: String, _ items: [(String?, Any)]) {
        out += open
        if items.isEmpty { out += close == "}" && open.hasSuffix(" {") ? "}" : close; return }
        if pretty {
            out += "\n"
            for (label, item) in items {
                out += pad
                if let label { out += label + ": " }
                rsDebugWrite(item, pretty, level + 1, &out, style)
                out += ",\n"
            }
            out += end + close
        } else {
            if open.hasSuffix("{") { out += " " }
            for (i, (label, item)) in items.enumerated() {
                if i > 0 { out += ", " }
                if let label { out += label + ": " }
                rsDebugWrite(item, pretty, level, &out, style)
            }
            if close == "}" { out += " " }
            out += close
        }
    }
    // `(x,)`: the support struct of one element, with Rust's trailing comma.
    if rsTypeName(value) == "RsTuple1", let element = mirror.children.first?.value {
        if pretty {
            list("(", ")", [(nil, element)])
        } else {
            out += "("
            rsDebugWrite(element, false, level, &out, style)
            out += ",)"
        }
        return
    }
    // `Reverse(x)`.
    if rsTypeName(value) == "RsReverse", let element = mirror.children.first?.value {
        out += "Reverse"
        list("(", ")", [(nil, element)])
        return
    }
    // `(a, b)` held comparable, in a heap or a `Reverse`.
    if rsTypeName(value) == "RsOrd2" || rsTypeName(value) == "RsOrd3" {
        list("(", ")", mirror.children.map { (nil, $0.value) })
        return
    }
    switch value {
    case let s as String:
        out += "\""
        var first = true
        for c in s.unicodeScalars {
            out += rsEscapeDebug(c, quote: "\"", graphemeExtended: first)
            first = false
        }
        out += "\""
        return
    case let c as Character:
        out += "'"
        for s in c.unicodeScalars { out += rsEscapeDebug(s, quote: "'", graphemeExtended: true) }
        out += "'"
        return
    case let d as Double:
        out += style.pad(rsDebugFloat(d, style.precision))
        return
    case let f as Float:
        out += style.pad(rsDebugFloat(f, style.precision))
        return
    case is Bool:
        out += style.pad("\(value)", numeric: false)
        return
    case is Int, is Int8, is Int16, is Int32, is Int64, is UInt, is UInt8, is UInt16, is UInt32, is UInt64:
        if style.hex != nil && pretty && style.zero, let width = style.width {
            out += "0x" + rsPadZero(rsDebugInt(value, style.hex, prefix: false), width - 2)
        } else {
            out += style.pad(rsDebugInt(value, style.hex, prefix: pretty))
        }
        return
    default:
        break
    }
    switch mirror.displayStyle {
    case .collection:
        list("[", "]", mirror.children.map { (nil, $0.value) })
    case .set:
        // In order when the elements can be compared (a `BTreeSet`; a `HashSet` has no order).
        list("{", "}", rsSorted(mirror.children.map(\.value)).map { (nil, $0) })
    case .dictionary:
        out += "{"
        var pairs = mirror.children.map { Mirror(reflecting: $0.value).children.map(\.value) }
        pairs = rsSorted(pairs.map(\.[0])).compactMap { key in pairs.first { rsSameKey($0[0], key) } }
        if pretty && !pairs.isEmpty {
            out += "\n"
            for pair in pairs {
                out += pad
                rsDebugWrite(pair[0], pretty, level + 1, &out, style)
                out += ": "
                rsDebugWrite(pair[1], pretty, level + 1, &out, style)
                out += ",\n"
            }
            out += end
        } else {
            for (i, pair) in pairs.enumerated() {
                if i > 0 { out += ", " }
                rsDebugWrite(pair[0], pretty, level, &out, style)
                out += ": "
                rsDebugWrite(pair[1], pretty, level, &out, style)
            }
        }
        out += "}"
    case .tuple:
        if mirror.children.isEmpty {
            out += style.pad("()", numeric: false)
            return
        }
        list("(", ")", mirror.children.map { (nil, $0.value) })
    case .enum:
        // Payloads are laid out here when pretty; the type's own text otherwise.
        if let custom = value as? CustomDebugStringConvertible, !pretty || mirror.children.first == nil {
            out += custom.debugDescription
            return
        }
        guard let payload = mirror.children.first else {
            let name = String(String(reflecting: value).split(separator: ".").last ?? "")
            out += name.prefix(1).uppercased() + name.dropFirst()
            return
        }
        let label = payload.label ?? ""
        let isResult = rsTypeName(value) == "Result"
        let name = isResult ? (label == "success" ? "Ok" : "Err") : label.prefix(1).uppercased() + label.dropFirst()
        out += name
        let fields = Mirror(reflecting: payload.value)
        // `Ok(())`: the unit payload.
        if fields.displayStyle == .tuple && fields.children.isEmpty {
            out += "(())"
            return
        }
        if fields.displayStyle == .tuple && fields.children.contains(where: { $0.label.map { !$0.hasPrefix(".") } ?? false }) {
            list(" {", "}", fields.children.map { ($0.label.map(rsSnake), $0.value) })
        } else if fields.displayStyle == .tuple && !isResult {
            list("(", ")", fields.children.map { (nil, $0.value) })
        } else {
            // `Ok((0, 0))`: a `Result` holds one value, a tuple included.
            list("(", ")", [(nil, payload.value)])
        }
    case .struct, .class:
        if let custom = value as? CustomDebugStringConvertible {
            out += custom.debugDescription
            return
        }
        let name = (value as? RsDebugNamed)?.rsDebugName ?? rsTypeName(value)
        // `Mutex { data: 1, poisoned: false, .. }`.
        if name == "Mutex" || name == "RwLock", let data = mirror.children.first?.value {
            out += name
            if pretty {
                out += " {\n" + pad + "data: "
                rsDebugWrite(data, pretty, level + 1, &out, style)
                out += ",\n" + pad + "poisoned: false,\n" + pad + "..\n" + end + "}"
            } else {
                out += " { data: "
                rsDebugWrite(data, false, level, &out, style)
                out += ", poisoned: false, .. }"
            }
            return
        }
        let children = mirror.children.filter { !($0.label ?? "").hasPrefix("$") }
        if children.isEmpty {
            out += name
        } else if children.allSatisfy({ ($0.label ?? "").hasPrefix("_") && Int(($0.label ?? "").dropFirst()) != nil }) {
            out += name
            list("(", ")", children.map { (nil, $0.value) })
        } else {
            out += name
            list(" {", "}", children.map { (rsSnake(String(($0.label ?? "").drop { $0 == "_" })), $0.value) })
        }
    default:
        out += "\(value)"
    }
}

/// `{:?}` of a float: Rust writes it in full below 1e16 (and from 1e-4), in scientific
/// notation otherwise; a precision fixes the decimals.
private func rsDebugFloat<F: BinaryFloatingPoint & LosslessStringConvertible>(_ x: F, _ precision: Int?) -> String {
    if let precision { return rsFixed(x, precision) }
    if x.isNaN { return "NaN" }
    if x.isInfinite { return x < 0 ? "-inf" : "inf" }
    let abs = x.magnitude
    if abs < 1e16 && (abs == 0 || abs >= 1e-4) {
        let s = rsDisplay(x)
        return s.contains(".") ? s : s + ".0"
    }
    let s = x.description
    guard let e = s.firstIndex(where: { $0 == "e" || $0 == "E" }) else { return s }
    var mantissa = String(s[..<e])
    if mantissa.hasSuffix(".0") { mantissa.removeLast(2) }
    var exponent = String(s[s.index(after: e)...])
    if exponent.hasPrefix("+") { exponent.removeFirst() }
    let negative = exponent.hasPrefix("-")
    if negative { exponent.removeFirst() }
    while exponent.count > 1 && exponent.hasPrefix("0") { exponent.removeFirst() }
    return mantissa + "e" + (negative ? "-" : "") + exponent
}
"#;

const DEBUG_CUSTOM: &str = r#"
/// Support types with their own `{:?}` text.
protocol RsDebugCustom { var rsDebugText: String { get } }
/// Support types printed by `{:?}` under Rust's name (`RefCell { value: 1 }`).
protocol RsDebugNamed { var rsDebugName: String { get } }
"#;

const FORMATTER: &str = r#"
/// `fmt::Formatter`: the text being written and the spec of the placeholder being filled
/// (`{:>8.2}`), for a `Display` or `Debug` that asks for them.
final class RsFormatter {
    var out = ""
    var style: RsDebugStyle
    var alternate: Bool
    var signPlus: Bool

    init(width: Int? = nil, precision: Int? = nil, align: Character? = nil, fill: Character = " ", zero: Bool = false, alternate: Bool = false, signPlus: Bool = false) {
        style = RsDebugStyle(width: width, align: align, fill: fill, zero: zero, precision: precision)
        self.alternate = alternate
        self.signPlus = signPlus
    }

    init(style: RsDebugStyle, alternate: Bool) {
        self.style = style
        self.alternate = alternate
        signPlus = false
    }

    var width: Int? { style.width }
    var precision: Int? { style.precision }
    var fill: Character { style.fill }
    var zero: Bool { style.zero }
    var signMinus: Bool { false }

    /// `write!(f, …)`, `f.write_str(s)`.
    func write(_ s: String) { out += s }

    /// `f.pad(s)`: the text cut to the precision and padded to the width.
    func pad(_ s: String) {
        var s = s
        if let precision = style.precision { s = String(s.unicodeScalars.prefix(precision)) }
        if let width = style.width { s = rsPad(s, width, style.align ?? "<", style.fill) }
        out += s
    }

    /// `f.pad_integral(is_nonnegative, prefix, digits)`: a number with its sign and, for `#`,
    /// its prefix, the zeros of `0` after them.
    func padIntegral(_ nonNegative: Bool, _ prefix: String, _ digits: String) {
        var head = nonNegative ? (signPlus ? "+" : "") : "-"
        if alternate { head += prefix }
        guard let width = style.width, head.count + digits.count < width else {
            out += head + digits
            return
        }
        if style.zero {
            out += head + String(repeating: "0", count: width - head.count - digits.count) + digits
        } else {
            out += rsPad(head + digits, width, style.align ?? ">", style.fill)
        }
    }

    /// `value.fmt(f)` of a `Display`: the value written with this spec.
    func display(_ value: Any) {
        switch value {
        case let v as RsFormat: v.rsFormat(self)
        case let s as String: pad(s)
        case let s as Substring: pad(String(s))
        case let c as Character: pad(String(c))
        case let b as Bool: pad(b ? "true" : "false")
        case let d as Double: number(style.precision.map { rsFixed(d, $0) } ?? rsDisplay(d))
        case let x as Float: number(style.precision.map { rsFixed(x, $0) } ?? rsDisplay(x))
        case is Int, is Int8, is Int16, is Int32, is Int64, is UInt, is UInt8, is UInt16, is UInt32, is UInt64:
            number("\(value)")
        default: out += "\(value)"
        }
    }

    /// `value.fmt(f)` of a `Debug`.
    func debug(_ value: Any) { out += rsDebug(value, pretty: alternate, style: style) }

    private func number(_ s: String) {
        var s = s
        if signPlus { s = rsSign(s) }
        if let width = style.width { s = style.zero ? rsPadZero(s, width) : rsPad(s, width, style.align ?? ">", style.fill) }
        out += s
    }

    func debugStruct(_ name: String) -> RsDebugBuilder { RsDebugBuilder(self, .structure, name) }
    func debugTuple(_ name: String) -> RsDebugBuilder { RsDebugBuilder(self, .tuple, name) }
    func debugList() -> RsDebugBuilder { RsDebugBuilder(self, .list, "[") }
    func debugSet() -> RsDebugBuilder { RsDebugBuilder(self, .set, "{") }
    func debugMap() -> RsDebugBuilder { RsDebugBuilder(self, .map, "{") }
}

/// A `Display` written with its formatter: `description` is its text with no spec.
protocol RsFormat: CustomStringConvertible {
    func rsFormat(_ f: RsFormatter)
}

extension RsFormat {
    var description: String { rsFormatted(RsFormatter()) }

    /// The text of a placeholder with a spec (`{:>8}`).
    func rsFormatted(_ f: RsFormatter) -> String {
        rsFormat(f)
        return f.out
    }
}

/// A `Debug` written with its formatter (`f.debug_struct(…)`).
protocol RsDebugFormat: RsDebugText, CustomDebugStringConvertible {
    func rsDebugFormat(_ f: RsFormatter)
}

extension RsDebugFormat {
    var debugDescription: String { rsDebugText(pretty: false, style: RsDebugStyle()) }

    func rsDebugText(pretty: Bool, style: RsDebugStyle) -> String {
        let f = RsFormatter(style: style, alternate: pretty)
        rsDebugFormat(f)
        return f.out
    }
}

/// `f.debug_struct("Name")`, `f.debug_tuple`, `f.debug_list`, `f.debug_set`, `f.debug_map`:
/// Rust's layout of the fields, on one line or, for `{:#?}`, one per line.
final class RsDebugBuilder {
    enum Kind { case structure, tuple, list, set, map }

    private let f: RsFormatter
    private let kind: Kind
    private let name: String
    private var fields = 0

    init(_ f: RsFormatter, _ kind: Kind, _ name: String) {
        self.f = f
        self.kind = kind
        self.name = name
        f.out += name
    }

    /// The `{:?}` of a value inside, its lines indented when pretty.
    private func text(_ value: Any) -> String {
        let s = rsDebug(value, pretty: f.alternate, style: f.style)
        return f.alternate ? s.split(separator: "\n", omittingEmptySubsequences: false).joined(separator: "\n    ") : s
    }

    @discardableResult
    func field(_ name: String, _ value: Any) -> RsDebugBuilder {
        if f.alternate {
            f.out += (fields == 0 ? " {\n" : "") + "    " + name + ": " + text(value) + ",\n"
        } else {
            f.out += (fields == 0 ? " { " : ", ") + name + ": " + text(value)
        }
        fields += 1
        return self
    }

    @discardableResult
    func field(_ value: Any) -> RsDebugBuilder {
        if f.alternate {
            f.out += (fields == 0 ? "(\n" : "") + "    " + text(value) + ",\n"
        } else {
            f.out += (fields == 0 ? "(" : ", ") + text(value)
        }
        fields += 1
        return self
    }

    @discardableResult
    func entry(_ value: Any) -> RsDebugBuilder {
        if f.alternate {
            f.out += (fields == 0 ? "\n" : "") + "    " + text(value) + ",\n"
        } else {
            f.out += (fields == 0 ? "" : ", ") + text(value)
        }
        fields += 1
        return self
    }

    @discardableResult
    func entry(_ key: Any, _ value: Any) -> RsDebugBuilder {
        self.key(key)
        return self.value(value)
    }

    @discardableResult
    func key(_ key: Any) -> RsDebugBuilder {
        if f.alternate {
            f.out += (fields == 0 ? "\n" : "") + "    " + text(key) + ": "
        } else {
            f.out += (fields == 0 ? "" : ", ") + text(key) + ": "
        }
        return self
    }

    @discardableResult
    func value(_ value: Any) -> RsDebugBuilder {
        f.out += text(value) + (f.alternate ? ",\n" : "")
        fields += 1
        return self
    }

    @discardableResult
    func entries<S: Sequence>(_ items: S) -> RsDebugBuilder {
        for item in items {
            let parts = Mirror(reflecting: item).children.map(\.value)
            if kind == .map && parts.count == 2 {
                entry(parts[0], parts[1])
            } else {
                entry(item)
            }
        }
        return self
    }

    func finish() {
        switch kind {
        case .structure:
            if fields > 0 { f.out += f.alternate ? "}" : " }" }
        case .tuple:
            if fields > 0 { f.out += f.alternate ? ")" : (fields == 1 && name.isEmpty ? ",)" : ")") }
        case .list:
            f.out += "]"
        case .set, .map:
            f.out += "}"
        }
    }

    /// `finish_non_exhaustive()`: `..` after the fields.
    func finishNonExhaustive() {
        let close: String
        switch kind {
        case .structure: close = "}"
        case .tuple: close = ")"
        case .list: close = "]"
        case .set, .map: close = "}"
        }
        if fields == 0 {
            f.out += kind == .structure ? " { .. }" : (kind == .tuple ? "(..)" : ".." + close)
        } else if f.alternate {
            f.out += "    ..\n" + close
        } else {
            f.out += kind == .structure ? ", .. }" : ", .." + close
        }
    }
}
"#;

const PAD: &str = r#"
/// `{:>8}`, `{:<8}`, `{:^8}`: a text padded to a width, counted in scalars as Rust counts
/// `char`s (a combining mark is one).
func rsPad(_ s: String, _ width: Int, _ align: Character, _ fill: Character) -> String {
    let missing = width - s.unicodeScalars.count
    if missing <= 0 { return s }
    let fill = String(fill)
    switch align {
    case ">": return String(repeating: fill, count: missing) + s
    case "^": return String(repeating: fill, count: missing / 2) + s + String(repeating: fill, count: missing - missing / 2)
    default: return s + String(repeating: fill, count: missing)
    }
}

/// `{:08}`: a number padded with zeros after its sign.
func rsPadZero(_ s: String, _ width: Int) -> String {
    let missing = width - s.count
    if missing <= 0 { return s }
    if s.hasPrefix("-") || s.hasPrefix("+") {
        return String(s.first!) + String(repeating: "0", count: missing) + s.dropFirst()
    }
    return String(repeating: "0", count: missing) + s
}

/// `{:+}`: the sign of positive numbers too.
func rsSign(_ s: String) -> String {
    s.hasPrefix("-") ? s : "+" + s
}

func rsSign<T: Numeric & Comparable>(_ x: T) -> String {
    x < 0 ? "\(x)" : "+\(x)"
}
"#;

const ORDERING: &str = r#"
/// `std::cmp::Ordering`.
enum Ordering: Equatable {
    case less, equal, greater

    func reverse() -> Ordering {
        switch self {
        case .less: return .greater
        case .equal: return .equal
        case .greater: return .less
        }
    }

    func then(_ other: Ordering) -> Ordering { self == .equal ? other : self }
    func thenWith(_ other: () -> Ordering) -> Ordering { self == .equal ? other() : self }
    var isLt: Bool { self == .less }
    var isLe: Bool { self != .greater }
    var isGt: Bool { self == .greater }
    var isGe: Bool { self != .less }
    var isEq: Bool { self == .equal }
    var isNe: Bool { self != .equal }
}

/// `a.cmp(&b)`.
func rsCmp<T: Comparable>(_ a: T, _ b: T) -> Ordering {
    a < b ? .less : a == b ? .equal : .greater
}

/// `false < true`, as in Rust.
func rsCmp(_ a: Bool, _ b: Bool) -> Ordering {
    a == b ? .equal : (b ? .less : .greater)
}

/// `a.partial_cmp(&b)`: `nil` when a value is NaN.
func rsPartialCmp<T: Comparable>(_ a: T, _ b: T) -> Ordering? {
    a < b ? .less : a == b ? .equal : b < a ? .greater : nil
}

/// `a.total_cmp(&b)`: IEEE total order (`-NaN < -inf < … < -0 < 0 < … < inf < NaN`).
func rsTotalCmp<T: BinaryFloatingPoint>(_ a: T, _ b: T) -> Ordering {
    if a.isTotallyOrdered(belowOrEqualTo: b) {
        return b.isTotallyOrdered(belowOrEqualTo: a) ? .equal : .less
    }
    return .greater
}

/// `a.cmp(&b)` on slices: element by element, the shorter first.
func rsCmp<T: Comparable>(_ a: [T], _ b: [T]) -> Ordering {
    for (x, y) in zip(a, b) where x != y {
        return x < y ? .less : .greater
    }
    return rsCmp(a.count, b.count)
}

/// `(a, b).cmp(&(c, d))`: tuples compare element by element.
func rsCmp<A: Comparable, B: Comparable>(_ a: (A, B), _ b: (A, B)) -> Ordering {
    rsCmp(a.0, b.0).then(rsCmp(a.1, b.1))
}

func rsCmp<A: Comparable, B: Comparable, C: Comparable>(_ a: (A, B, C), _ b: (A, B, C)) -> Ordering {
    rsCmp(a.0, b.0).then(rsCmp(a.1, b.1)).then(rsCmp(a.2, b.2))
}

func rsPartialCmp<A: Comparable, B: Comparable>(_ a: (A, B), _ b: (A, B)) -> Ordering? {
    guard let first = rsPartialCmp(a.0, b.0) else { return nil }
    return first == .equal ? rsPartialCmp(a.1, b.1) : first
}

func rsPartialCmp<A: Comparable, B: Comparable, C: Comparable>(_ a: (A, B, C), _ b: (A, B, C)) -> Ordering? {
    guard let first = rsPartialCmp((a.0, a.1), (b.0, b.1)) else { return nil }
    return first == .equal ? rsPartialCmp(a.2, b.2) : first
}
"#;

const STRING: &str = r#"
extension String {
    /// `&s[a..b]`: a slice by UTF-8 byte offsets.
    func rsSlice(_ start: Int, _ end: Int) -> String {
        String(decoding: Array(utf8)[start..<end], as: UTF8.self)
    }

    /// `s.escape_default()`: tabs, newlines, quotes and backslashes escaped, anything
    /// outside printable ASCII as `\u{…}`.
    func escapeDefault() -> String {
        var out = ""
        for c in unicodeScalars {
            switch c {
            case "\t": out += "\\t"
            case "\r": out += "\\r"
            case "\n": out += "\\n"
            case "\\": out += "\\\\"
            case "'": out += "\\'"
            case "\"": out += "\\\""
            case " "..."~": out.unicodeScalars.append(c)
            default: out += "\\u{" + String(c.value, radix: 16) + "}"
            }
        }
        return out
    }

    /// `c.escape_unicode()`: every scalar as `\u{…}`.
    func escapeUnicode() -> String {
        unicodeScalars.map { "\\u{" + String($0.value, radix: 16) + "}" }.joined()
    }

    /// `s.replace_range(a..b, "x")`: the bytes of the range replaced.
    mutating func replaceRange(_ r: some RangeExpression<Int>, _ with: String) {
        var bytes = Array(utf8)
        let range = r.relative(to: bytes)
        bytes.replaceSubrange(range, with: Array(with.utf8))
        self = String(decoding: bytes, as: UTF8.self)
    }

    /// `s.get(a..b)`: the slice when both byte offsets fall on character boundaries.
    func rsGet(_ r: some RangeExpression<Int>) -> String? {
        let bytes = Array(utf8)
        let range = r.relative(to: bytes)
        guard range.lowerBound >= 0, range.upperBound <= bytes.count else { return nil }
        func boundary(_ i: Int) -> Bool { i == bytes.count || bytes[i] & 0xC0 != 0x80 }
        guard boundary(range.lowerBound), boundary(range.upperBound) else { return nil }
        return String(decoding: bytes[range], as: UTF8.self)
    }

    /// `s.find(p)`: the byte offset of the first occurrence.
    func find(_ pattern: String) -> Int? {
        guard let r = range(of: pattern) else { return nil }
        return utf8.distance(from: startIndex, to: r.lowerBound)
    }

    /// `s.rfind(p)`.
    func rfind(_ pattern: String) -> Int? {
        guard let r = range(of: pattern, options: .backwards) else { return nil }
        return utf8.distance(from: startIndex, to: r.lowerBound)
    }

    func trimStart() -> String { String(drop(while: \.isWhitespace)) }
    func trimEnd() -> String { String(reversed().drop(while: \.isWhitespace).reversed()) }

    func trimStartMatches(_ p: String) -> String {
        var s = self
        while !p.isEmpty && s.hasPrefix(p) { s.removeFirst(p.count) }
        return s
    }

    func trimEndMatches(_ p: String) -> String {
        var s = self
        while !p.isEmpty && s.hasSuffix(p) { s.removeLast(p.count) }
        return s
    }

    func trimMatches(_ p: String) -> String { trimStartMatches(p).trimEndMatches(p) }

    /// `s.match_indices(p)`: the byte offset of each match with the matched text.
    func matchIndices(_ p: String) -> [(Int, String)] {
        var out: [(Int, String)] = []
        guard !p.isEmpty else { return out }
        var from = startIndex
        while from < endIndex, let r = range(of: p, range: from..<endIndex) {
            out.append((utf8.distance(from: startIndex, to: r.lowerBound), p))
            from = r.upperBound
        }
        return out
    }

    /// `s.match_indices(char::is_numeric)`.
    func matchIndices(where p: (Character) -> Bool) -> [(Int, String)] {
        var out: [(Int, String)] = []
        var offset = 0
        for s in unicodeScalars {
            if p(Character(s)) { out.append((offset, String(s))) }
            offset += s.utf8.count
        }
        return out
    }

    /// `s.retain(|c| …)`: keeps the characters the predicate accepts.
    mutating func retain(_ keep: (Character) -> Bool) {
        self = String(String.UnicodeScalarView(unicodeScalars.filter { keep(Character($0)) }))
    }

    /// `s.drain(a..b)`: removes and returns the text between two byte offsets.
    mutating func drain(_ r: some RangeExpression<Int>) -> String {
        let bytes = Array(utf8)
        let range = r.relative(to: bytes)
        let removed = String(decoding: bytes[range], as: UTF8.self)
        self = String(decoding: bytes[..<range.lowerBound] + bytes[range.upperBound...], as: UTF8.self)
        return removed
    }

    /// `s.drain(..)`.
    mutating func drain(_ r: UnboundedRange) -> String {
        defer { removeAll() }
        return self
    }

    /// `s.split_off(n)`: the text from a byte offset, removed from `s`.
    mutating func splitOff(_ at: Int) -> String {
        let tail = rsSlice(at, utf8.count)
        self = rsSlice(0, at)
        return tail
    }

    /// `s.to_ascii_uppercase()`: only the ASCII letters change.
    func toAsciiUppercase() -> String {
        String(String.UnicodeScalarView(unicodeScalars.map { s in
            s.value >= 97 && s.value <= 122 ? UnicodeScalar(s.value - 32)! : s
        }))
    }

    /// `s.to_ascii_lowercase()`.
    func toAsciiLowercase() -> String {
        String(String.UnicodeScalarView(unicodeScalars.map { s in
            s.value >= 65 && s.value <= 90 ? UnicodeScalar(s.value + 32)! : s
        }))
    }

    mutating func makeAsciiUppercase() { self = toAsciiUppercase() }
    mutating func makeAsciiLowercase() { self = toAsciiLowercase() }

    /// `s.replacen(from, to, n)`: the first `n` occurrences.
    func replacen(_ from: String, _ to: String, _ n: Int) -> String {
        var out = ""
        var rest = self[...]
        var left = n
        while left > 0, !from.isEmpty, let r = rest.range(of: from) {
            out += rest[..<r.lowerBound] + to
            rest = rest[r.upperBound...]
            left -= 1
        }
        return out + rest
    }

    /// `s.splitn(n, p)`: at most `n` pieces.
    func splitn(_ n: Int, _ p: String) -> [String] {
        var out: [String] = []
        var rest = self[...]
        while out.count < n - 1, let r = rest.range(of: p) {
            out.append(String(rest[..<r.lowerBound]))
            rest = rest[r.upperBound...]
        }
        if n > 0 { out.append(String(rest)) }
        return out
    }

    /// `s.rsplitn(n, p)`: at most `n` pieces, from the end.
    func rsplitn(_ n: Int, _ p: String) -> [String] {
        var out: [String] = []
        var rest = self[...]
        while out.count < n - 1, let r = rest.range(of: p, options: .backwards) {
            out.append(String(rest[r.upperBound...]))
            rest = rest[..<r.lowerBound]
        }
        if n > 0 { out.append(String(rest)) }
        return out
    }

    /// `s.split_terminator(p)`: like `split`, without a last empty piece.
    func splitTerminator(_ p: String) -> [String] {
        var parts = components(separatedBy: p)
        if parts.last == "" { parts.removeLast() }
        return parts
    }

    /// `s.lines()`.
    /// `s.lines()`: split at `\n`, a `\r` before it dropped, on scalars (Swift's characters
    /// group `\r\n` into one); the last line needs no line ending.
    func lines() -> [String] {
        var out: [String] = []
        var current = String.UnicodeScalarView()
        for s in unicodeScalars {
            if s == "\n" {
                if current.last == "\r" { current.removeLast() }
                out.append(String(current))
                current = String.UnicodeScalarView()
            } else {
                current.append(s)
            }
        }
        if !current.isEmpty { out.append(String(current)) }
        return out
    }

    func splitOnce(_ p: String) -> (String, String)? {
        guard let r = range(of: p) else { return nil }
        return (String(self[..<r.lowerBound]), String(self[r.upperBound...]))
    }

    func rsplitOnce(_ p: String) -> (String, String)? {
        guard let r = range(of: p, options: .backwards) else { return nil }
        return (String(self[..<r.lowerBound]), String(self[r.upperBound...]))
    }

    func stripPrefix(_ p: String) -> String? { hasPrefix(p) ? String(dropFirst(p.count)) : nil }
    func stripSuffix(_ p: String) -> String? { hasSuffix(p) ? String(dropLast(p.count)) : nil }

    /// `s.is_char_boundary(i)`: a byte offset at the start of a character, or the end.
    func isCharBoundary(_ offset: Int) -> Bool {
        if offset == 0 || offset == utf8.count { return true }
        if offset < 0 || offset > utf8.count { return false }
        let byte = utf8[utf8.index(utf8.startIndex, offsetBy: offset)]
        return byte & 0xC0 != 0x80
    }

    /// `s.split_at(i)`: at a byte offset.
    func splitAt(_ offset: Int) -> (String, String) {
        let i = utf8.index(startIndex, offsetBy: offset)
        return (String(self[..<i]), String(self[i...]))
    }

    /// `s.find(char::is_whitespace)`: the byte offset of the first character that matches.
    func findWhere(_ p: (Character) -> Bool) -> Int? {
        var offset = 0
        for s in unicodeScalars {
            if p(Character(s)) { return offset }
            offset += s.utf8.count
        }
        return nil
    }

    /// `s.rfind(char::is_whitespace)`.
    func rfindWhere(_ p: (Character) -> Bool) -> Int? {
        var found: Int? = nil
        var offset = 0
        for s in unicodeScalars {
            if p(Character(s)) { found = offset }
            offset += s.utf8.count
        }
        return found
    }

    /// `s.trim_end_matches(char::is_numeric)`.
    func trimEndMatches(where p: (Character) -> Bool) -> String {
        var scalars = unicodeScalars[...]
        while let last = scalars.last, p(Character(last)) { scalars = scalars.dropLast() }
        return String(scalars)
    }

    /// `s.trim_matches(char::is_numeric)`.
    func trimMatches(where p: (Character) -> Bool) -> String {
        String(unicodeScalars.drop(while: { p(Character($0)) })).trimEndMatches(where: p)
    }

    /// `s.char_indices()`: characters with their byte offsets.
    func charIndices() -> [(Int, Character)] {
        var offset = 0
        return unicodeScalars.map { s in
            defer { offset += s.utf8.count }
            return (offset, Character(s))
        }
    }

    /// `s.insert(byte_offset, c)`.
    mutating func insertAt(_ offset: Int, _ s: some StringProtocol) {
        let i = utf8.index(startIndex, offsetBy: offset)
        insert(contentsOf: s, at: i)
    }

    mutating func insertAt(_ offset: Int, _ c: Character) {
        let i = utf8.index(startIndex, offsetBy: offset)
        insert(c, at: i)
    }

    /// `s.remove(byte_offset)`.
    mutating func removeAt(_ offset: Int) -> Character {
        remove(at: utf8.index(startIndex, offsetBy: offset))
    }

    /// `s.truncate(byte_length)`.
    mutating func truncate(_ length: Int) {
        if utf8.count > length { self = rsSlice(0, length) }
    }
}
"#;

const CHAR: &str = r#"
extension Character {
    /// `c as u32`.
    var rsCode: UInt32 { unicodeScalars.first!.value }

    /// `n as char`.
    init(rsScalar value: some BinaryInteger) {
        self.init(UnicodeScalar(UInt32(truncatingIfNeeded: value)) ?? "\u{FFFD}")
    }

    var isAsciiDigit: Bool { isASCII && isNumber }
    var isAsciiAlphabetic: Bool { isASCII && isLetter }
    var isAsciiAlphanumeric: Bool { isASCII && (isLetter || isNumber) }
    var isAsciiUppercase: Bool { isASCII && isUppercase }
    var isAsciiLowercase: Bool { isASCII && isLowercase }
    var isAsciiGraphic: Bool { asciiValue.map { (33...126).contains($0) } ?? false }
    var isAsciiControl: Bool { asciiValue.map { $0 < 32 || $0 == 127 } ?? false }
    var isAsciiPunctuation: Bool { isASCII && (isPunctuation || isSymbol) }
    var isAsciiHexdigit: Bool { isASCII && isHexDigit }

    /// `c.to_digit(radix)`.
    func toDigit(_ radix: UInt32) -> UInt32? {
        guard isASCII, let v = hexDigitValue ?? (isLetter ? Int(rsCode) - 87 : nil), v < Int(radix) else { return nil }
        return UInt32(v)
    }

    /// `char::from_digit(d, radix)`.
    static func fromDigit(_ d: UInt32, _ radix: UInt32) -> Character? {
        guard d < radix else { return nil }
        return Character(String(d, radix: Int(radix)))
    }
}

/// `'a'..='z'`: the characters between two scalars.
func rsChars(_ a: Character, _ b: Character, through: Bool) -> [Character] {
    let lo = a.rsCode, hi = b.rsCode
    guard through ? lo <= hi : lo < hi else { return [] }
    return (lo...(through ? hi : hi - 1)).compactMap { UnicodeScalar($0).map(Character.init) }
}
"#;

const REVERSE: &str = r#"
/// `std::cmp::Reverse`: the opposite order.
struct RsReverse<T: Comparable>: Comparable {
    let value: T
    init(_ value: T) { self.value = value }
    static func < (a: RsReverse, b: RsReverse) -> Bool { a.value > b.value }
}
"#;

const CBRT: &str = r#"
/// `x.cbrt()`: the cube root, corrected to the closest of its neighbours (the system's
/// `cbrt(27)` is a step above 3).
func rsCbrt<F: BinaryFloatingPoint>(_ x: F) -> F {
    let r = F(cbrt(Double(x)))
    guard r.isFinite, r != 0 else { return r }
    var best = r
    var error = abs(r * r * r - x)
    for c in [r.nextDown, r.nextUp] {
        let e = abs(c * c * c - x)
        if e < error {
            best = c
            error = e
        }
    }
    return best
}
"#;

const EXP: &str = r#"
/// `{:e}`: Rust's scientific notation (`1.2345e3`, `1e-7`), with a precision when given.
func rsExp<F: BinaryFloatingPoint>(_ x: F, _ precision: Int? = nil, upper: Bool = false) -> String {
    let d = Double(x)
    if d.isNaN { return "NaN" }
    if d.isInfinite { return d < 0 ? "-inf" : "inf" }
    var mantissa: String
    var exponent: Int
    if let p = precision {
        let parts = String(format: "%.\(p)e", d).split(separator: "e")
        mantissa = String(parts[0])
        exponent = Int(parts[1]) ?? 0
    } else {
        // The shortest digits that read back as the same number, like Rust.
        var text = "\(d.magnitude)"
        var e = 0
        if let i = text.firstIndex(where: { $0 == "e" || $0 == "E" }) {
            e = Int(text[text.index(after: i)...]) ?? 0
            text = String(text[..<i])
        }
        let int = text.split(separator: ".", omittingEmptySubsequences: false)[0]
        var digits = text.replacingOccurrences(of: ".", with: "")
        exponent = int.count - 1 + e
        while digits.hasPrefix("0") && digits.count > 1 { digits.removeFirst(); exponent -= 1 }
        while digits.hasSuffix("0") && digits.count > 1 { digits.removeLast() }
        if digits == "0" { exponent = 0 }
        mantissa = (d.sign == .minus ? "-" : "") + String(digits.prefix(1)) + (digits.count > 1 ? "." + digits.dropFirst() : "")
    }
    return mantissa + (upper ? "E" : "e") + String(exponent)
}

func rsExp<I: BinaryInteger>(_ x: I, _ precision: Int? = nil, upper: Bool = false) -> String {
    rsExp(Double(x), precision, upper: upper)
}
"#;

const VEC: &str = r#"
extension MutableCollection {
    /// `v.fill(x)`, also on a slice (`v[..3].fill(0)`).
    mutating func fill(_ value: Element) {
        for i in indices { self[i] = value }
    }
}

extension Array {
    /// `q.remove(i)` of a `VecDeque`: the element, or `nil` past the end.
    mutating func rsDequeRemove(_ i: Int) -> Element? {
        i < count ? remove(at: i) : nil
    }

    /// `q.swap_remove_back(i)`: the element, the last one taking its place.
    mutating func rsSwapRemoveBack(_ i: Int) -> Element? {
        guard i < count else { return nil }
        swapAt(i, count - 1)
        return removeLast()
    }

    /// `q.swap_remove_front(i)`: the element, the first one taking its place.
    mutating func rsSwapRemoveFront(_ i: Int) -> Element? {
        guard i < count else { return nil }
        swapAt(i, 0)
        return removeFirst()
    }
}

extension MutableCollection where Index == Int {
    /// `v[1..3].swap(i, j)`: the indices count from the start of the slice.
    mutating func rsSwap(_ i: Int, _ j: Int) {
        swapAt(startIndex + i, startIndex + j)
    }
}

extension Array {
    /// `v.get(i)`.
    func get(_ i: Int) -> Element? { indices.contains(i) ? self[i] : nil }

    /// `v.get(a..b)`: the part, when within bounds.
    func get(_ r: Range<Int>) -> [Element]? {
        r.lowerBound >= 0 && r.upperBound <= count ? Array(self[r]) : nil
    }

    mutating func retain(_ keep: (Element) throws -> Bool) rethrows {
        try removeAll { try !keep($0) }
    }

    /// `v.copy_within(a..b, dest)`: a part copied over another.
    mutating func copyWithin(_ source: some RangeExpression<Int>, _ dest: Int) {
        let part = Array(self[source])
        replaceSubrange(dest..<dest + part.count, with: part)
    }

    /// `v.retain_mut(|x| …)`: the closure may change each element.
    mutating func retainMut(_ keep: (inout Element) throws -> Bool) rethrows {
        var out: [Element] = []
        for var x in self where try keep(&x) { out.append(x) }
        self = out
    }

    /// `v.dedup_by_key(|x| k)`: consecutive elements with the same key.
    mutating func dedupByKey<K: Equatable>(_ key: (inout Element) throws -> K) rethrows {
        var out: [Element] = []
        var last: K?
        for var x in self {
            let k = try key(&x)
            if last != k { out.append(x) }
            last = k
        }
        self = out
    }

    /// `v.dedup_by(|a, b| same)`: consecutive elements the closure finds equal.
    mutating func dedupBy(_ same: (inout Element, inout Element) throws -> Bool) rethrows {
        var out: [Element] = []
        for var x in self {
            if var prev = out.last, try same(&x, &prev) { continue }
            out.append(x)
        }
        self = out
    }

    mutating func truncate(_ length: Int) {
        if count > length { removeLast(count - length) }
    }

    mutating func swapRemove(_ i: Int) -> Element {
        swapAt(i, count - 1)
        return removeLast()
    }

    mutating func resize(_ length: Int, _ value: Element) {
        if count > length { removeLast(count - length) } else { append(contentsOf: repeatElement(value, count: length - count)) }
    }

    mutating func rotateLeft(_ n: Int) {
        guard !isEmpty else { return }
        let k = n % count
        self = Array(self[k...] + self[..<k])
    }

    mutating func rotateRight(_ n: Int) {
        guard !isEmpty else { return }
        rotateLeft(count - n % count)
    }

    mutating func splitOff(_ at: Int) -> [Element] {
        let tail = Array(self[at...])
        removeLast(count - at)
        return tail
    }

    /// `v.drain(..)`.
    mutating func drain() -> [Element] {
        defer { removeAll() }
        return self
    }

    /// `v.drain(a..b)`: the removed elements.
    mutating func drain(_ r: some RangeExpression<Int>) -> [Element] {
        let range = r.relative(to: self)
        defer { removeSubrange(range) }
        return Array(self[range])
    }

    func splitAt(_ i: Int) -> ([Element], [Element]) { (Array(self[..<i]), Array(self[i...])) }

    func windows(_ size: Int) -> [[Element]] {
        guard size > 0, count >= size else { return [] }
        return (0...(count - size)).map { Array(self[$0..<($0 + size)]) }
    }

    func chunks(_ size: Int) -> [[Element]] {
        stride(from: 0, to: count, by: size).map { Array(self[$0..<Swift.min($0 + size, count)]) }
    }

    /// `v.chunks_exact(n)`: the full chunks, and `remainder()` for what is left.
    func chunksExact(_ size: Int) -> RsChunksExact<Element> {
        let full = count - count % size
        return RsChunksExact(chunks: Array(self[..<full]).chunks(size), remainder: Array(self[full...]))
    }

    /// `v.chunk_by(|a, b| …)`: runs of consecutive elements the predicate chains.
    func chunkBy(_ same: (Element, Element) throws -> Bool) rethrows -> [[Element]] {
        var out: [[Element]] = []
        for x in self {
            if let last = out.last?.last, try same(last, x) {
                out[out.count - 1].append(x)
            } else {
                out.append([x])
            }
        }
        return out
    }

    /// `v.rchunks(n)`: the chunks from the end.
    func rchunks(_ size: Int) -> [[Element]] {
        var out: [[Element]] = []
        var end = count
        while end > 0 {
            let start = Swift.max(0, end - size)
            out.append(Array(self[start..<end]))
            end = start
        }
        return out
    }

    /// `v.partition_point(|x| …)`: the first index whose element fails the predicate, by
    /// binary search.
    func partitionPoint(_ belongsToFirst: (Element) throws -> Bool) rethrows -> Int {
        var lo = 0
        var hi = count
        while lo < hi {
            let mid = (lo + hi) / 2
            if try belongsToFirst(self[mid]) { lo = mid + 1 } else { hi = mid }
        }
        return lo
    }

    mutating func sortBy(_ compare: (Element, Element) throws -> Ordering) rethrows {
        try sort { try compare($0, $1) == .less }
    }

    mutating func sortByKey<K: Comparable>(_ key: (Element) throws -> K) rethrows {
        try sort { try key($0) < key($1) }
    }
}

struct RsChunksExact<Element>: Sequence {
    let chunks: [[Element]]
    let remainder: [Element]

    func makeIterator() -> IndexingIterator<[[Element]]> { chunks.makeIterator() }
}

extension Sequence {
    /// `iter.inspect(|x| …)`: the closure runs as each element is pulled.
    func inspect(_ f: @escaping (Element) -> Void) -> LazyMapSequence<Self, Element> {
        lazy.map { x in
            f(x)
            return x
        }
    }

    /// `iter.map_while(|x| …)`: the values until the closure gives `None`.
    func mapWhile<T>(_ f: (Element) throws -> T?) rethrows -> [T] {
        var out: [T] = []
        for x in self {
            guard let y = try f(x) else { break }
            out.append(y)
        }
        return out
    }
}

extension MutableCollection {
    /// `v[..n].copy_from_slice(&src)`.
    mutating func copyFromSlice(_ source: [Element]) {
        precondition(
            source.count == count,
            "source slice length (\(source.count)) does not match destination slice length (\(count))"
        )
        var i = startIndex
        for x in source {
            self[i] = x
            formIndex(after: &i)
        }
    }
}

extension Sequence {
    /// `iter.nth(n)`.
    func nth(_ n: Int) -> Element? {
        var i = 0
        for x in self {
            if i == n { return x }
            i += 1
        }
        return nil
    }

    /// `iter.step_by(n)`: lazily, the sequence may be endless.
    func stepBy(_ step: Int) -> RsStepBy<Self> { RsStepBy(base: makeIterator(), step: step) }

    /// `iter.scan(state, f)`: `f` changes the state and gives the next value, `nil` to stop.
    func rsScan<S, T>(_ initial: S, _ f: (inout S, Element) throws -> T?) rethrows -> [T] {
        var state = initial
        var out: [T] = []
        for x in self {
            guard let v = try f(&state, x) else { break }
            out.append(v)
        }
        return out
    }

    func minByKey<K: Comparable>(_ key: (Element) throws -> K) rethrows -> Element? {
        var best: (Element, K)?
        for x in self {
            let k = try key(x)
            if best == nil || k < best!.1 { best = (x, k) }
        }
        return best?.0
    }

    /// Like Rust, the last maximum wins.
    func maxByKey<K: Comparable>(_ key: (Element) throws -> K) rethrows -> Element? {
        var best: (Element, K)?
        for x in self {
            let k = try key(x)
            if best == nil || k >= best!.1 { best = (x, k) }
        }
        return best?.0
    }

    func minBy(_ compare: (Element, Element) throws -> Ordering) rethrows -> Element? {
        var best: Element?
        for x in self where try best == nil || compare(x, best!) == .less { best = x }
        return best
    }

    func maxBy(_ compare: (Element, Element) throws -> Ordering) rethrows -> Element? {
        var best: Element?
        for x in self where try best == nil || compare(x, best!) != .less { best = x }
        return best
    }

    /// `iter.reduce(f)`: `nil` for an empty sequence.
    func reduceFirst(_ f: (Element, Element) throws -> Element) rethrows -> Element? {
        var it = makeIterator()
        guard var acc = it.next() else { return nil }
        while let x = it.next() { acc = try f(acc, x) }
        return acc
    }

    /// `iter.position(p)`: how many items come before the first that matches.
    func rsPosition(where p: (Element) throws -> Bool) rethrows -> Int? {
        var i = 0
        for x in self {
            if try p(x) { return i }
            i += 1
        }
        return nil
    }

    /// `iter.rposition(p)`.
    func rsRposition(where p: (Element) throws -> Bool) rethrows -> Int? {
        var found: Int? = nil
        var i = 0
        for x in self {
            if try p(x) { found = i }
            i += 1
        }
        return found
    }

    func partitioned(_ p: (Element) throws -> Bool) rethrows -> ([Element], [Element]) {
        var yes: [Element] = [], no: [Element] = []
        for x in self { if try p(x) { yes.append(x) } else { no.append(x) } }
        return (yes, no)
    }
}

/// `iter.step_by(n)`.
struct RsStepBy<Base: Sequence>: Sequence, IteratorProtocol {
    var base: Base.Iterator
    let step: Int
    var started = false

    init(base: Base.Iterator, step: Int) {
        self.base = base
        self.step = step
    }

    mutating func next() -> Base.Element? {
        if !started {
            started = true
            return base.next()
        }
        for _ in 1..<step {
            if base.next() == nil { return nil }
        }
        return base.next()
    }
}

extension Set where Element: Comparable {
    /// `set.pop_first()` of a `BTreeSet`: removes and returns the smallest element.
    mutating func rsPopFirst() -> Element? {
        guard let m = self.min() else { return nil }
        remove(m)
        return m
    }

    /// `set.pop_last()`.
    mutating func rsPopLast() -> Element? {
        guard let m = self.max() else { return nil }
        remove(m)
        return m
    }

    /// `set.split_off(&x)` of a `BTreeSet`: the elements from `x` on, removed from the set.
    mutating func rsSplitOff(_ from: Element) -> Set<Element> {
        let out = filter { $0 >= from }
        subtract(out)
        return out
    }
}

extension Array where Element: Equatable {
    mutating func dedup() {
        var out: [Element] = []
        for x in self where out.last != x { out.append(x) }
        self = out
    }
}

"#;

const BINARY_SEARCH: &str = r#"
extension Array {
    /// `v.binary_search_by(|x| x.cmp(&target))`: Rust's own search, so that the same match
    /// is found among equal elements; `.failure` carries the insertion point.
    func binarySearchBy(_ compare: (Element) -> Ordering) -> Result<Int, Int> {
        var size = count
        if size == 0 { return .failure(0) }
        var base = 0
        while size > 1 {
            let half = size / 2
            let mid = base + half
            if compare(self[mid]) != .greater { base = mid }
            size -= half
        }
        let cmp = compare(self[base])
        if cmp == .equal { return .success(base) }
        return .failure(base + (cmp == .less ? 1 : 0))
    }

    /// `v.binary_search_by_key(&k, |x| x.key)`.
    func binarySearchByKey<K: Comparable>(_ key: K, _ of: (Element) -> K) -> Result<Int, Int> {
        binarySearchBy { x in
            let k = of(x)
            return k == key ? .equal : (k < key ? .less : .greater)
        }
    }
}

extension Array where Element: Comparable {
    /// `v.binary_search(&x)`: `.success(index)` or `.failure(insertion point)`.
    func binarySearch(_ x: Element) -> Result<Int, Int> {
        binarySearchBy { $0 == x ? .equal : ($0 < x ? .less : .greater) }
    }
}
"#;

const ITER: &str = r#"
/// `std::iter::successors(first, next)`.
func rsSuccessors<T>(_ first: T?, _ next: @escaping (T) -> T?) -> UnfoldSequence<T, T?> {
    sequence(state: first) { state in
        guard let current = state else { return nil }
        state = next(current)
        return current
    }
}

/// `std::iter::from_fn(f)`: the values until `f` gives `None`.
func rsFromFn<T>(_ next: @escaping () -> T?) -> AnyIterator<T> { AnyIterator(next) }

/// `std::iter::repeat_with(f)`.
func rsRepeatWith<T>(_ make: @escaping () -> T) -> AnyIterator<T> { AnyIterator { make() } }

/// `std::iter::repeat(x)`.
func rsRepeat<T>(_ x: T) -> UnfoldFirstSequence<T> {
    sequence(first: x) { _ in x }
}

/// `iter.cycle()`: the items again and again (nothing for an empty sequence).
func rsCycle<S: Sequence>(_ items: S) -> UnfoldSequence<S.Element, Int> {
    let all = Array(items)
    return sequence(state: 0) { i in
        guard !all.isEmpty else { return nil }
        defer { i += 1 }
        return all[i % all.count]
    }
}

/// A Rust iterator kept in a variable: `it.next()`, `it.peek()`.
struct RsIter<Element>: Sequence, IteratorProtocol {
    private var base: AnyIterator<Element>
    private var buffered: Element? = nil
    private var hasBuffered = false
    /// Once a copy was asked for (`it.clone()`): the elements left, owned by each copy.
    private var items: [Element]? = nil
    private var index = 0

    init<S: Sequence>(_ items: S) where S.Element == Element {
        base = AnyIterator(items.makeIterator())
    }

    mutating func next() -> Element? {
        if hasBuffered {
            hasBuffered = false
            defer { buffered = nil }
            return buffered
        }
        if let items {
            guard index < items.count else { return nil }
            defer { index += 1 }
            return items[index]
        }
        return base.next()
    }

    /// Gathers what is left into an array, so that copies of the iterator are independent.
    private mutating func materialize() {
        guard items == nil else { return }
        var rest: [Element] = []
        if hasBuffered, let b = buffered { rest.append(b) }
        hasBuffered = false
        buffered = nil
        while let x = base.next() { rest.append(x) }
        items = rest
        index = 0
    }

    /// `it.clone()`, `it.as_str()`: what is left, without consuming it.
    mutating func remaining() -> [Element] {
        materialize()
        return Array((items ?? [])[index...])
    }

    mutating func peek() -> Element? {
        if !hasBuffered {
            buffered = base.next()
            hasBuffered = true
        }
        return buffered
    }

    mutating func nextIf(_ test: (Element) -> Bool) -> Element? {
        guard let x = peek(), test(x) else { return nil }
        hasBuffered = false
        buffered = nil
        return x
    }

    mutating func nth(_ n: Int) -> Element? {
        for _ in 0..<n {
            if next() == nil { return nil }
        }
        return next()
    }

    /// The items not consumed yet.
    mutating func rest() -> [Element] {
        var out: [Element] = []
        while let x = next() { out.append(x) }
        return out
    }

    /// `it.by_ref().take(n)`: the next `n` items, the others left to the iterator.
    mutating func takeFirst(_ n: Int) -> [Element] {
        var out: [Element] = []
        while out.count < n, let x = next() { out.append(x) }
        return out
    }

    /// `it.by_ref().take_while(p)`: the items up to the first refused, which is consumed.
    mutating func takeWhile(_ keep: (Element) throws -> Bool) rethrows -> [Element] {
        var out: [Element] = []
        while let x = next() {
            if try !keep(x) { break }
            out.append(x)
        }
        return out
    }

    /// `it.find(p)`: the first item accepted, the following ones left to the iterator.
    mutating func findFirst(_ test: (Element) throws -> Bool) rethrows -> Element? {
        while let x = next() {
            if try test(x) { return x }
        }
        return nil
    }

    /// `it.position(p)`: the rank of the first item accepted.
    mutating func findPosition(_ test: (Element) throws -> Bool) rethrows -> Int? {
        var i = 0
        while let x = next() {
            if try test(x) { return i }
            i += 1
        }
        return nil
    }

    /// `it.any(p)`, `it.all(p)`: stop at the first decisive item.
    mutating func anyItem(_ test: (Element) throws -> Bool) rethrows -> Bool {
        try findFirst(test) != nil
    }

    mutating func allItems(_ test: (Element) throws -> Bool) rethrows -> Bool {
        try findFirst { try !test($0) } == nil
    }

    func makeIterator() -> RsIter<Element> { self }
}
"#;

const MAP: &str = r#"
extension Dictionary {
    mutating func retain(_ keep: ((key: Key, value: Value)) throws -> Bool) rethrows {
        self = try filter(keep)
    }

    /// `map.drain()`: every entry, the map left empty.
    mutating func drain() -> [(Key, Value)] {
        defer { removeAll() }
        return map { ($0.key, $0.value) }
    }
}

extension Dictionary where Key: Comparable {
    /// `map.pop_first()` of a `BTreeMap`: the entry with the smallest key, removed.
    mutating func rsPopFirst() -> (Key, Value)? {
        guard let k = keys.min() else { return nil }
        return (k, removeValue(forKey: k)!)
    }

    mutating func rsPopLast() -> (Key, Value)? {
        guard let k = keys.max() else { return nil }
        return (k, removeValue(forKey: k)!)
    }

    /// `map.split_off(&k)` of a `BTreeMap`: the entries from `k` on, removed from the map.
    mutating func rsSplitOff(_ from: Key) -> [Key: Value] {
        let out = filter { $0.key >= from }
        for k in out.keys { removeValue(forKey: k) }
        return out
    }
}

extension Set {
    /// `set.drain()`: every element, the set left empty.
    mutating func drain() -> [Element] {
        defer { removeAll() }
        return Array(self)
    }
}
"#;

const OPTION_CMP: &str = r#"
/// `Some(1) < None`: options ordered as Rust does, `None` before any `Some`.
extension Optional: @retroactive Comparable where Wrapped: Comparable {
    public static func < (lhs: Wrapped?, rhs: Wrapped?) -> Bool {
        switch (lhs, rhs) {
        case (nil, nil): return false
        case (nil, _): return true
        case (_, nil): return false
        case let (a?, b?): return a < b
        }
    }
}
"#;

const OPTION: &str = r#"
extension Optional {
    /// `o.take()`.
    mutating func take() -> Wrapped? {
        defer { self = nil }
        return self
    }

    /// `o.replace(v)`.
    mutating func replace(_ value: Wrapped) -> Wrapped? {
        defer { self = value }
        return self
    }

    func filter(_ keep: (Wrapped) throws -> Bool) rethrows -> Wrapped? {
        guard let v = self, try keep(v) else { return nil }
        return v
    }

    func okOr<E: Error>(_ error: E) -> Result<Wrapped, E> {
        map { .success($0) } ?? .failure(error)
    }

    /// `a.xor(b)`: the one that is `Some`, if exactly one is.
    func xor(_ other: Wrapped?) -> Wrapped? {
        switch (self, other) {
        case (.some, .none): self
        case (.none, .some): other
        default: nil
        }
    }

    /// `a.zip(b)`.
    func zip<U>(_ other: U?) -> (Wrapped, U)? {
        if let a = self, let b = other { (a, b) } else { nil }
    }

    /// `o.take_if(|v| …)`: the value, taken out, when it passes the test.
    mutating func takeIf(_ keep: (Wrapped) throws -> Bool) rethrows -> Wrapped? {
        guard let v = self, try keep(v) else { return nil }
        self = nil
        return v
    }
}

/// `iter.collect::<Option<Vec<_>>>()`: every value, or `nil` at the first `None`.
func rsAllSome<T>(_ values: some Sequence<T?>) -> [T]? {
    var out: [T] = []
    for v in values {
        guard let v else { return nil }
        out.append(v)
    }
    return out
}
"#;

const RESULT: &str = r#"
extension Result {
    var isOk: Bool { if case .success = self { true } else { false } }
    var isErr: Bool { !isOk }
    func ok() -> Success? { try? get() }
    func err() -> Failure? { if case .failure(let e) = self { e } else { nil } }
    func unwrapOrElse(_ f: (Failure) -> Success) -> Success {
        switch self {
        case .success(let v): v
        case .failure(let e): f(e)
        }
    }

    /// `r.and_then(f)`.
    func andThen<T>(_ f: (Success) throws(Failure) -> T) -> Result<T, Failure> {
        switch self {
        case .success(let v):
            do { return .success(try f(v)) } catch { return .failure(error) }
        case .failure(let e):
            return .failure(e)
        }
    }

    /// `r.transpose()` of a `Result<Option<T>, E>`.
    func transpose<T>() -> Result<T, Failure>? where Success == T? {
        switch self {
        case .success(nil): nil
        case .success(let v?): .success(v)
        case .failure(let e): .failure(e)
        }
    }

    /// `r.or_else(f)`.
    func orElse<F: Error>(_ f: (Failure) throws(F) -> Success) -> Result<Success, F> {
        switch self {
        case .success(let v):
            return .success(v)
        case .failure(let e):
            do { return .success(try f(e)) } catch { return .failure(error) }
        }
    }
}

extension Optional {
    /// `o.transpose()` of an `Option<Result<T, E>>`.
    func transpose<T, E: Error>() -> Result<T?, E> where Wrapped == Result<T, E> {
        switch self {
        case nil: .success(nil)
        case .some(.success(let v)): .success(v)
        case .some(.failure(let e)): .failure(e)
        }
    }
}
"#;

const TRY_FROM: &str = r#"
/// `u8::try_from(x)`: `TryFromIntError` when the value does not fit.
struct RsTryFromIntError: Error, Equatable, RsDebugText, CustomStringConvertible {
    let negative: Bool
    func rsDebugText(pretty: Bool, style: RsDebugStyle) -> String {
        rsDebugRecord("TryFromIntError", [("", RsDebugBare(rsDebugText: negative ? "NegOverflow" : "PosOverflow"))], pretty: pretty, style: style)
    }
    var description: String { "out of range integral type conversion attempted" }
}

func rsTryFrom<T: FixedWidthInteger>(_: T.Type, _ x: some BinaryInteger, bits: Int = T.bitWidth) -> Result<T, RsTryFromIntError> {
    let (min, max) = rsIntBounds(T.self, bits)
    if x < 0 && (!T.isSigned || x < min) { return .failure(RsTryFromIntError(negative: true)) }
    guard let v = T(exactly: x), v >= min, v <= max else { return .failure(RsTryFromIntError(negative: false)) }
    return .success(v)
}
"#;

const INT: &str = r#"
extension BinaryInteger {
    /// `n.isqrt()`: the integer square root.
    func isqrt() -> Self {
        precondition(self >= 0, "argument of integer square root cannot be negative")
        if self < 2 { return self }
        var x = Self(Double(self).squareRoot())
        while x * x > self { x -= 1 }
        while (x + 1) * (x + 1) <= self { x += 1 }
        return x
    }

    /// `x.pow(n)`.
    func pow(_ exponent: some BinaryInteger) -> Self {
        var result: Self = 1
        var base = self
        var e = Int(exponent)
        while e > 0 {
            if e & 1 == 1 { result *= base }
            e >>= 1
            if e > 0 { base *= base }
        }
        return result
    }

    func remEuclid(_ n: Self) -> Self { let r = self % n; return r < 0 ? r + (n < 0 ? 0 - n : n) : r }
    func divEuclid(_ n: Self) -> Self { let q = self / n; return self % n < 0 ? (n > 0 ? q - 1 : q + 1) : q }
    func absDiff(_ other: Self) -> Self { self > other ? self - other : other - self }
    /// `x.div_ceil(n)`: the quotient, rounded towards positive infinity.
    func divCeil(_ n: Self) -> Self {
        let q = self / n
        let r = self % n
        return (r != 0 && ((r > 0) == (n > 0))) ? q + 1 : q
    }
    /// `x.next_power_of_two()`: the smallest power of two that is at least `x`.
    func nextPowerOfTwo() -> Self {
        var p: Self = 1
        while p < self { p *= 2 }
        return p
    }
    /// `x.ilog2()`: the floor of the base-2 logarithm.
    func ilog2() -> UInt32 {
        precondition(self > 0, "argument of integer logarithm must be positive")
        var n = self
        var r: UInt32 = 0
        while n > 1 { n /= 2; r += 1 }
        return r
    }
    /// `x.ilog10()`: the floor of the base-10 logarithm.
    func ilog10() -> UInt32 {
        precondition(self > 0, "argument of integer logarithm must be positive")
        var n = self
        var r: UInt32 = 0
        while n >= 10 { n /= 10; r += 1 }
        return r
    }
}

extension FixedWidthInteger {
    /// `x.to_be_bytes()`, `x.to_le_bytes()`.
    func toBeBytes() -> [UInt8] { withUnsafeBytes(of: bigEndian, Array.init) }
    func toLeBytes() -> [UInt8] { withUnsafeBytes(of: littleEndian, Array.init) }
    func toNeBytes() -> [UInt8] { withUnsafeBytes(of: self, Array.init) }

    /// `T::from_be_bytes(bytes)`, `T::from_le_bytes(bytes)`.
    init(rsBeBytes bytes: [UInt8]) {
        self = bytes.reduce(0) { ($0 << 8) | Self(truncatingIfNeeded: $1) }
    }

    init(rsLeBytes bytes: [UInt8]) {
        self = bytes.reversed().reduce(0) { ($0 << 8) | Self(truncatingIfNeeded: $1) }
    }

    /// `x.reverse_bits()`.
    func reverseBits() -> Self {
        var v = self
        var r: Self = 0
        for _ in 0..<Self.bitWidth {
            r = (r << 1) | (v & 1)
            v >>= 1
        }
        return r
    }
}

extension BinaryFloatingPoint {
    /// `x.signum()`: `1` for positive numbers and `+0.0`, `-1` for negative ones and `-0.0`.
    func rsSignum() -> Self {
        isNaN ? self : (sign == .minus ? -1 : 1)
    }

    /// `x.rem_euclid(n)`.
    func remEuclid(_ n: Self) -> Self {
        let r = self.truncatingRemainder(dividingBy: n)
        return r < 0 ? r + n.magnitude : r
    }

    /// `x.div_euclid(n)`.
    func divEuclid(_ n: Self) -> Self {
        let q = (self / n).rounded(.towardZero)
        if self.truncatingRemainder(dividingBy: n) < 0 { return n > 0 ? q - 1 : q + 1 }
        return q
    }
}

extension FixedWidthInteger {
    /// `x.rotate_left(n)`.
    func rotateLeft(_ n: some BinaryInteger) -> Self {
        let k = Int(n) % Self.bitWidth
        if k == 0 { return self }
        let bits = Magnitude(truncatingIfNeeded: self)
        return Self(truncatingIfNeeded: (bits << k) | (bits >> (Self.bitWidth - k)))
    }

    /// `x.rotate_right(n)`.
    func rotateRight(_ n: some BinaryInteger) -> Self {
        rotateLeft(Self.bitWidth - Int(n) % Self.bitWidth)
    }
}

/// `(i32, bool)` of an `overflowing_*` operation, as Swift's `Int`.
func rsWiden(_ r: (Int32, Bool)) -> (Int, Bool) { (Int(r.0), r.1) }

/// `x as i32` for a float: toward zero, saturating, 0 for NaN.
func rsInt<T: FixedWidthInteger>(_: T.Type, _ x: some BinaryFloatingPoint) -> T {
    if x.isNaN { return 0 }
    if x >= .init(T.max) { return T.max }
    if x <= .init(T.min) { return T.min }
    return T(x)
}

extension FixedWidthInteger {
    func wrappingNeg() -> Self { 0 &- self }
    func wrappingAbs() -> Self { self < 0 ? 0 &- self : self }
    func checkedAbs() -> Self? { Self.isSigned && self == Self.min ? nil : (self < 0 ? 0 - self : self) }
    func checkedNeg() -> Self? { Self.isSigned ? (self == Self.min ? nil : 0 - self) : (self == 0 ? 0 : nil) }
    func saturatingNeg() -> Self { Self.isSigned && self == Self.min ? Self.max : 0 - self }
    func saturatingAbs() -> Self { Self.isSigned && self == Self.min ? Self.max : (self < 0 ? 0 - self : self) }
    func checkedAdd(_ o: Self) -> Self? { let (v, overflow) = addingReportingOverflow(o); return overflow ? nil : v }
    func checkedSub(_ o: Self) -> Self? { let (v, overflow) = subtractingReportingOverflow(o); return overflow ? nil : v }
    func checkedMul(_ o: Self) -> Self? { let (v, overflow) = multipliedReportingOverflow(by: o); return overflow ? nil : v }
    func checkedDiv(_ o: Self) -> Self? { o == 0 ? nil : dividedReportingOverflow(by: o).overflow ? nil : self / o }
    func checkedRem(_ o: Self) -> Self? { o == 0 ? nil : remainderReportingOverflow(dividingBy: o).overflow ? nil : self % o }
    func saturatingAdd(_ o: Self) -> Self { let (v, overflow) = addingReportingOverflow(o); return overflow ? (o > 0 ? .max : .min) : v }
    func saturatingSub(_ o: Self) -> Self { let (v, overflow) = subtractingReportingOverflow(o); return overflow ? (o > 0 ? .min : .max) : v }
    func saturatingMul(_ o: Self) -> Self { let (v, overflow) = multipliedReportingOverflow(by: o); return overflow ? ((self < 0) != (o < 0) ? .min : .max) : v }
    func overflowingAdd(_ o: Self) -> (Self, Bool) { let r = addingReportingOverflow(o); return (r.partialValue, r.overflow) }
    func overflowingSub(_ o: Self) -> (Self, Bool) { let r = subtractingReportingOverflow(o); return (r.partialValue, r.overflow) }
    func overflowingMul(_ o: Self) -> (Self, Bool) { let r = multipliedReportingOverflow(by: o); return (r.partialValue, r.overflow) }
    func overflowingNeg() -> (Self, Bool) { (0 &- self, Self.isSigned ? self == Self.min : self != 0) }
    /// `x.checked_shl(n)`, `wrapping_shl`, `overflowing_shl` and the right shifts: a shift
    /// of the bit width or more is refused, wrapped or reported.
    func checkedShl(_ n: some BinaryInteger) -> Self? { Int(n) < Self.bitWidth ? self << Int(n) : nil }
    func checkedShr(_ n: some BinaryInteger) -> Self? { Int(n) < Self.bitWidth ? self >> Int(n) : nil }
    func wrappingShl(_ n: some BinaryInteger) -> Self { self << (Int(n) % Self.bitWidth) }
    func wrappingShr(_ n: some BinaryInteger) -> Self { self >> (Int(n) % Self.bitWidth) }
    func overflowingShl(_ n: some BinaryInteger) -> (Self, Bool) { (wrappingShl(n), Int(n) >= Self.bitWidth) }
    func overflowingShr(_ n: some BinaryInteger) -> (Self, Bool) { (wrappingShr(n), Int(n) >= Self.bitWidth) }
    func overflowingAbs() -> (Self, Bool) { self < 0 ? overflowingNeg() : (self, false) }
    /// `x.checked_pow(n)` and its wrapping, saturating and overflowing forms.
    func overflowingPow(_ exponent: some BinaryInteger) -> (Self, Bool) {
        var result: Self = 1
        var base = self
        var e = Int(exponent)
        var overflow = false
        while e > 0 {
            if e & 1 == 1 {
                let r = result.multipliedReportingOverflow(by: base)
                result = r.partialValue
                overflow = overflow || r.overflow
            }
            e >>= 1
            if e > 0 {
                let b = base.multipliedReportingOverflow(by: base)
                base = b.partialValue
                overflow = overflow || b.overflow
            }
        }
        return (result, overflow)
    }
    func checkedPow(_ exponent: some BinaryInteger) -> Self? { let (v, overflow) = overflowingPow(exponent); return overflow ? nil : v }
    func wrappingPow(_ exponent: some BinaryInteger) -> Self { overflowingPow(exponent).0 }
    func saturatingPow(_ exponent: some BinaryInteger) -> Self {
        let (v, overflow) = overflowingPow(exponent)
        if !overflow { return v }
        return self < 0 && Int(exponent) & 1 == 1 ? .min : .max
    }
}
"#;

const ASYNC_RESULT: &str = r#"
/// The `Result` of an async function that throws (`match f().await { … }`).
func rsAsyncResult<T, E: Error>(_ body: () async throws(E) -> T) async -> Result<T, E> {
    do {
        return .success(try await body())
    } catch {
        return .failure(error)
    }
}
"#;

const MEM: &str = r#"
/// `std::mem::replace` / `std::mem::take`.
func rsReplace<T>(_ place: inout T, _ value: T) -> T {
    let old = place
    place = value
    return old
}
"#;

const DEFAULT: &str = r#"
/// `T: Default` in generic code: the types with a default value.
protocol RsDefault { init() }
extension Int: RsDefault {}
extension Int8: RsDefault {}
extension Int16: RsDefault {}
extension Int32: RsDefault {}
extension UInt: RsDefault {}
extension UInt8: RsDefault {}
extension UInt16: RsDefault {}
extension UInt32: RsDefault {}
extension Double: RsDefault {}
extension Float: RsDefault {}
extension Bool: RsDefault {}
extension String: RsDefault {}
extension Array: RsDefault {}
extension Dictionary: RsDefault {}
extension Set: RsDefault {}
extension Character: RsDefault { init() { self = "\0" } }
extension Optional: RsDefault { init() { self = nil } }
"#;

const INTO_DOUBLE: &str = r#"
/// `T: Into<f64>` in generic code: the numbers that convert to a `Double`.
protocol RsIntoDouble { var rsDouble: Double { get } }
extension Int: RsIntoDouble { var rsDouble: Double { Double(self) } }
extension Int8: RsIntoDouble { var rsDouble: Double { Double(self) } }
extension Int16: RsIntoDouble { var rsDouble: Double { Double(self) } }
extension Int32: RsIntoDouble { var rsDouble: Double { Double(self) } }
extension UInt: RsIntoDouble { var rsDouble: Double { Double(self) } }
extension UInt8: RsIntoDouble { var rsDouble: Double { Double(self) } }
extension UInt16: RsIntoDouble { var rsDouble: Double { Double(self) } }
extension UInt32: RsIntoDouble { var rsDouble: Double { Double(self) } }
extension Float: RsIntoDouble { var rsDouble: Double { Double(self) } }
extension Double: RsIntoDouble { var rsDouble: Double { self } }
"#;

const REFCELL: &str = r#"
/// `RefCell<T>`, `Cell<T>`, `Mutex<T>`, `RwLock<T>`: a value shared by reference and changed
/// in place, as the clones of an `Rc` see it.
final class RsRefCell<T> {
    var value: T
    /// The Rust type, for `{:?}`.
    let kind: String

    init(_ value: T, kind: String = "RefCell") {
        self.value = value
        self.kind = kind
    }

    /// `cell.replace(x)`, `cell.take()`.
    func replace(_ newValue: T) -> T { rsReplace(&value, newValue) }
}

extension RsRefCell: Equatable where T: Equatable {
    static func == (lhs: RsRefCell, rhs: RsRefCell) -> Bool { lhs.value == rhs.value }
}

extension RsRefCell: Comparable where T: Comparable {
    static func < (lhs: RsRefCell, rhs: RsRefCell) -> Bool { lhs.value < rhs.value }
}

extension RsRefCell: RsDebugNamed, CustomReflectable {
    var rsDebugName: String { kind }
    var customMirror: Mirror { Mirror(self, children: ["value": value], displayStyle: .struct) }
}
"#;

const EPRINT: &str = r#"
/// `eprintln!`: a line on the standard error. The standard output is flushed first, so that
/// the lines of both keep their order, as with Rust's line-buffered output.
func rsEprint(_ text: String, terminator: String = "\n") {
    fflush(nil)
    FileHandle.standardError.write(Data((text + terminator).utf8))
}
"#;

const STRING_ERROR: &str = r#"
/// Rust functions returning `Result<T, String>` throw their message.
extension String: @retroactive Error {}
"#;

const INT_ERROR: &str = r#"
/// Rust functions returning `Result<T, i32>` throw the number.
extension Int: @retroactive Error {}
extension Int8: @retroactive Error {}
extension Int16: @retroactive Error {}
extension Int32: @retroactive Error {}
extension UInt: @retroactive Error {}
extension UInt8: @retroactive Error {}
extension UInt16: @retroactive Error {}
extension UInt32: @retroactive Error {}
"#;

const FLOAT_ERROR: &str = r#"
/// Rust functions returning `Result<T, f64>` throw the number.
extension Double: @retroactive Error {}
extension Float: @retroactive Error {}
"#;

const BOOL_ERROR: &str = r#"
/// Rust functions returning `Result<T, bool>` throw the flag.
extension Bool: @retroactive Error {}
"#;

const CHAR_ERROR: &str = r#"
/// Rust functions returning `Result<T, char>` throw the character.
extension Character: @retroactive Error {}
"#;

const COW: &str = r##"
/// `Cow<'a, T>`: a value borrowed or owned; it reads, prints and compares as the value.
enum RsCow<T> {
    case borrowed(T)
    case owned(T)

    /// `&*cow`, the value behind it.
    var value: T {
        switch self {
        case .borrowed(let v), .owned(let v): v
        }
    }

    func deref() -> T { value }

    var isBorrowed: Bool { if case .borrowed = self { true } else { false } }
    var isOwned: Bool { !isBorrowed }

    /// `cow.to_mut()`: the value, owned from then on.
    var rsMut: T {
        get { value }
        set { self = .owned(newValue) }
    }
}

extension RsCow: CustomStringConvertible {
    var description: String { String(describing: value) }
}

extension RsCow: RsDebugText {
    func rsDebugText(pretty: Bool, style: RsDebugStyle) -> String {
        rsDebug(value, pretty: pretty, style: style)
    }
}

extension RsCow: Equatable where T: Equatable {
    static func == (lhs: RsCow, rhs: RsCow) -> Bool { lhs.value == rhs.value }
    static func == (lhs: RsCow, rhs: T) -> Bool { lhs.value == rhs }
    static func == (lhs: T, rhs: RsCow) -> Bool { lhs == rhs.value }
}

extension RsCow: Hashable where T: Hashable {
    func hash(into hasher: inout Hasher) { hasher.combine(value) }
}
"##;

const THREAD: &str = r#"
/// `std::thread`: a spawned thread is a task run when something waits for it (its `join`,
/// a `recv` on an empty channel, the end of its `thread::scope`, a `sleep` or `yield_now`),
/// in the order the tasks were spawned, so that the program prints what Rust's prints when
/// its threads are joined, without data races.
final class RsTask: @unchecked Sendable {
    var run: (() -> Void)?
    init(_ run: @escaping () -> Void) { self.run = run }

    func finish() {
        if let run {
            self.run = nil
            run()
        }
    }
}

/// The tasks spawned and not run yet (a static, initialized on first use: the globals of a
/// `main.swift` are only set when the top-level code reaches them).
enum RsTasks {
    nonisolated(unsafe) static var pending: [RsTask] = []
}

/// Runs the first task not run yet: `false` when none is left.
@discardableResult
func rsRunPending() -> Bool {
    while !RsTasks.pending.isEmpty {
        let task = RsTasks.pending.removeFirst()
        if task.run != nil {
            task.finish()
            return true
        }
    }
    return false
}

/// `JoinHandle<T>`.
final class RsJoinHandle<T>: @unchecked Sendable {
    var task: RsTask? = nil
    var result: T? = nil

    var isFinished: Bool { task?.run == nil }

    func join() -> Result<T, RsJoinError> {
        task?.finish()
        return .success(result!)
    }
}

/// The error of `join` (a thread that panicked, which stops the Swift program instead).
struct RsJoinError: Error, CustomStringConvertible, RsDebugCustom {
    var description: String { "Any { .. }" }
    var rsDebugText: String { "Any { .. }" }
}

func rsSpawn<T>(_ body: @escaping () -> T) -> RsJoinHandle<T> {
    let handle = RsJoinHandle<T>()
    let task = RsTask { handle.result = body() }
    handle.task = task
    RsTasks.pending.append(task)
    return handle
}

/// `thread::sleep(d)`: the other threads progress meanwhile.
func rsSleep(_ d: RsDuration) {
    while rsRunPending() {}
    Thread.sleep(forTimeInterval: d.asSecsF64())
}

func rsYieldNow() {
    rsRunPending()
}

/// `thread::scope(|s| …)`: the threads of the scope are joined at its end.
final class RsScope: @unchecked Sendable {
    var tasks: [RsTask] = []

    func spawn<T>(_ body: @escaping () -> T) -> RsJoinHandle<T> {
        let handle = rsSpawn(body)
        if let task = handle.task { tasks.append(task) }
        return handle
    }
}

func rsThreadScope<T>(_ body: (RsScope) -> T) -> T {
    let scope = RsScope()
    let result = body(scope)
    for task in scope.tasks { task.finish() }
    return result
}

/// `mpsc::channel()`.
final class RsChannelState<T>: @unchecked Sendable {
    var queue: [T] = []
    var senders = 0
}

final class RsSender<T>: @unchecked Sendable {
    let state: RsChannelState<T>
    private var closed = false

    init(_ state: RsChannelState<T>) {
        self.state = state
        state.senders += 1
    }

    deinit { close() }

    /// `drop(tx)`.
    func close() {
        if !closed {
            closed = true
            state.senders -= 1
        }
    }

    func clone() -> RsSender<T> { RsSender(state) }

    func send(_ value: T) -> Result<Void, RsSendError> {
        state.queue.append(value)
        return .success(())
    }
}

final class RsReceiver<T>: Sequence, IteratorProtocol, @unchecked Sendable {
    let state: RsChannelState<T>
    init(_ state: RsChannelState<T>) { self.state = state }

    /// Waits for a value: the pending threads run until one is sent; none left means that
    /// every sender is gone (a Rust program waiting otherwise would never end).
    func recv() -> Result<T, RsRecvError> {
        while state.queue.isEmpty {
            if !rsRunPending() { return .failure(RsRecvError()) }
        }
        return .success(state.queue.removeFirst())
    }

    func tryRecv() -> Result<T, RsTryRecvError> {
        if state.queue.isEmpty { return .failure(state.senders > 0 ? .empty : .disconnected) }
        return .success(state.queue.removeFirst())
    }

    func recvTimeout() -> Result<T, RsRecvTimeoutError> {
        switch recv() {
        case .success(let v): .success(v)
        case .failure: .failure(.disconnected)
        }
    }

    func next() -> T? { try? recv().get() }

    /// `rx.try_iter()`: the values already sent.
    func tryIter() -> [T] {
        defer { state.queue.removeAll() }
        return state.queue
    }
}

func rsChannel<T>(_: T.Type) -> (RsSender<T>, RsReceiver<T>) {
    let state = RsChannelState<T>()
    return (RsSender(state), RsReceiver(state))
}

struct RsSendError: Error, CustomStringConvertible, RsDebugCustom {
    var description: String { "sending on a closed channel" }
    var rsDebugText: String { "SendError { .. }" }
}

struct RsRecvError: Error, Equatable, CustomStringConvertible, RsDebugCustom {
    var description: String { "receiving on a closed channel" }
    var rsDebugText: String { "RecvError" }
}

enum RsTryRecvError: Error, Equatable, CustomStringConvertible, RsDebugCustom {
    case empty, disconnected
    var description: String {
        self == .empty ? "receiving on an empty channel" : "receiving on an empty and disconnected channel"
    }
    var rsDebugText: String { self == .empty ? "Empty" : "Disconnected" }
}

enum RsRecvTimeoutError: Error, Equatable, CustomStringConvertible, RsDebugCustom {
    case timeout, disconnected
    var description: String {
        self == .timeout ? "timed out waiting on channel" : "channel is empty and sending half is closed"
    }
    var rsDebugText: String { self == .timeout ? "Timeout" : "Disconnected" }
}

/// `std::sync::atomic::Ordering`: one thread runs at a time, so every ordering is the same.
enum RsMemoryOrdering { case relaxed, release, acquire, acqRel, seqCst }

/// `AtomicUsize`, `AtomicBool`…: a shared value.
final class RsAtomic<T>: @unchecked Sendable, RsDebugText {
    var value: T
    init(_ value: T) { self.value = value }

    func load(_: RsMemoryOrdering = .seqCst) -> T { value }
    func store(_ v: T, _: RsMemoryOrdering = .seqCst) { value = v }
    func swap(_ v: T, _: RsMemoryOrdering = .seqCst) -> T {
        defer { value = v }
        return value
    }

    func rsDebugText(pretty: Bool, style: RsDebugStyle) -> String {
        rsDebug(value, pretty: pretty, style: style)
    }
}

/// The `Err(actual)` of `compare_exchange`: a value, which Swift's `Result` wants as an error.
struct RsValueError<T>: Error, @unchecked Sendable, RsDebugText, CustomStringConvertible {
    let value: T
    var description: String { String(describing: value) }
    func rsDebugText(pretty: Bool, style: RsDebugStyle) -> String {
        rsDebug(value, pretty: pretty, style: style)
    }
}

extension RsAtomic where T: Equatable {
    func compareExchange(_ current: T, _ new: T, _: RsMemoryOrdering = .seqCst, _: RsMemoryOrdering = .seqCst) -> Result<T, RsValueError<T>> {
        if value == current {
            value = new
            return .success(current)
        }
        return .failure(RsValueError(value: value))
    }
}

extension RsAtomic where T: FixedWidthInteger {
    func fetchAdd(_ v: T, _: RsMemoryOrdering = .seqCst) -> T { defer { value &+= v }; return value }
    func fetchSub(_ v: T, _: RsMemoryOrdering = .seqCst) -> T { defer { value &-= v }; return value }
    func fetchAnd(_ v: T, _: RsMemoryOrdering = .seqCst) -> T { defer { value &= v }; return value }
    func fetchOr(_ v: T, _: RsMemoryOrdering = .seqCst) -> T { defer { value |= v }; return value }
    func fetchXor(_ v: T, _: RsMemoryOrdering = .seqCst) -> T { defer { value ^= v }; return value }
    func fetchNand(_ v: T, _: RsMemoryOrdering = .seqCst) -> T { defer { value = ~(value & v) }; return value }
    func fetchMax(_ v: T, _: RsMemoryOrdering = .seqCst) -> T { defer { value = Swift.max(value, v) }; return value }
    func fetchMin(_ v: T, _: RsMemoryOrdering = .seqCst) -> T { defer { value = Swift.min(value, v) }; return value }
}

extension RsAtomic where T == Bool {
    func fetchAnd(_ v: Bool, _: RsMemoryOrdering = .seqCst) -> Bool { defer { value = value && v }; return value }
    func fetchOr(_ v: Bool, _: RsMemoryOrdering = .seqCst) -> Bool { defer { value = value || v }; return value }
    func fetchXor(_ v: Bool, _: RsMemoryOrdering = .seqCst) -> Bool { defer { value = value != v }; return value }
    func fetchNand(_ v: Bool, _: RsMemoryOrdering = .seqCst) -> Bool { defer { value = !(value && v) }; return value }
}
"#;

const PANIC: &str = r#"
/// A Rust panic: what Rust's default hook writes to the standard error, then exit status 101.
func rsPanic(_ message: String, _ location: String) -> Never {
    fflush(nil)
    let text = "\nthread 'main' (\(getpid())) panicked at \(location):\n\(message)\nnote: run with `RUST_BACKTRACE=1` environment variable to display a backtrace\n"
    FileHandle.standardError.write(Data(text.utf8))
    exit(101)
}

extension String {
    /// `&s[a..b]`, with Rust's panics when a bound is past the end or inside a character.
    func rsSlice(_ start: Int, _ end: Int, at location: String) -> String {
        let bytes = Array(utf8)
        rsCheckStrSlice(bytes, start, end, location)
        return String(decoding: bytes[start..<end], as: UTF8.self)
    }
}

func rsCheckStrSlice(_ bytes: [UInt8], _ start: Int, _ end: Int, _ location: String) {
    let len = bytes.count
    if start > len || end > len {
        let (which, oob) = start > len ? ("start", start) : ("end", end)
        rsPanic("\(which) byte index \(oob) is out of bounds for string of length \(len)", location)
    }
    if start > end { rsPanic("byte range starts at \(start) but ends at \(end)", location) }
    func boundary(_ i: Int) -> Bool { i == len || (bytes[i] & 0xC0) != 0x80 }
    let (which, index) = !boundary(start) ? ("start", start) : ("end", end)
    if boundary(index) { return }
    var lo = index
    while !boundary(lo) { lo -= 1 }
    var hi = lo + 1
    while hi < len && !boundary(hi) { hi += 1 }
    let ch = String(decoding: bytes[lo..<hi], as: UTF8.self)
    rsPanic("\(which) byte index \(index) is not a char boundary; it is inside \(rsDebug(Character(ch))) (bytes \(lo)..\(hi) of string)", location)
}

/// `a / b` and `a % b` of integers.
func rsDiv<T: BinaryInteger>(_ a: T, _ b: T, _ location: String) -> T {
    if b == 0 { rsPanic("attempt to divide by zero", location) }
    return a / b
}

func rsRem<T: BinaryInteger>(_ a: T, _ b: T, _ location: String) -> T {
    if b == 0 { rsPanic("attempt to calculate the remainder with a divisor of zero", location) }
    return a % b
}

extension Array {
    /// `v[i]`: Rust's panic when `i` is out of bounds.
    subscript(_ i: Int, at location: String) -> Element {
        get {
            if i < 0 || i >= count { rsPanic("index out of bounds: the len is \(count) but the index is \(i)", location) }
            return self[i]
        }
        set {
            if i < 0 || i >= count { rsPanic("index out of bounds: the len is \(count) but the index is \(i)", location) }
            self[i] = newValue
        }
    }

    /// `&v[a..b]`, `&v[a..]`, `&v[..b]`: Rust's panics when the bounds are out of range.
    subscript(from start: Int?, to end: Int?, at location: String) -> ArraySlice<Element> {
        get { self[rsCheckedRange(start, end, location)] }
        set { self[rsCheckedRange(start, end, location)] = newValue }
    }

    private func rsCheckedRange(_ start: Int?, _ end: Int?, _ location: String) -> Range<Int> {
        let lo = start ?? 0, hi = end ?? count
        if let end, end > count { rsPanic("range end index \(end) out of range for slice of length \(count)", location) }
        if lo > count { rsPanic("range start index \(lo) out of range for slice of length \(count)", location) }
        if lo > hi { rsPanic("slice index starts at \(lo) but ends at \(hi)", location) }
        return lo..<hi
    }
}

/// `map[&k]`: Rust's panic when the key is missing.
func rsEntry<V>(_ value: V?, _ location: String) -> V {
    guard let value else { rsPanic("no entry found for key", location) }
    return value
}

/// `assert!(cond)`, `assert!(cond, "…", …)`.
func rsAssert(_ condition: Bool, _ message: @autoclosure () -> String, _ location: String) {
    if !condition { rsPanic(message(), location) }
}

/// `assert_eq!(a, b)`, `assert_ne!(a, b, "…")`: both values in `{:?}` when it fails.
func rsAssertCmp<A, B>(_ a: A, _ b: B, _ op: String, _ location: String, _ message: String? = nil, _ holds: (A, B) -> Bool) {
    if holds(a, b) { return }
    let head = "assertion `left \(op) right` failed" + (message.map { ": " + $0 } ?? "")
    rsPanic(head + "\n  left: " + rsDebug(a) + "\n right: " + rsDebug(b), location)
}

extension Optional {
    /// `opt.unwrap()`.
    func rsUnwrap(_ location: String) -> Wrapped {
        guard let value = self else { rsPanic("called `Option::unwrap()` on a `None` value", location) }
        return value
    }

    /// `opt.expect("…")`.
    func rsExpect(_ message: String, _ location: String) -> Wrapped {
        guard let value = self else { rsPanic(message, location) }
        return value
    }
}

extension Result {
    /// `result.unwrap()`, `result.expect("…")`.
    func rsUnwrap(_ location: String, _ message: String? = nil) -> Success {
        switch self {
        case .success(let value):
            return value
        case .failure(let error):
            let head = message ?? "called `Result::unwrap()` on an `Err` value"
            rsPanic(head + ": " + rsDebug(error), location)
        }
    }

    /// `result.unwrap_err()`, `result.expect_err("…")`.
    func rsUnwrapErr(_ location: String, _ message: String? = nil) -> Failure {
        switch self {
        case .success(let value):
            let head = message ?? "called `Result::unwrap_err()` on an `Ok` value"
            rsPanic(head + ": " + rsDebug(value), location)
        case .failure(let error):
            return error
        }
    }
}

/// `f()?.unwrap()` of a throwing call: its value, or the panic of `Result::unwrap`.
func rsTryUnwrap<T>(_ location: String, _ message: String? = nil, _ body: () throws -> T) -> T {
    do {
        return try body()
    } catch {
        let head = message ?? "called `Result::unwrap()` on an `Err` value"
        rsPanic(head + ": " + rsDebug(error), location)
    }
}
"#;

const RC: &str = r#"
/// The allocation an `Rc` shares: the value, dropped with the last strong reference, and
/// the counts.
final class RsRcBox<T>: @unchecked Sendable {
    var value: T?
    var strong = 0
    var weak = 0
    init(_ value: T) { self.value = value }
}

/// `Rc<T>` in a crate counting references: each clone is a handle, released with it (at
/// the end of its scope, or by `drop`), as Rust drops it.
final class RsRc<T>: @unchecked Sendable {
    let box: RsRcBox<T>
    private var released = false

    init(_ value: T) {
        box = RsRcBox(value)
        box.strong = 1
    }

    init(box: RsRcBox<T>) {
        self.box = box
        box.strong += 1
    }

    deinit { release() }

    /// `drop(rc)`.
    func release() {
        if released { return }
        released = true
        box.strong -= 1
        if box.strong == 0 { box.value = nil }
    }

    var value: T {
        get { box.value! }
        set { box.value = newValue }
    }

    func deref() -> T { box.value! }
    func clone() -> RsRc<T> { RsRc(box: box) }
    var strongCount: Int { box.strong }
    var weakCount: Int { box.weak }
}

/// `Weak<T>`.
final class RsWeak<T>: @unchecked Sendable, RsDebugCustom {
    let box: RsRcBox<T>?
    private var released = false

    init() { box = nil }

    init(_ rc: RsRc<T>) {
        box = rc.box
        rc.box.weak += 1
    }

    init(box: RsRcBox<T>) {
        self.box = box
        box.weak += 1
    }

    deinit { release() }

    func release() {
        if released { return }
        released = true
        box?.weak -= 1
    }

    func upgrade() -> RsRc<T>? {
        guard let box, box.strong > 0 else { return nil }
        return RsRc(box: box)
    }

    func clone() -> RsWeak<T> { box.map { RsWeak(box: $0) } ?? RsWeak() }
    var strongCount: Int { box?.strong ?? 0 }
    var weakCount: Int {
        guard let box, box.strong > 0 else { return 0 }
        return box.weak
    }
    var rsDebugText: String { "(Weak)" }
}

extension RsRc: RsDebugText {
    func rsDebugText(pretty: Bool, style: RsDebugStyle) -> String {
        rsDebug(value, pretty: pretty, style: style)
    }
}

extension RsRc: CustomStringConvertible {
    var description: String { String(describing: value) }
}

extension RsRc: Equatable where T: Equatable {
    static func == (lhs: RsRc, rhs: RsRc) -> Bool { lhs.value == rhs.value }
}

extension RsRc: Comparable where T: Comparable {
    static func < (lhs: RsRc, rhs: RsRc) -> Bool { lhs.value < rhs.value }
}

extension RsRc: Hashable where T: Hashable {
    func hash(into hasher: inout Hasher) { hasher.combine(value) }
}
"#;

const LINK: &str = r#"
/// `let mut cur = &mut self.head; … cur = &mut node.next;`: a cursor on a link between
/// nodes (classes); the root link is the variable it started from, given at each use.
struct RsLink<Node: AnyObject> {
    var parent: Node? = nil
    var path: ReferenceWritableKeyPath<Node, Node?>? = nil

    /// The node the cursor points at.
    func get(_ root: Node?) -> Node? {
        if let parent, let path { return parent[keyPath: path] }
        return root
    }

    /// `*cur = value`.
    func set(_ root: inout Node?, _ value: Node?) {
        if let parent, let path { parent[keyPath: path] = value } else { root = value }
    }

    /// `cur.take()`.
    func take(_ root: inout Node?) -> Node? {
        let value = get(root)
        set(&root, nil)
        return value
    }

    /// `cur = &mut node.next`.
    mutating func move(_ node: Node, _ path: ReferenceWritableKeyPath<Node, Node?>) {
        parent = node
        self.path = path
    }
}
"#;

const ARRAY_CMP: &str = r#"
/// `Vec`s compare element by element, the shorter first, as in Rust.
extension Array: @retroactive Comparable where Element: Comparable {
    public static func < (lhs: [Element], rhs: [Element]) -> Bool {
        for (x, y) in zip(lhs, rhs) where x != y {
            return x < y
        }
        return lhs.count < rhs.count
    }
}
"#;

const PARSE_ERROR: &str = r#"
/// The error of `s.parse()?` when the text is not a valid number.
struct RsParseError: Error, CustomStringConvertible {
    var description: String { "invalid digit found in string" }
}
"#;

const PARSE_ANY: &str = r#"
/// `T::Err` of a generic `s.parse::<T>()`: the error of the type `T` stands for
/// (`ParseIntError { kind: InvalidDigit }`, `ParseFloatError { kind: Invalid }`…).
struct RsAnyParseError: Error, CustomStringConvertible, RsDebugText {
    let inner: any Error
    var description: String { String(describing: inner) }
    func rsDebugText(pretty: Bool, style: RsDebugStyle) -> String {
        rsDebug(inner, pretty: pretty, style: style)
    }
}

/// `s.parse::<T>()` in generic code: the parse of the type `T` stands for, as Rust reads
/// it (`Character("ab")` would trap, `Double("0x10")` read a hexadecimal number), with
/// that type's error.
func rsParseAny<T: LosslessStringConvertible>(_ s: String, _: T.Type, bits: Int = 0) -> Result<T, RsAnyParseError> {
    func done<U, E: Error>(_ r: Result<U, E>) -> Result<T, RsAnyParseError> {
        switch r {
        case .success(let v): return .success(v as! T)
        case .failure(let e): return .failure(RsAnyParseError(inner: e))
        }
    }
    if T.self == Character.self { return done(rsParseChar(s)) }
    if T.self == Bool.self { return done(rsParseBool(s)) }
    if T.self == Double.self { return done(rsParseFloat(s, Double.self)) }
    if T.self == Float.self { return done(rsParseFloat(s, Float.self)) }
    if T.self == String.self { return .success(s as! T) }
    if let intType = T.self as? any FixedWidthInteger.Type {
        return done(rsParseAnyInt(s, intType, bits))
    }
    if let v = T(s) { return .success(v) }
    return .failure(RsAnyParseError(inner: RsParseError()))
}

private func rsParseAnyInt<I: FixedWidthInteger>(_ s: String, _ t: I.Type, _ bits: Int) -> Result<Any, RsParseIntError> {
    rsParseInt(s, t, bits: bits == 0 ? I.bitWidth : bits).map { $0 as Any }
}
"#;

const COLOR_HEX: &str = r#"
extension Color {
    /// `Color::hex(0xRRGGBB)`.
    init(hex: UInt32) {
        self.init(
            red: Double((hex >> 16) & 0xFF) / 255,
            green: Double((hex >> 8) & 0xFF) / 255,
            blue: Double(hex & 0xFF) / 255
        )
    }
}
"#;

const INT_BOUNDS: &str = r#"
/// The limits of a Rust integer of `bits` bits held in a wider Swift type (`i32` in `Int`).
func rsIntBounds<T: FixedWidthInteger>(_: T.Type, _ bits: Int) -> (T, T) {
    if bits >= T.bitWidth { return (T.min, T.max) }
    if T.isSigned {
        let max = (T(1) << (bits - 1)) - 1
        return (0 - max - 1, max)
    }
    return (0, (T(1) << bits) - 1)
}
"#;

const PARSE: &str = r#"
/// `std::num::ParseIntError`.
struct RsParseIntError: Error, Equatable, CustomStringConvertible, RsDebugText {
    /// `std::num::IntErrorKind`.
    enum Kind: String, RsDebugCustom {
        case empty = "Empty", invalidDigit = "InvalidDigit"
        case posOverflow = "PosOverflow", negOverflow = "NegOverflow", zero = "Zero"
        var rsDebugText: String { rawValue }
    }
    let kind: Kind
    var description: String {
        switch kind {
        case .empty: "cannot parse integer from empty string"
        case .invalidDigit: "invalid digit found in string"
        case .posOverflow: "number too large to fit in target type"
        case .negOverflow: "number too small to fit in target type"
        case .zero: "number would be zero for non-zero type"
        }
    }
    func rsDebugText(pretty: Bool, style: RsDebugStyle) -> String {
        rsDebugRecord("ParseIntError", [("kind", kind)], pretty: pretty, style: style)
    }
}

/// `s.parse::<i32>()`, `i32::from_str_radix(s, 16)`: Rust's rules, within the limits of the
/// Rust type (`bits`) when Swift's is wider.
func rsParseInt<T: FixedWidthInteger>(_ s: String, _: T.Type, radix: Int = 10, bits: Int = T.bitWidth) -> Result<T, RsParseIntError> {
    var bytes = Array(s.utf8)[...]
    if bytes.isEmpty { return .failure(RsParseIntError(kind: .empty)) }
    var negative = false
    if bytes.first == UInt8(ascii: "+") || bytes.first == UInt8(ascii: "-") {
        if bytes.count == 1 { return .failure(RsParseIntError(kind: .invalidDigit)) }
        if bytes.first == UInt8(ascii: "-") {
            if !T.isSigned { return .failure(RsParseIntError(kind: .invalidDigit)) }
            negative = true
        }
        bytes = bytes.dropFirst()
    }
    let (min, max) = rsIntBounds(T.self, bits)
    let overflow = RsParseIntError(kind: negative ? .negOverflow : .posOverflow)
    var result: T = 0
    for b in bytes {
        let digit: Int
        switch b {
        case UInt8(ascii: "0")...UInt8(ascii: "9"): digit = Int(b - UInt8(ascii: "0"))
        case UInt8(ascii: "a")...UInt8(ascii: "z"): digit = Int(b - UInt8(ascii: "a")) + 10
        case UInt8(ascii: "A")...UInt8(ascii: "Z"): digit = Int(b - UInt8(ascii: "A")) + 10
        default: return .failure(RsParseIntError(kind: .invalidDigit))
        }
        if digit >= radix { return .failure(RsParseIntError(kind: .invalidDigit)) }
        let (shifted, o1) = result.multipliedReportingOverflow(by: T(radix))
        if o1 { return .failure(overflow) }
        let (next, o2) = negative
            ? shifted.subtractingReportingOverflow(T(digit))
            : shifted.addingReportingOverflow(T(digit))
        if o2 || next < min || next > max { return .failure(overflow) }
        result = next
    }
    return .success(result)
}

/// `std::num::ParseFloatError`.
struct RsParseFloatError: Error, Equatable, CustomStringConvertible, RsDebugText {
    let empty: Bool
    var description: String {
        empty ? "cannot parse float from empty string" : "invalid float literal"
    }
    func rsDebugText(pretty: Bool, style: RsDebugStyle) -> String {
        rsDebugRecord("ParseFloatError", [("kind", RsDebugBare(rsDebugText: empty ? "Empty" : "Invalid"))], pretty: pretty, style: style)
    }
}

/// `s.parse::<f64>()`: Rust's grammar (digits, a point, an exponent, `inf`, `nan`; no spaces,
/// no hexadecimal).
func rsParseFloat<T: BinaryFloatingPoint & LosslessStringConvertible>(_ s: String, _: T.Type) -> Result<T, RsParseFloatError> {
    if s.isEmpty { return .failure(RsParseFloatError(empty: true)) }
    var text = Substring(s)
    var negative = false
    if text.first == "+" || text.first == "-" {
        negative = text.first == "-"
        text = text.dropFirst()
    }
    let word = text.lowercased()
    if word == "inf" || word == "infinity" { return .success(negative ? -T.infinity : T.infinity) }
    if word == "nan" { return .success(T.nan) }
    var i = text.startIndex
    func digits() -> Int {
        var n = 0
        while i < text.endIndex, text[i].isASCII, text[i].isNumber {
            n += 1
            i = text.index(after: i)
        }
        return n
    }
    let integral = digits()
    var fractional = 0
    if i < text.endIndex, text[i] == "." {
        i = text.index(after: i)
        fractional = digits()
    }
    if integral == 0 && fractional == 0 { return .failure(RsParseFloatError(empty: false)) }
    if i < text.endIndex, text[i] == "e" || text[i] == "E" {
        i = text.index(after: i)
        if i < text.endIndex, text[i] == "+" || text[i] == "-" { i = text.index(after: i) }
        if digits() == 0 { return .failure(RsParseFloatError(empty: false)) }
    }
    if i != text.endIndex { return .failure(RsParseFloatError(empty: false)) }
    guard let value = T(s) else { return .failure(RsParseFloatError(empty: false)) }
    return .success(value)
}

/// `std::str::ParseBoolError`.
struct RsParseBoolError: Error, Equatable, CustomStringConvertible, RsDebugCustom {
    var description: String { "provided string was not `true` or `false`" }
    var rsDebugText: String { "ParseBoolError" }
}

/// `s.parse::<bool>()`.
/// `s.parse()` whose target type is found later: one overload per type.
func rsParse(_ s: String, _ t: Int.Type) -> Result<Int, RsParseIntError> { rsParseInt(s, Int.self) }
func rsParse(_ s: String, _ t: Int8.Type) -> Result<Int8, RsParseIntError> { rsParseInt(s, Int8.self) }
func rsParse(_ s: String, _ t: Int16.Type) -> Result<Int16, RsParseIntError> { rsParseInt(s, Int16.self) }
func rsParse(_ s: String, _ t: Int32.Type) -> Result<Int32, RsParseIntError> { rsParseInt(s, Int32.self) }
func rsParse(_ s: String, _ t: UInt.Type) -> Result<UInt, RsParseIntError> { rsParseInt(s, UInt.self) }
func rsParse(_ s: String, _ t: UInt8.Type) -> Result<UInt8, RsParseIntError> { rsParseInt(s, UInt8.self) }
func rsParse(_ s: String, _ t: UInt16.Type) -> Result<UInt16, RsParseIntError> { rsParseInt(s, UInt16.self) }
func rsParse(_ s: String, _ t: UInt32.Type) -> Result<UInt32, RsParseIntError> { rsParseInt(s, UInt32.self) }
func rsParse(_ s: String, _ t: Int64.Type) -> Result<Int64, RsParseIntError> { rsParseInt(s, Int64.self) }
func rsParse(_ s: String, _ t: UInt64.Type) -> Result<UInt64, RsParseIntError> { rsParseInt(s, UInt64.self) }
func rsParse(_ s: String, _ t: Double.Type) -> Result<Double, RsParseFloatError> { rsParseFloat(s, Double.self) }
func rsParse(_ s: String, _ t: Float.Type) -> Result<Float, RsParseFloatError> { rsParseFloat(s, Float.self) }
func rsParse(_ s: String, _ t: Bool.Type) -> Result<Bool, RsParseBoolError> { rsParseBool(s) }
func rsParse(_ s: String, _ t: Character.Type) -> Result<Character, RsParseCharError> { rsParseChar(s) }
func rsParse(_ s: String, _ t: String.Type) -> Result<String, Never> { .success(s) }

func rsParseBool(_ s: String) -> Result<Bool, RsParseBoolError> {
    switch s {
    case "true": .success(true)
    case "false": .success(false)
    default: .failure(RsParseBoolError())
    }
}

/// `std::char::ParseCharError`.
struct RsParseCharError: Error, Equatable, CustomStringConvertible, RsDebugText {
    let empty: Bool
    var description: String {
        empty ? "cannot parse char from empty string" : "too many characters in string"
    }
    func rsDebugText(pretty: Bool, style: RsDebugStyle) -> String {
        rsDebugRecord("ParseCharError", [("kind", RsDebugBare(rsDebugText: empty ? "EmptyString" : "TooManyChars"))], pretty: pretty, style: style)
    }
}

/// `s.parse::<char>()`: exactly one Unicode scalar.
func rsParseChar(_ s: String) -> Result<Character, RsParseCharError> {
    let scalars = s.unicodeScalars
    guard let first = scalars.first else { return .failure(RsParseCharError(empty: true)) }
    if scalars.count > 1 { return .failure(RsParseCharError(empty: false)) }
    return .success(Character(first))
}

/// `std::char::CharTryFromError`.
struct RsCharTryFromError: Error, Equatable, CustomStringConvertible, RsDebugText {
    var description: String { "converted integer out of range for `char`" }
    func rsDebugText(pretty: Bool, style: RsDebugStyle) -> String {
        rsDebugRecord("CharTryFromError", [("", ())], pretty: pretty, style: style)
    }
}

/// `char::try_from(n)`.
func rsCharTryFrom(_ n: some BinaryInteger) -> Result<Character, RsCharTryFromError> {
    guard let value = UInt32(exactly: n), let scalar = UnicodeScalar(value) else {
        return .failure(RsCharTryFromError())
    }
    return .success(Character(scalar))
}
"#;

const UTF8: &str = r#"
/// `std::str::Utf8Error`.
struct RsUtf8Error: Error, Equatable, CustomStringConvertible, RsDebugText {
    let validUpTo: Int
    let errorLen: Int?
    var description: String {
        if let errorLen {
            "invalid utf-8 sequence of \(errorLen) bytes from index \(validUpTo)"
        } else {
            "incomplete utf-8 byte sequence from index \(validUpTo)"
        }
    }
    func rsDebugText(pretty: Bool, style: RsDebugStyle) -> String {
        rsDebugRecord("Utf8Error", [("valid_up_to", validUpTo), ("error_len", errorLen as Any)], pretty: pretty, style: style)
    }
}

/// `std::string::FromUtf8Error`.
struct RsFromUtf8Error: Error, Equatable, CustomStringConvertible, RsDebugText {
    let bytes: [UInt8]
    let error: RsUtf8Error
    var description: String { error.description }
    func rsDebugText(pretty: Bool, style: RsDebugStyle) -> String {
        rsDebugRecord("FromUtf8Error", [("bytes", bytes), ("error", error)], pretty: pretty, style: style)
    }
    func intoBytes() -> [UInt8] { bytes }
    func asBytes() -> [UInt8] { bytes }
    func utf8Error() -> RsUtf8Error { error }
}

/// Where `bytes` stop being UTF-8, as Rust reports it.
func rsUtf8Error(_ v: [UInt8]) -> RsUtf8Error? {
    var index = 0
    let len = v.count
    func next() -> UInt8? {
        index += 1
        return index < len ? v[index] : nil
    }
    func continuation(_ b: UInt8) -> Bool { Int8(bitPattern: b) < -64 }
    while index < len {
        let start = index
        let first = v[index]
        if first < 128 {
            index += 1
            continue
        }
        let width: Int
        switch first {
        case 0xC2...0xDF: width = 2
        case 0xE0...0xEF: width = 3
        case 0xF0...0xF4: width = 4
        default: return RsUtf8Error(validUpTo: start, errorLen: 1)
        }
        guard let second = next() else { return RsUtf8Error(validUpTo: start, errorLen: nil) }
        if width == 2 {
            if !continuation(second) { return RsUtf8Error(validUpTo: start, errorLen: 1) }
        } else {
            let valid: Bool
            switch (width, first, second) {
            case (3, 0xE0, 0xA0...0xBF), (3, 0xE1...0xEC, 0x80...0xBF), (3, 0xED, 0x80...0x9F),
                (3, 0xEE...0xEF, 0x80...0xBF), (4, 0xF0, 0x90...0xBF), (4, 0xF1...0xF3, 0x80...0xBF),
                (4, 0xF4, 0x80...0x8F):
                valid = true
            default:
                valid = false
            }
            if !valid { return RsUtf8Error(validUpTo: start, errorLen: 1) }
            guard let third = next() else { return RsUtf8Error(validUpTo: start, errorLen: nil) }
            if !continuation(third) { return RsUtf8Error(validUpTo: start, errorLen: 2) }
            if width == 4 {
                guard let fourth = next() else { return RsUtf8Error(validUpTo: start, errorLen: nil) }
                if !continuation(fourth) { return RsUtf8Error(validUpTo: start, errorLen: 3) }
            }
        }
        index = start + width
    }
    return nil
}

/// `String::from_utf8(bytes)`.
/// `std::str::from_utf8(&bytes)`.
func rsStrFromUtf8(_ bytes: [UInt8]) -> Result<String, RsUtf8Error> {
    if let error = rsUtf8Error(bytes) { return .failure(error) }
    return .success(String(decoding: bytes, as: UTF8.self))
}

func rsFromUtf8(_ bytes: [UInt8]) -> Result<String, RsFromUtf8Error> {
    if let error = rsUtf8Error(bytes) { return .failure(RsFromUtf8Error(bytes: bytes, error: error)) }
    return .success(String(decoding: bytes, as: UTF8.self))
}
"#;

const TUPLE1: &str = r#"
/// `(T,)`: Swift has no tuple of one element.
struct RsTuple1<T> {
    var _0: T
    init(_ value: T) { _0 = value }
}

extension RsTuple1: Equatable where T: Equatable {}
extension RsTuple1: Hashable where T: Hashable {}
extension RsTuple1: Comparable where T: Comparable {
    static func < (lhs: Self, rhs: Self) -> Bool { lhs._0 < rhs._0 }
}
"#;

const DURATION: &str = r#"
/// `std::time::Duration`: whole seconds and nanoseconds, printed by `{:?}` as Rust does
/// (`1.5s`, `250ms`, `5ns`).
struct RsDuration: Equatable, Hashable, Comparable {
    var secs: UInt64
    var nanos: UInt32

    static let zero = RsDuration(secs: 0, nanos: 0)
    static let max = RsDuration(secs: UInt64.max, nanos: 999_999_999)

    init() {
        secs = 0
        nanos = 0
    }

    init(secs: UInt64, nanos: UInt32) {
        self.secs = secs + UInt64(nanos / 1_000_000_000)
        self.nanos = nanos % 1_000_000_000
    }

    static func fromSecs(_ s: UInt64) -> RsDuration { RsDuration(secs: s, nanos: 0) }
    static func fromMillis(_ ms: UInt64) -> RsDuration { RsDuration(secs: ms / 1000, nanos: UInt32(ms % 1000) * 1_000_000) }
    static func fromMicros(_ us: UInt64) -> RsDuration { RsDuration(secs: us / 1_000_000, nanos: UInt32(us % 1_000_000) * 1000) }
    static func fromNanos(_ ns: UInt64) -> RsDuration { RsDuration(secs: ns / 1_000_000_000, nanos: UInt32(ns % 1_000_000_000)) }
    static func fromSecsF64(_ s: Double) -> RsDuration {
        precondition(s >= 0 && s.isFinite, "cannot convert float seconds to Duration: value is negative or not finite")
        let whole = s.rounded(.down)
        let nanos = ((s - whole) * 1e9).rounded()
        return RsDuration(secs: UInt64(whole), nanos: UInt32(nanos))
    }
    static func fromSecsF32(_ s: Float) -> RsDuration { fromSecsF64(Double(s)) }

    func asSecs() -> UInt64 { secs }
    func asMillis() -> UInt64 { secs * 1000 + UInt64(nanos / 1_000_000) }
    func asMicros() -> UInt64 { secs * 1_000_000 + UInt64(nanos / 1000) }
    func asNanos() -> UInt64 { secs * 1_000_000_000 + UInt64(nanos) }
    func asSecsF64() -> Double { Double(secs) + Double(nanos) / 1e9 }
    func asSecsF32() -> Float { Float(asSecsF64()) }
    func subsecMillis() -> UInt32 { nanos / 1_000_000 }
    func subsecMicros() -> UInt32 { nanos / 1000 }
    func subsecNanos() -> UInt32 { nanos }
    func isZero() -> Bool { secs == 0 && nanos == 0 }

    func checkedAdd(_ o: RsDuration) -> RsDuration? {
        let (s, overflow) = secs.addingReportingOverflow(o.secs)
        if overflow { return nil }
        var total = RsDuration(secs: s, nanos: 0)
        let n = nanos + o.nanos
        if n >= 1_000_000_000 {
            if total.secs == UInt64.max { return nil }
            total.secs += 1
            total.nanos = n - 1_000_000_000
        } else {
            total.nanos = n
        }
        return total
    }
    func checkedSub(_ o: RsDuration) -> RsDuration? {
        if self < o { return nil }
        var s = secs - o.secs
        var n: UInt32
        if nanos >= o.nanos {
            n = nanos - o.nanos
        } else {
            s -= 1
            n = nanos + 1_000_000_000 - o.nanos
        }
        return RsDuration(secs: s, nanos: n)
    }
    func saturatingAdd(_ o: RsDuration) -> RsDuration { checkedAdd(o) ?? .max }
    func saturatingSub(_ o: RsDuration) -> RsDuration { checkedSub(o) ?? .zero }
    func checkedMul(_ k: UInt32) -> RsDuration? {
        let totalNanos = UInt64(nanos) * UInt64(k)
        let extra = totalNanos / 1_000_000_000
        let (s1, o1) = secs.multipliedReportingOverflow(by: UInt64(k))
        if o1 { return nil }
        let (s2, o2) = s1.addingReportingOverflow(extra)
        if o2 { return nil }
        return RsDuration(secs: s2, nanos: UInt32(totalNanos % 1_000_000_000))
    }
    func mulF64(_ k: Double) -> RsDuration { RsDuration.fromSecsF64(asSecsF64() * k) }
    func divF64(_ k: Double) -> RsDuration { RsDuration.fromSecsF64(asSecsF64() / k) }

    static func + (a: RsDuration, b: RsDuration) -> RsDuration {
        guard let r = a.checkedAdd(b) else { fatalError("overflow when adding durations") }
        return r
    }
    static func - (a: RsDuration, b: RsDuration) -> RsDuration {
        guard let r = a.checkedSub(b) else { fatalError("overflow when subtracting durations") }
        return r
    }
    static func * (a: RsDuration, k: Int) -> RsDuration {
        guard let r = a.checkedMul(UInt32(k)) else { fatalError("overflow when multiplying duration by scalar") }
        return r
    }
    static func * (k: Int, a: RsDuration) -> RsDuration { a * k }
    static func / (a: RsDuration, k: Int) -> RsDuration {
        precondition(k != 0, "divide by zero error when dividing duration by scalar")
        let s = a.secs / UInt64(k)
        let carry = a.secs - s * UInt64(k)
        let extra = carry * 1_000_000_000 / UInt64(k)
        let n = UInt64(a.nanos) / UInt64(k) + extra
        return RsDuration(secs: s, nanos: UInt32(n))
    }
    static func += (a: inout RsDuration, b: RsDuration) { a = a + b }
    static func -= (a: inout RsDuration, b: RsDuration) { a = a - b }
    static func *= (a: inout RsDuration, k: Int) { a = a * k }
    static func /= (a: inout RsDuration, k: Int) { a = a / k }
    static func < (a: RsDuration, b: RsDuration) -> Bool {
        a.secs != b.secs ? a.secs < b.secs : a.nanos < b.nanos
    }
}

extension RsDuration: RsDebugCustom {
    /// `1.5s`, `1.000001s`, `250ms`, `1.5µs`, `5ns`: Rust's `{:?}`.
    var rsDebugText: String {
        func decimal(_ whole: UInt64, _ frac: UInt32, _ digits: Int, _ unit: String) -> String {
            var text = "\(whole)"
            if frac > 0 {
                var f = String(frac)
                f = String(repeating: "0", count: digits - f.count) + f
                while f.hasSuffix("0") { f.removeLast() }
                text += "." + f
            }
            return text + unit
        }
        if secs > 0 { return decimal(secs, nanos, 9, "s") }
        if nanos >= 1_000_000 { return decimal(UInt64(nanos / 1_000_000), nanos % 1_000_000, 6, "ms") }
        if nanos >= 1_000 { return decimal(UInt64(nanos / 1_000), nanos % 1_000, 3, "µs") }
        return "\(nanos)ns"
    }
}
"#;

const HEAP: &str = r#"
/// `BinaryHeap<T>`: a max-heap laid out exactly like Rust's, so that `{:?}`, `iter()` and
/// `into_vec()` list the elements in the same order.
struct RsBinaryHeap<T: Comparable>: Sequence, CustomReflectable {
    var items: [T] = []

    init() {}
    /// `BinaryHeap::from(vec)`, `collect()`.
    init(_ values: [T]) {
        items = values
        rebuild()
    }

    var count: Int { items.count }
    var isEmpty: Bool { items.isEmpty }
    /// `heap.peek()`.
    var peek: T? { items.first }

    mutating func push(_ item: T) {
        items.append(item)
        siftUp(0, items.count - 1)
    }

    mutating func pop() -> T? {
        guard var item = items.popLast() else { return nil }
        if !items.isEmpty {
            swap(&item, &items[0])
            siftDownToBottom(0)
        }
        return item
    }

    mutating func clear() { items.removeAll() }

    mutating func drain() -> [T] {
        defer { items.removeAll() }
        return items
    }

    /// `heap.into_sorted_vec()`: ascending.
    func intoSortedVec() -> [T] {
        var heap = self
        var end = heap.items.count
        while end > 1 {
            end -= 1
            heap.items.swapAt(0, end)
            heap.siftDownRange(0, end)
        }
        return heap.items
    }

    mutating func extend(_ values: some Sequence<T>) {
        let start = items.count
        items.append(contentsOf: values)
        rebuildTail(start)
    }

    /// `heap.append(&mut other)`: `other` is emptied.
    mutating func append(_ other: inout RsBinaryHeap<T>) {
        if items.count < other.items.count { swap(&items, &other.items) }
        let start = items.count
        items.append(contentsOf: other.items)
        other.items.removeAll()
        rebuildTail(start)
    }

    mutating func retain(_ keep: (T) -> Bool) {
        var firstRemoved = items.count
        var i = 0
        items.removeAll { x in
            let remove = !keep(x)
            if remove && i < firstRemoved { firstRemoved = i }
            i += 1
            return remove
        }
        rebuildTail(firstRemoved)
    }

    func makeIterator() -> IndexingIterator<[T]> { items.makeIterator() }
    var customMirror: Mirror { Mirror(self, unlabeledChildren: items, displayStyle: .collection) }

    private mutating func siftUp(_ start: Int, _ position: Int) {
        var pos = position
        let element = items[pos]
        while pos > start {
            let parent = (pos - 1) / 2
            if element <= items[parent] { break }
            items[pos] = items[parent]
            pos = parent
        }
        items[pos] = element
    }

    private mutating func siftDownRange(_ position: Int, _ end: Int) {
        var pos = position
        let element = items[pos]
        var child = 2 * pos + 1
        while child <= end - 2 {
            if items[child] <= items[child + 1] { child += 1 }
            if element >= items[child] {
                items[pos] = element
                return
            }
            items[pos] = items[child]
            pos = child
            child = 2 * pos + 1
        }
        if child == end - 1 && element < items[child] {
            items[pos] = items[child]
            pos = child
        }
        items[pos] = element
    }

    private mutating func siftDownToBottom(_ position: Int) {
        let end = items.count
        var pos = position
        let element = items[pos]
        var child = 2 * pos + 1
        while child <= end - 2 {
            if items[child] <= items[child + 1] { child += 1 }
            items[pos] = items[child]
            pos = child
            child = 2 * pos + 1
        }
        if child == end - 1 {
            items[pos] = items[child]
            pos = child
        }
        items[pos] = element
        siftUp(position, pos)
    }

    private mutating func rebuild() {
        var n = items.count / 2
        while n > 0 {
            n -= 1
            siftDownRange(n, items.count)
        }
    }

    private mutating func rebuildTail(_ start: Int) {
        if start >= items.count { return }
        let tail = items.count - start
        let better: Bool
        if start < tail {
            better = true
        } else if items.count <= 2048 {
            better = 2 * items.count < tail * (Int.bitWidth - start.leadingZeroBitCount - 1)
        } else {
            better = 2 * items.count < tail * 11
        }
        if better {
            rebuild()
        } else {
            for i in start..<items.count { siftUp(0, i) }
        }
    }
}
"#;
const TUPLE_KEYS: &str = r#"
/// A hashable `(A, B)`: Swift cannot hash a tuple.
struct RsKey2<A: Hashable, B: Hashable>: Hashable {
    let a: A; let b: B
    init(_ a: A, _ b: B) { self.a = a; self.b = b }
    var tuple: (A, B) { (a, b) }
}

/// `HashSet<(A, B)>`, `BTreeSet<(A, B)>`.
struct RsTupleSet2<A: Hashable, B: Hashable>: Sequence, Hashable, ExpressibleByArrayLiteral, CustomReflectable {
    var items: Set<RsKey2<A, B>> = []

    init() {}
    init(_ values: some Sequence<(A, B)>) { for v in values { items.insert(RsKey2(v.0, v.1)) } }
    init(arrayLiteral elements: (A, B)...) { self.init(elements) }

    var count: Int { items.count }
    var isEmpty: Bool { items.isEmpty }
    @discardableResult
    mutating func insert(_ v: (A, B)) -> (inserted: Bool, memberAfterInsert: (A, B)) {
        (items.insert(RsKey2(v.0, v.1)).inserted, v)
    }
    func contains(_ v: (A, B)) -> Bool { items.contains(RsKey2(v.0, v.1)) }
    @discardableResult
    mutating func remove(_ v: (A, B)) -> (A, B)? { items.remove(RsKey2(v.0, v.1)).map { $0.tuple } }
    mutating func removeAll() { items.removeAll() }
    mutating func retain(_ keep: ((A, B)) -> Bool) { items = items.filter { keep($0.tuple) } }
    func union(_ other: RsTupleSet2) -> RsTupleSet2 { var r = self; r.items.formUnion(other.items); return r }
    func intersection(_ other: RsTupleSet2) -> RsTupleSet2 { var r = self; r.items.formIntersection(other.items); return r }
    func subtracting(_ other: RsTupleSet2) -> RsTupleSet2 { var r = self; r.items.subtract(other.items); return r }
    func symmetricDifference(_ other: RsTupleSet2) -> RsTupleSet2 { var r = self; r.items.formSymmetricDifference(other.items); return r }
    func isSubset(of other: RsTupleSet2) -> Bool { items.isSubset(of: other.items) }
    func isSuperset(of other: RsTupleSet2) -> Bool { items.isSuperset(of: other.items) }
    func isDisjoint(with other: RsTupleSet2) -> Bool { items.isDisjoint(with: other.items) }
    mutating func formUnion(_ other: some Sequence<(A, B)>) { for v in other { items.insert(RsKey2(v.0, v.1)) } }
    func makeIterator() -> AnyIterator<(A, B)> {
        var it = items.makeIterator()
        return AnyIterator { it.next().map { $0.tuple } }
    }
    var customMirror: Mirror { Mirror(self, unlabeledChildren: items.map { $0.tuple }, displayStyle: .set) }
}

extension RsTupleSet2 where A: Comparable, B: Comparable {
    /// A `BTreeSet` is iterated in order.
    func sorted() -> [(A, B)] { items.map { $0.tuple }.sorted { $0 < $1 } }
    func min() -> (A, B)? { sorted().first }
    func max() -> (A, B)? { sorted().last }
}

/// `HashMap<(A, B), V>`, `BTreeMap<(A, B), V>`.
struct RsTupleMap2<A: Hashable, B: Hashable, V>: Sequence, ExpressibleByDictionaryLiteral, CustomReflectable {
    var items: [RsKey2<A, B>: V] = [:]

    init() {}
    init(dictionaryLiteral elements: ((A, B), V)...) { for (k, v) in elements { items[RsKey2(k.0, k.1)] = v } }
    init(_ pairs: some Sequence<((A, B), V)>, uniquingKeysWith combine: (V, V) -> V) {
        for (k, v) in pairs {
            let key = RsKey2(k.0, k.1)
            if let old = items[key] { items[key] = combine(old, v) } else { items[key] = v }
        }
    }
    init(uniqueKeysWithValues pairs: some Sequence<((A, B), V)>) { self.init(pairs, uniquingKeysWith: { _, new in new }) }

    var count: Int { items.count }
    var isEmpty: Bool { items.isEmpty }
    subscript(key: (A, B)) -> V? {
        get { items[RsKey2(key.0, key.1)] }
        set { items[RsKey2(key.0, key.1)] = newValue }
    }
    subscript(key: (A, B), default value: @autoclosure () -> V) -> V {
        get { items[RsKey2(key.0, key.1), default: value()] }
        set { items[RsKey2(key.0, key.1)] = newValue }
    }
    @discardableResult
    mutating func updateValue(_ v: V, forKey key: (A, B)) -> V? { items.updateValue(v, forKey: RsKey2(key.0, key.1)) }
    @discardableResult
    mutating func removeValue(forKey key: (A, B)) -> V? { items.removeValue(forKey: RsKey2(key.0, key.1)) }
    mutating func removeAll() { items.removeAll() }
    mutating func merge(_ other: some Sequence<((A, B), V)>, uniquingKeysWith combine: (V, V) -> V) {
        for (k, v) in other {
            let key = RsKey2(k.0, k.1)
            if let old = items[key] { items[key] = combine(old, v) } else { items[key] = v }
        }
    }
    mutating func retain(_ keep: ((A, B), V) -> Bool) { items = items.filter { keep($0.key.tuple, $0.value) } }
    var keys: [(A, B)] { items.keys.map { $0.tuple } }
    var values: [V] { Array(items.values) }
    func mapValues<T>(_ f: (V) -> T) -> RsTupleMap2<A, B, T> {
        var out = RsTupleMap2<A, B, T>()
        out.items = items.mapValues(f)
        return out
    }
    func filter(_ keep: ((key: (A, B), value: V)) -> Bool) -> RsTupleMap2 {
        var out = RsTupleMap2()
        out.items = items.filter { keep((key: $0.key.tuple, value: $0.value)) }
        return out
    }
    func sorted(by areInIncreasingOrder: ((key: (A, B), value: V), (key: (A, B), value: V)) -> Bool) -> [(key: (A, B), value: V)] {
        map { $0 }.sorted(by: areInIncreasingOrder)
    }
    func makeIterator() -> AnyIterator<(key: (A, B), value: V)> {
        var it = items.makeIterator()
        return AnyIterator { it.next().map { (key: $0.key.tuple, value: $0.value) } }
    }
    var customMirror: Mirror {
        Mirror(self, unlabeledChildren: items.map { ($0.key.tuple, $0.value) }, displayStyle: .dictionary)
    }
}

extension RsTupleMap2: Equatable where V: Equatable {}

/// A hashable `(A, B, C)`: Swift cannot hash a tuple.
struct RsKey3<A: Hashable, B: Hashable, C: Hashable>: Hashable {
    let a: A; let b: B; let c: C
    init(_ a: A, _ b: B, _ c: C) { self.a = a; self.b = b; self.c = c }
    var tuple: (A, B, C) { (a, b, c) }
}

/// `HashSet<(A, B, C)>`, `BTreeSet<(A, B, C)>`.
struct RsTupleSet3<A: Hashable, B: Hashable, C: Hashable>: Sequence, Hashable, ExpressibleByArrayLiteral, CustomReflectable {
    var items: Set<RsKey3<A, B, C>> = []

    init() {}
    init(_ values: some Sequence<(A, B, C)>) { for v in values { items.insert(RsKey3(v.0, v.1, v.2)) } }
    init(arrayLiteral elements: (A, B, C)...) { self.init(elements) }

    var count: Int { items.count }
    var isEmpty: Bool { items.isEmpty }
    @discardableResult
    mutating func insert(_ v: (A, B, C)) -> (inserted: Bool, memberAfterInsert: (A, B, C)) {
        (items.insert(RsKey3(v.0, v.1, v.2)).inserted, v)
    }
    func contains(_ v: (A, B, C)) -> Bool { items.contains(RsKey3(v.0, v.1, v.2)) }
    @discardableResult
    mutating func remove(_ v: (A, B, C)) -> (A, B, C)? { items.remove(RsKey3(v.0, v.1, v.2)).map { $0.tuple } }
    mutating func removeAll() { items.removeAll() }
    mutating func retain(_ keep: ((A, B, C)) -> Bool) { items = items.filter { keep($0.tuple) } }
    func union(_ other: RsTupleSet3) -> RsTupleSet3 { var r = self; r.items.formUnion(other.items); return r }
    func intersection(_ other: RsTupleSet3) -> RsTupleSet3 { var r = self; r.items.formIntersection(other.items); return r }
    func subtracting(_ other: RsTupleSet3) -> RsTupleSet3 { var r = self; r.items.subtract(other.items); return r }
    func symmetricDifference(_ other: RsTupleSet3) -> RsTupleSet3 { var r = self; r.items.formSymmetricDifference(other.items); return r }
    func isSubset(of other: RsTupleSet3) -> Bool { items.isSubset(of: other.items) }
    func isSuperset(of other: RsTupleSet3) -> Bool { items.isSuperset(of: other.items) }
    func isDisjoint(with other: RsTupleSet3) -> Bool { items.isDisjoint(with: other.items) }
    mutating func formUnion(_ other: some Sequence<(A, B, C)>) { for v in other { items.insert(RsKey3(v.0, v.1, v.2)) } }
    func makeIterator() -> AnyIterator<(A, B, C)> {
        var it = items.makeIterator()
        return AnyIterator { it.next().map { $0.tuple } }
    }
    var customMirror: Mirror { Mirror(self, unlabeledChildren: items.map { $0.tuple }, displayStyle: .set) }
}

extension RsTupleSet3 where A: Comparable, B: Comparable, C: Comparable {
    /// A `BTreeSet` is iterated in order.
    func sorted() -> [(A, B, C)] { items.map { $0.tuple }.sorted { $0 < $1 } }
    func min() -> (A, B, C)? { sorted().first }
    func max() -> (A, B, C)? { sorted().last }
}

/// `HashMap<(A, B, C), V>`, `BTreeMap<(A, B, C), V>`.
struct RsTupleMap3<A: Hashable, B: Hashable, C: Hashable, V>: Sequence, ExpressibleByDictionaryLiteral, CustomReflectable {
    var items: [RsKey3<A, B, C>: V] = [:]

    init() {}
    init(dictionaryLiteral elements: ((A, B, C), V)...) { for (k, v) in elements { items[RsKey3(k.0, k.1, k.2)] = v } }
    init(_ pairs: some Sequence<((A, B, C), V)>, uniquingKeysWith combine: (V, V) -> V) {
        for (k, v) in pairs {
            let key = RsKey3(k.0, k.1, k.2)
            if let old = items[key] { items[key] = combine(old, v) } else { items[key] = v }
        }
    }
    init(uniqueKeysWithValues pairs: some Sequence<((A, B, C), V)>) { self.init(pairs, uniquingKeysWith: { _, new in new }) }

    var count: Int { items.count }
    var isEmpty: Bool { items.isEmpty }
    subscript(key: (A, B, C)) -> V? {
        get { items[RsKey3(key.0, key.1, key.2)] }
        set { items[RsKey3(key.0, key.1, key.2)] = newValue }
    }
    subscript(key: (A, B, C), default value: @autoclosure () -> V) -> V {
        get { items[RsKey3(key.0, key.1, key.2), default: value()] }
        set { items[RsKey3(key.0, key.1, key.2)] = newValue }
    }
    @discardableResult
    mutating func updateValue(_ v: V, forKey key: (A, B, C)) -> V? { items.updateValue(v, forKey: RsKey3(key.0, key.1, key.2)) }
    @discardableResult
    mutating func removeValue(forKey key: (A, B, C)) -> V? { items.removeValue(forKey: RsKey3(key.0, key.1, key.2)) }
    mutating func removeAll() { items.removeAll() }
    mutating func merge(_ other: some Sequence<((A, B, C), V)>, uniquingKeysWith combine: (V, V) -> V) {
        for (k, v) in other {
            let key = RsKey3(k.0, k.1, k.2)
            if let old = items[key] { items[key] = combine(old, v) } else { items[key] = v }
        }
    }
    mutating func retain(_ keep: ((A, B, C), V) -> Bool) { items = items.filter { keep($0.key.tuple, $0.value) } }
    var keys: [(A, B, C)] { items.keys.map { $0.tuple } }
    var values: [V] { Array(items.values) }
    func mapValues<T>(_ f: (V) -> T) -> RsTupleMap3<A, B, C, T> {
        var out = RsTupleMap3<A, B, C, T>()
        out.items = items.mapValues(f)
        return out
    }
    func filter(_ keep: ((key: (A, B, C), value: V)) -> Bool) -> RsTupleMap3 {
        var out = RsTupleMap3()
        out.items = items.filter { keep((key: $0.key.tuple, value: $0.value)) }
        return out
    }
    func sorted(by areInIncreasingOrder: ((key: (A, B, C), value: V), (key: (A, B, C), value: V)) -> Bool) -> [(key: (A, B, C), value: V)] {
        map { $0 }.sorted(by: areInIncreasingOrder)
    }
    func makeIterator() -> AnyIterator<(key: (A, B, C), value: V)> {
        var it = items.makeIterator()
        return AnyIterator { it.next().map { (key: $0.key.tuple, value: $0.value) } }
    }
    var customMirror: Mirror {
        Mirror(self, unlabeledChildren: items.map { ($0.key.tuple, $0.value) }, displayStyle: .dictionary)
    }
}

extension RsTupleMap3: Equatable where V: Equatable {}
"#;
const ORD_TUPLE: &str = r#"
/// A comparable `(A, B)`, for a `BinaryHeap` or a `Reverse`: Swift compares tuples but cannot
/// make them `Comparable`.
struct RsOrd2<A: Comparable, B: Comparable>: Comparable {
    let a: A; let b: B
    init(_ value: (A, B)) { self.a = value.0; self.b = value.1 }
    var tuple: (A, B) { (a, b) }
    static func == (lhs: RsOrd2, rhs: RsOrd2) -> Bool { lhs.tuple == rhs.tuple }
    static func < (lhs: RsOrd2, rhs: RsOrd2) -> Bool { (lhs.a < rhs.a) || (lhs.a == rhs.a && lhs.b < rhs.b) }
}
extension RsOrd2: Hashable where A: Hashable, B: Hashable {}

/// A comparable `(A, B, C)`, for a `BinaryHeap` or a `Reverse`: Swift compares tuples but cannot
/// make them `Comparable`.
struct RsOrd3<A: Comparable, B: Comparable, C: Comparable>: Comparable {
    let a: A; let b: B; let c: C
    init(_ value: (A, B, C)) { self.a = value.0; self.b = value.1; self.c = value.2 }
    var tuple: (A, B, C) { (a, b, c) }
    static func == (lhs: RsOrd3, rhs: RsOrd3) -> Bool { lhs.tuple == rhs.tuple }
    static func < (lhs: RsOrd3, rhs: RsOrd3) -> Bool { (lhs.a < rhs.a) || (lhs.a == rhs.a && lhs.b < rhs.b) || (lhs.a == rhs.a && lhs.b == rhs.b && lhs.c < rhs.c) }
}
extension RsOrd3: Hashable where A: Hashable, B: Hashable, C: Hashable {}
"#;

const CHARS: &str = r#"
extension String {
    /// `s.chars()`: the Unicode scalars, which Rust's `char`s are, as characters; Swift's own
    /// iteration goes by grapheme clusters.
    var rsChars: LazyMapSequence<String.UnicodeScalarView, Character> {
        unicodeScalars.lazy.map { Character($0) }
    }
}
"#;
