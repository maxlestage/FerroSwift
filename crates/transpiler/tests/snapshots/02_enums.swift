import Foundation

enum Shape: Equatable {
    case circle(Double)
    case rect(width: Double, height: Double)
    case empty
}

extension Shape {
    func area() -> Double {
        switch self {
        case .circle(let r):
            return Double.pi * r * r
        case .rect(let width, let height):
            return width * height
        case .empty:
            return 0.0
        }
    }

    func name() -> String {
        switch self {
        case .circle:
            return "circle"
        case .rect(let width, let height) where width == height:
            return "square"
        case .rect:
            return "rectangle"
        case .empty:
            return "nothing"
        }
    }
}

indirect enum Tree {
    case leaf(Int)
    case node(Tree, Tree)
}

func sum(_ t: Tree) -> Int {
    switch t {
    case .leaf(let v):
        return v
    case .node(let l, let r):
        return sum(l) + sum(r)
    }
}
