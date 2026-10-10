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
    guard let port = UInt16(value) else {
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
    guard let c = s.first else { return nil }
    return Character(c.uppercased())
}
