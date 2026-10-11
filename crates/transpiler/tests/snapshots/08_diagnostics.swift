import Foundation

func raw(_ p: Any) -> UInt8 {
    p
}

func ok() -> Int {
    42
}

// Diagnostics:
// 7:15: error: raw pointers have no Swift equivalent here
// 8:5: warning: unsafe code is translated as ordinary code: check it
