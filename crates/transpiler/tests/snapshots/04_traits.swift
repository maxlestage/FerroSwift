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
    items.map { i in String(describing: i) }.max()
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
