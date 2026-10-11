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

/// Support types with their own `{:?}` text.
protocol RsDebugCustom { var rsDebugText: String { get } }
/// Support types printed by `{:?}` under Rust's name (`RefCell { value: 1 }`).
protocol RsDebugNamed { var rsDebugName: String { get } }

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
struct RsParseIntError: Error, Equatable, CustomStringConvertible, RsDebugCustom {
    enum Kind: String {
        case empty = "Empty", invalidDigit = "InvalidDigit"
        case posOverflow = "PosOverflow", negOverflow = "NegOverflow"
    }
    let kind: Kind
    var description: String {
        switch kind {
        case .empty: "cannot parse integer from empty string"
        case .invalidDigit: "invalid digit found in string"
        case .posOverflow: "number too large to fit in target type"
        case .negOverflow: "number too small to fit in target type"
        }
    }
    var rsDebugText: String { "ParseIntError { kind: \(kind.rawValue) }" }
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
struct RsParseFloatError: Error, Equatable, CustomStringConvertible, RsDebugCustom {
    let empty: Bool
    var description: String {
        empty ? "cannot parse float from empty string" : "invalid float literal"
    }
    var rsDebugText: String { "ParseFloatError { kind: \(empty ? "Empty" : "Invalid") }" }
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
func rsParseBool(_ s: String) -> Result<Bool, RsParseBoolError> {
    switch s {
    case "true": .success(true)
    case "false": .success(false)
    default: .failure(RsParseBoolError())
    }
}

/// `std::char::ParseCharError`.
struct RsParseCharError: Error, Equatable, CustomStringConvertible, RsDebugCustom {
    let empty: Bool
    var description: String {
        empty ? "cannot parse char from empty string" : "too many characters in string"
    }
    var rsDebugText: String { "ParseCharError { kind: \(empty ? "EmptyString" : "TooManyChars") }" }
}

/// `s.parse::<char>()`: exactly one Unicode scalar.
func rsParseChar(_ s: String) -> Result<Character, RsParseCharError> {
    let scalars = s.unicodeScalars
    guard let first = scalars.first else { return .failure(RsParseCharError(empty: true)) }
    if scalars.count > 1 { return .failure(RsParseCharError(empty: false)) }
    return .success(Character(first))
}

/// `std::char::CharTryFromError`.
struct RsCharTryFromError: Error, Equatable, CustomStringConvertible, RsDebugCustom {
    var description: String { "converted integer out of range for `char`" }
    var rsDebugText: String { "CharTryFromError(())" }
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

