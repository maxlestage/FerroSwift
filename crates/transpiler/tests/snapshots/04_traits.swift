import Foundation

protocol Describe {
    func name() -> String
    func describe() -> String
}

extension Describe {
    func describe() -> String {
        "This is \(name())"
    }
}

struct Dog {
    var age: UInt8
}

extension Dog: Describe {
    func name() -> String {
        "a dog of \(age)"
    }
}

extension Dog: CustomStringConvertible {
    var description: String {
        "Dog(\(age))"
    }
}

func loudest<T: Describe & CustomStringConvertible>(_ items: [T]) -> String? {
    items.map { i in rsDisplay(i) }.max()
}

struct Stack<T> {
    var items: [T]
}

extension Stack {
    mutating func push(_ item: T) {
        items.append(item)
    }

    func peek() -> T? {
        items.last
    }
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

