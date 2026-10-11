import Foundation

func stats(_ values: [Double]) -> (Double, Double) {
    let total = values.reduce(0.0, +)
    let mean = total / Double(values.count)
    let max = values.reduce(-Double.greatestFiniteMagnitude) { x, y in Double.maximum(x, y) }
    return (mean, max)
}

func wordCounts(_ text: String) -> [(String, Int)] {
    var counts: [String: Int] = [:]
    for word in text.split(whereSeparator: \.isWhitespace).map(String.init) {
        // Case does not matter.
        counts[word.lowercased(), default: 0] += 1
    }
    var pairs: [(String, Int)] = Array(counts)
    pairs.sortBy { a, b in rsCmp(b.1, a.1).then(rsCmp(a.0, b.0)) }
    return pairs
}

func evensSquared(_ limit: UInt32) -> [UInt32] {
    (0..<limit).filter { n in n % 2 == 0 }.map { n in n * n }
}

func report(_ names: [String]) -> String {
    names.enumerated().map { pair2 in
        let (i, name) = pair2
        return "\(rsPad("\(i + 1)", 2, ">", " ")). \(name)"
    }.joined(separator: "\n")
}

// MARK: - FerroSwift support

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

extension Array where Element: Equatable {
    mutating func dedup() {
        var out: [Element] = []
        for x in self where out.last != x { out.append(x) }
        self = out
    }
}


