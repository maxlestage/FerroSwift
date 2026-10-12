[![FerroSwift: write Rust, ship SwiftUI](https://raw.githubusercontent.com/maxlestage/FerroSwift/master/site/static/og.png)](https://maxlestage.github.io/FerroSwift/)

# FerroSwift

Write SwiftUI apps in Rust. This crate is the Rust side of [FerroSwift](https://maxlestage.github.io/FerroSwift/): views, state and modifiers that type-check with `cargo check`, then become Swift with `ferroswift build`.

```rust
use ferroswift::prelude::*;

#[derive(View, Default)]
pub struct Counter {
    #[state]
    count: i64,
}

impl View for Counter {
    fn body(&mut self) -> impl View {
        vstack()
            .spacing(16.0)
            .child(text(format!("Count: {}", self.count)).font(Font::LargeTitle))
            .child(button("Increment", || self.count += 1))
            .padding()
    }
}
```

becomes

```swift
import SwiftUI

struct Counter: View {
    @State var count = 0

    var body: some View {
        VStack(spacing: 16) {
            Text("Count: \(count)")
                .font(.largeTitle)
            Button("Increment") {
                count += 1
            }
        }
        .padding()
    }
}
```

The functions do nothing at run time: they exist so that the compiler, rust-analyzer and the docs know your views. Each one documents the SwiftUI code it becomes.
