//! Swift support code: small helpers giving Swift the behavior of Rust where the standard
//! libraries differ (number formatting, `{:?}`, `Ordering`, string slicing by bytes…). Only
//! the helpers a translation uses are written.

use std::collections::BTreeSet;

/// (name, other helpers it needs, Swift code).
const HELPERS: &[(&str, &[&str], &str)] = &[
    ("display", &[], DISPLAY),
    ("fixed", &[], FIXED),
    ("debug", &["display"], DEBUG),
    ("pad", &[], PAD),
    ("Ordering", &[], ORDERING),
    ("string", &[], STRING),
    ("char", &[], CHAR),
    ("vec", &["Ordering"], VEC),
    ("reverse", &[], REVERSE),
    ("try_from", &["debug"], TRY_FROM),
    ("exp", &[], EXP),
    ("map", &[], MAP),
    ("iter", &[], ITER),
    ("option", &[], OPTION),
    ("result", &[], RESULT),
    ("int", &[], INT),
    ("mem", &[], MEM),
    ("async_result", &[], ASYNC_RESULT),
    ("eprint", &[], EPRINT),
    ("StringError", &[], STRING_ERROR),
    ("ParseError", &[], PARSE_ERROR),
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
"#;

const FIXED: &str = r#"
/// `{:.2}`: a number with a fixed count of decimals.
func rsFixed<F: BinaryFloatingPoint>(_ x: F, _ decimals: Int) -> String {
    String(format: "%.\(decimals)f", Double(x))
}
"#;

const DEBUG: &str = r#"
/// A value as Rust's `{:?}` writes it: `Some(1)`, `Point { x: 1, y: 2 }`, `"text"`.
/// `{:?}` of a `BTreeSet`, given its elements in order.
func rsDebugSet<T>(_ items: [T], pretty: Bool = false) -> String {
    let inner = rsDebug(items, pretty: pretty)
    return "{" + inner.dropFirst().dropLast() + "}"
}

/// `{:?}` of `s.parse::<T>()`: `Ok(v)` or `Err(…)`.
func rsDebugParsed<T>(_ value: T?, _ error: String) -> String {
    value.map { "Ok(" + rsDebug($0) + ")" } ?? "Err(" + error + ")"
}

/// Support types with their own `{:?}` text.
protocol RsDebugCustom { var rsDebugText: String { get } }

func rsDebug<T>(_ value: T, pretty: Bool = false) -> String {
    var out = ""
    rsDebugWrite(value as Any, pretty, 0, &out)
    return out
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

/// Values of a comparable type, in order; anything else as given.
private func rsSorted(_ values: [Any]) -> [Any] {
    func attempt<T: Comparable>(_: T.Type) -> [Any]? {
        let typed = values.compactMap { $0 as? T }
        return typed.count == values.count ? typed.sorted() : nil
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
    return values
}

private func rsSameKey(_ a: Any, _ b: Any) -> Bool {
    func same<T: Equatable>(_: T.Type) -> Bool? {
        guard let x = a as? T, let y = b as? T else { return nil }
        return x == y
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

private func rsDebugWrite(_ value: Any, _ pretty: Bool, _ level: Int, _ out: inout String) {
    if let custom = value as? RsDebugCustom {
        out += custom.rsDebugText
        return
    }
    let mirror = Mirror(reflecting: value)
    if mirror.displayStyle == .optional {
        if let some = mirror.children.first {
            out += "Some("
            if pretty {
                out += "\n" + String(repeating: "    ", count: level + 1)
                rsDebugWrite(some.value, pretty, level + 1, &out)
                out += ",\n" + String(repeating: "    ", count: level) + ")"
            } else {
                rsDebugWrite(some.value, pretty, level, &out)
                out += ")"
            }
        } else {
            out += "None"
        }
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
                rsDebugWrite(item, pretty, level + 1, &out)
                out += ",\n"
            }
            out += end + close
        } else {
            if open.hasSuffix("{") { out += " " }
            for (i, (label, item)) in items.enumerated() {
                if i > 0 { out += ", " }
                if let label { out += label + ": " }
                rsDebugWrite(item, pretty, level, &out)
            }
            if close == "}" { out += " " }
            out += close
        }
    }
    switch value {
    case let s as String:
        out += "\""
        for c in s.unicodeScalars {
            switch c {
            case "\"": out += "\\\""
            case "\\": out += "\\\\"
            case "\n": out += "\\n"
            case "\r": out += "\\r"
            case "\t": out += "\\t"
            case "\0": out += "\\0"
            default: out.unicodeScalars.append(c)
            }
        }
        out += "\""
        return
    case let c as Character:
        out += c == "'" ? "'\\''" : c == "\n" ? "'\\n'" : "'\(c)'"
        return
    case let d as Double:
        out += rsDebugFloat(d.description, d == d.rounded() && d.isFinite)
        return
    case let f as Float:
        out += rsDebugFloat(f.description, f == f.rounded() && f.isFinite)
        return
    case is Int, is Int8, is Int16, is Int32, is Int64, is UInt, is UInt8, is UInt16, is UInt32, is UInt64, is Bool:
        out += "\(value)"
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
                rsDebugWrite(pair[0], pretty, level + 1, &out)
                out += ": "
                rsDebugWrite(pair[1], pretty, level + 1, &out)
                out += ",\n"
            }
            out += end
        } else {
            for (i, pair) in pairs.enumerated() {
                if i > 0 { out += ", " }
                rsDebugWrite(pair[0], pretty, level, &out)
                out += ": "
                rsDebugWrite(pair[1], pretty, level, &out)
            }
        }
        out += "}"
    case .tuple:
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
        if fields.displayStyle == .tuple && fields.children.contains(where: { $0.label.map { !$0.hasPrefix(".") } ?? false }) {
            list(" {", "}", fields.children.map { ($0.label.map(rsSnake), $0.value) })
        } else if fields.displayStyle == .tuple {
            list("(", ")", fields.children.map { (nil, $0.value) })
        } else {
            list("(", ")", [(nil, payload.value)])
        }
    case .struct, .class:
        if let custom = value as? CustomDebugStringConvertible {
            out += custom.debugDescription
            return
        }
        let name = rsTypeName(value)
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

private func rsDebugFloat(_ s: String, _ integral: Bool) -> String {
    if let e = s.firstIndex(of: "e") {
        var mantissa = String(s[..<e])
        if mantissa.hasSuffix(".0") { mantissa.removeLast(2) }
        var exponent = String(s[s.index(after: e)...])
        if exponent.hasPrefix("+") { exponent.removeFirst() }
        let negative = exponent.hasPrefix("-")
        if negative { exponent.removeFirst() }
        while exponent.count > 1 && exponent.hasPrefix("0") { exponent.removeFirst() }
        return mantissa + "e" + (negative ? "-" : "") + exponent
    }
    if s == "nan" { return "NaN" }
    if s == "inf" || s == "-inf" { return s }
    return s
}
"#;

const PAD: &str = r#"
/// `{:>8}`, `{:<8}`, `{:^8}`: a text padded to a width.
func rsPad(_ s: String, _ width: Int, _ align: Character, _ fill: Character) -> String {
    let missing = width - s.count
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
"#;

const STRING: &str = r#"
extension String {
    /// `&s[a..b]`: a slice by UTF-8 byte offsets.
    func rsSlice(_ start: Int, _ end: Int) -> String {
        let bytes = Array(utf8)
        return String(decoding: bytes[start..<end], as: UTF8.self)
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
    func lines() -> [String] {
        var parts = components(separatedBy: "\n").map { $0.hasSuffix("\r") ? String($0.dropLast()) : $0 }
        if parts.last == "" { parts.removeLast() }
        return parts
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

    /// `s.char_indices()`: characters with their byte offsets.
    func charIndices() -> [(Int, Character)] {
        var offset = 0
        return map { c in
            defer { offset += String(c).utf8.count }
            return (offset, c)
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

const EXP: &str = r#"
/// `{:e}`: Rust's scientific notation (`1.2345e3`, `1e-7`), with a precision when given.
func rsExp<F: BinaryFloatingPoint>(_ x: F, _ precision: Int? = nil, upper: Bool = false) -> String {
    let d = Double(x)
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
        mantissa = (d < 0 ? "-" : "") + String(digits.prefix(1)) + (digits.count > 1 ? "." + digits.dropFirst() : "")
    }
    return mantissa + (upper ? "E" : "e") + String(exponent)
}

func rsExp<I: BinaryInteger>(_ x: I, _ precision: Int? = nil, upper: Bool = false) -> String {
    rsExp(Double(x), precision, upper: upper)
}
"#;

const VEC: &str = r#"
extension Array {
    /// `v.get(i)`.
    func get(_ i: Int) -> Element? { indices.contains(i) ? self[i] : nil }

    mutating func retain(_ keep: (Element) throws -> Bool) rethrows {
        try removeAll { try !keep($0) }
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

    mutating func fill(_ value: Element) {
        self = Array(repeating: value, count: count)
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

    mutating func sortBy(_ compare: (Element, Element) throws -> Ordering) rethrows {
        try sort { try compare($0, $1) == .less }
    }

    mutating func sortByKey<K: Comparable>(_ key: (Element) throws -> K) rethrows {
        try sort { try key($0) < key($1) }
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

    func stepBy(_ step: Int) -> [Element] {
        var out: [Element] = []
        var i = 0
        for x in self {
            if i % step == 0 { out.append(x) }
            i += 1
        }
        return out
    }

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

    func partitioned(_ p: (Element) throws -> Bool) rethrows -> ([Element], [Element]) {
        var yes: [Element] = [], no: [Element] = []
        for x in self { if try p(x) { yes.append(x) } else { no.append(x) } }
        return (yes, no)
    }
}

extension Array where Element: Equatable {
    mutating func dedup() {
        var out: [Element] = []
        for x in self where out.last != x { out.append(x) }
        self = out
    }
}

extension Array where Element: Comparable {
    /// `v.binary_search(&x)`: `.success(index)` or `.failure(insertion point)`.
    func binarySearch(_ x: Element) -> Result<Int, RsNotFound> {
        var lo = 0, hi = count
        while lo < hi {
            let mid = (lo + hi) / 2
            if self[mid] == x { return .success(mid) }
            if self[mid] < x { lo = mid + 1 } else { hi = mid }
        }
        return .failure(RsNotFound(index: lo))
    }
}

struct RsNotFound: Error { let index: Int }
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
    private var items: [Element]
    private var index = 0

    init<S: Sequence>(_ items: S) where S.Element == Element {
        self.items = Array(items)
    }

    mutating func next() -> Element? {
        guard index < items.count else { return nil }
        defer { index += 1 }
        return items[index]
    }

    func peek() -> Element? {
        index < items.count ? items[index] : nil
    }

    mutating func nextIf(_ test: (Element) -> Bool) -> Element? {
        guard let x = peek(), test(x) else { return nil }
        index += 1
        return x
    }

    mutating func nth(_ n: Int) -> Element? {
        index += n
        return next()
    }

    /// The items not consumed yet.
    func rest() -> [Element] {
        Array(items[Swift.min(index, items.count)...])
    }
}
"#;

const MAP: &str = r#"
extension Dictionary {
    mutating func retain(_ keep: ((key: Key, value: Value)) throws -> Bool) rethrows {
        self = try filter(keep)
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
"#;

const TRY_FROM: &str = r#"
/// `u8::try_from(x)`: `TryFromIntError` when the value does not fit.
struct RsTryFromIntError: Error, RsDebugCustom, CustomStringConvertible {
    var rsDebugText: String { "TryFromIntError(())" }
    var description: String { "out of range integral type conversion attempted" }
}

func rsTryFrom<T: FixedWidthInteger>(_: T.Type, _ x: some BinaryInteger) -> Result<T, RsTryFromIntError> {
    T(exactly: x).map { .success($0) } ?? .failure(RsTryFromIntError())
}
"#;

const INT: &str = r#"
extension BinaryInteger {
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

const EPRINT: &str = r#"
/// `eprintln!`: a line on the standard error.
func rsEprint(_ text: String, terminator: String = "\n") {
    FileHandle.standardError.write(Data((text + terminator).utf8))
}
"#;

const STRING_ERROR: &str = r#"
/// Rust functions returning `Result<T, String>` throw their message.
extension String: @retroactive Error {}
"#;

const PARSE_ERROR: &str = r#"
/// The error of `s.parse()?` when the text is not a valid number.
struct RsParseError: Error, CustomStringConvertible {
    var description: String { "invalid digit found in string" }
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
