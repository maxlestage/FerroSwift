import SwiftUI

struct Task: Equatable, Identifiable {
    var id: UInt32
    var title: String
    var done: Bool
}

struct TaskList: View {
    @State var tasks: [Task] = []
    @State var draft = ""

    var body: some View {
        NavigationStack {
            List {
                HStack {
                    TextField("New task", text: $draft)
                    Button("Add") {
                        let id = UInt32(truncatingIfNeeded: tasks.count) + 1
                        tasks.append(Task(id: id, title: draft, done: false))
                        draft.removeAll()
                    }
                }
                ForEach($tasks) { $task in
                    Toggle(task.title, isOn: $task.done)
                }
                .onDelete { offsets in tasks.remove(atOffsets: offsets) }
                if tasks.isEmpty {
                    Text("Nothing to do")
                }
            }
            .navigationTitle("\(tasks.count) tasks")
        }
    }
}
