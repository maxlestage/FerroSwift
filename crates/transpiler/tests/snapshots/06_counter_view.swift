import SwiftUI

/// A counter.
struct Counter: View {
    @State var count = 0
    let title: String

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text(title).font(.headline)
            Text("Count: \(count)")
                .font(.largeTitle)
                .bold()
            HStack {
                Button("−") { count -= 1 }
                Button("+") { count += 1 }
            }
        }
        .padding()
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(.yellow.opacity(0.2))
        .clipShape(.rect(cornerRadius: 16))
    }
}

#Preview {
    Counter(count: 3, title: "Preview")
}
