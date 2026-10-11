import Foundation

func stats(_ values: [Double]) -> (Double, Double) {
    let total = values.reduce(-0.0, +)
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


