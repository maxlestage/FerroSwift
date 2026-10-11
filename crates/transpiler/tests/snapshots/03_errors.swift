import Foundation

enum ConfigError: Error {
    case missing(String)
    case invalid(key: String, value: String)
}

func port(_ text: String) throws(ConfigError) -> UInt16 {
    let value = text.trimmingCharacters(in: .whitespacesAndNewlines)
    if value.isEmpty {
        throw .missing("port")
    }
    guard let port = rsParseInt(value, UInt16.self).ok() else {
        throw .invalid(key: "port", value: value)
    }
    return port
}

func address(_ host: String, _ portText: String) throws(ConfigError) -> String {
    let p = try port(portText)
    return "\(host):\(p)"
}

func portOrDefault(_ text: String) -> UInt16 {
    (try? port(text)) ?? 8080
}

func firstCharUpper(_ s: String) -> Character? {
    guard let c = s.rsChars.first else { return nil }
    return Character(c.uppercased())
}

// MARK: - FerroSwift support

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

/// `{:.2}`: a number with a fixed count of decimals.
func rsFixed<F: BinaryFloatingPoint>(_ x: F, _ decimals: Int) -> String {
    if x.isNaN { return "NaN" }
    return String(format: "%.\(decimals)f", Double(x))
}

/// Support types with their own `{:?}` text.
protocol RsDebugCustom { var rsDebugText: String { get } }
/// Support types printed by `{:?}` under Rust's name (`RefCell { value: 1 }`).
protocol RsDebugNamed { var rsDebugName: String { get } }

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
/// `i128` and `u128` (`RsInt128`, `RsUInt128`): integers for `{:?}`.
protocol RsWideInteger {
    func rsHex(uppercase: Bool) -> String
}

private func rsDebugInt(_ value: Any, _ hex: Character?, prefix: Bool) -> String {
    guard let hex else { return "\(value)" }
    let upper = hex == "X"
    let digits: String
    switch value {
    case let v as RsWideInteger: digits = v.rsHex(uppercase: upper)
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
    case is Int, is Int8, is Int16, is Int32, is Int64, is UInt, is UInt8, is UInt16, is UInt32, is UInt64, is RsWideInteger:
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

/// The limits of a Rust integer of `bits` bits held in a wider Swift type (`i32` in `Int`).
func rsIntBounds<T: FixedWidthInteger>(_: T.Type, _ bits: Int) -> (T, T) {
    if bits >= T.bitWidth { return (T.min, T.max) }
    if T.isSigned {
        let max = (T(1) << (bits - 1)) - 1
        return (0 - max - 1, max)
    }
    return (0, (T(1) << bits) - 1)
}

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

extension String {
    /// `s.chars()`: the Unicode scalars, which Rust's `char`s are, as characters; Swift's own
    /// iteration goes by grapheme clusters.
    var rsChars: LazyMapSequence<String.UnicodeScalarView, Character> {
        unicodeScalars.lazy.map { Character($0) }
    }
}

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

