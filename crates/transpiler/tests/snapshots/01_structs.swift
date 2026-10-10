import Foundation

/// A point on a grid.
struct Point: Equatable {
    var x: Int
    var y: Int
}

extension Point {
    /// The origin.
    static let origin = Point(x: 0, y: 0)

    init(_ x: Int, _ y: Int) {
        self = Point(x: x, y: y)
    }

    func manhattan(_ other: Point) -> Int {
        abs(x - other.x) + abs(y - other.y)
    }

    mutating func shift(_ dx: Int, _ dy: Int) {
        x += dx
        y += dy
    }

    func withX(_ x: Int) -> Point {
        var copy = self
        copy.x = x
        return copy
    }
}
