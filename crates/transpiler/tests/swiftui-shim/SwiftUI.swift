// A stand-in for Apple's SwiftUI, for type-checking translated views where SwiftUI does not
// exist (Linux). It declares the subset of the real API that FerroSwift writes, with the same
// signatures, so that code accepted here is accepted by the real framework. Nothing runs.

@_exported import Foundation
@_exported import Observation

// MARK: - Views and builders

public protocol View {
    associatedtype Body: View
    @ViewBuilder @MainActor var body: Body { get }
}

extension Never: View {
    public typealias Body = Never
    public var body: Never { fatalError() }
}

/// Views made by the framework itself.
public protocol _PrimitiveView: View where Body == Never {}

extension _PrimitiveView {
    public var body: Never { fatalError() }
}

@resultBuilder
public struct ViewBuilder {
    public static func buildBlock() -> EmptyView { EmptyView() }
    public static func buildBlock<Content: View>(_ content: Content) -> Content { content }
    public static func buildBlock<each Content: View>(_ content: repeat each Content) -> TupleView<(repeat each Content)> {
        TupleView((repeat each content))
    }
    public static func buildExpression<Content: View>(_ content: Content) -> Content { content }
    public static func buildIf<Content: View>(_ content: Content?) -> Content? { content }
    public static func buildEither<T: View, F: View>(first: T) -> _ConditionalContent<T, F> { .init() }
    public static func buildEither<T: View, F: View>(second: F) -> _ConditionalContent<T, F> { .init() }
    public static func buildLimitedAvailability<Content: View>(_ content: Content) -> AnyView { AnyView(content) }
}

extension Optional: View where Wrapped: View {
    public typealias Body = Never
    public var body: Never { fatalError() }
}

public struct TupleView<T>: _PrimitiveView {
    public init(_ value: T) {}
}

public struct _ConditionalContent<TrueContent, FalseContent>: _PrimitiveView {
    init() {}
}

public struct EmptyView: _PrimitiveView {
    public init() {}
}

public struct AnyView: _PrimitiveView {
    public init<V: View>(_ view: V) {}
}

public struct ModifiedContent<Content, Modifier>: _PrimitiveView {
    init() {}
}

public protocol DynamicProperty {}

// MARK: - State

@propertyWrapper
public struct State<Value>: DynamicProperty {
    public init(wrappedValue value: Value) {}
    public init(initialValue value: Value) {}
    public var wrappedValue: Value {
        get { fatalError() }
        nonmutating set {}
    }
    public var projectedValue: Binding<Value> { fatalError() }
}

extension State where Value: ExpressibleByNilLiteral {
    public init() {}
}

@propertyWrapper
@dynamicMemberLookup
public struct Binding<Value>: DynamicProperty {
    public init(get: @escaping () -> Value, set: @escaping (Value) -> Void) {}
    public init(projectedValue: Binding<Value>) {}
    public static func constant(_ value: Value) -> Binding<Value> { fatalError() }
    public var wrappedValue: Value {
        get { fatalError() }
        nonmutating set {}
    }
    public var projectedValue: Binding<Value> { self }
    public subscript<Subject>(dynamicMember keyPath: WritableKeyPath<Value, Subject>) -> Binding<Subject> { fatalError() }
}

extension Binding: Identifiable where Value: Identifiable {
    public var id: Value.ID { fatalError() }
}

@propertyWrapper
public struct Bindable<Value: AnyObject & Observable>: DynamicProperty {
    public init(wrappedValue: Value) {}
    public var wrappedValue: Value { fatalError() }
    public var projectedValue: Bindable<Value> { self }
}

@propertyWrapper
public struct Environment<Value>: DynamicProperty {
    public init(_ keyPath: KeyPath<EnvironmentValues, Value>) {}
    public var wrappedValue: Value { fatalError() }
}

public struct EnvironmentValues {
    public var dismiss: DismissAction { fatalError() }
    public var colorScheme: ColorScheme { fatalError() }
}

public struct DismissAction {
    public func callAsFunction() {}
}

// MARK: - Text

public protocol _FormatSpecifiable {}
extension Int: _FormatSpecifiable {}
extension Int8: _FormatSpecifiable {}
extension Int16: _FormatSpecifiable {}
extension Int32: _FormatSpecifiable {}
extension Int64: _FormatSpecifiable {}
extension UInt: _FormatSpecifiable {}
extension UInt8: _FormatSpecifiable {}
extension UInt16: _FormatSpecifiable {}
extension UInt32: _FormatSpecifiable {}
extension UInt64: _FormatSpecifiable {}
extension Float: _FormatSpecifiable {}
extension Double: _FormatSpecifiable {}

public struct LocalizedStringKey: ExpressibleByStringInterpolation, Equatable {
    public init(_ value: String) {}
    public init(stringLiteral value: String) {}
    public init(stringInterpolation: StringInterpolation) {}

    public struct StringInterpolation: StringInterpolationProtocol {
        public init(literalCapacity: Int, interpolationCount: Int) {}
        public mutating func appendLiteral(_ literal: String) {}
        public mutating func appendInterpolation(_ string: String) {}
        public mutating func appendInterpolation<T: _FormatSpecifiable>(_ value: T) {}
        public mutating func appendInterpolation<T: _FormatSpecifiable>(_ value: T, specifier: String) {}
        public mutating func appendInterpolation(_ text: Text) {}
        public mutating func appendInterpolation(_ image: Image) {}
    }
}

public struct Text: _PrimitiveView, Equatable {
    public init(_ key: LocalizedStringKey, tableName: String? = nil, bundle: Bundle? = nil, comment: StaticString? = nil) {}
    @_disfavoredOverload public init<S: StringProtocol>(_ content: S) {}
    public init(verbatim content: String) {}
    public func bold() -> Text { self }
    public func italic() -> Text { self }
    public func font(_ font: Font?) -> Text { self }
    public func fontWeight(_ weight: Font.Weight?) -> Text { self }
    public func foregroundStyle<S: ShapeStyle>(_ style: S) -> Text { self }
    public func monospaced(_ isActive: Bool = true) -> Text { self }
    public func underline(_ isActive: Bool = true, color: Color? = nil) -> Text { self }
    public func strikethrough(_ isActive: Bool = true, color: Color? = nil) -> Text { self }
    public static func + (lhs: Text, rhs: Text) -> Text { lhs }
}

// MARK: - Styles and values

public protocol ShapeStyle {}

public struct Color: _PrimitiveView, ShapeStyle, Hashable {
    public init(red: Double, green: Double, blue: Double, opacity: Double = 1) {}
    public init(_ name: String) {}
    public func opacity(_ opacity: Double) -> Color { self }
    public static let red = Color("red"), orange = Color("orange"), yellow = Color("yellow"), green = Color("green")
    public static let mint = Color("mint"), teal = Color("teal"), cyan = Color("cyan"), blue = Color("blue")
    public static let indigo = Color("indigo"), purple = Color("purple"), pink = Color("pink"), brown = Color("brown")
    public static let white = Color("white"), gray = Color("gray"), black = Color("black"), clear = Color("clear")
    public static let primary = Color("primary"), secondary = Color("secondary"), accentColor = Color("accent")
}

extension ShapeStyle where Self == Color {
    public static var red: Color { .red }
    public static var orange: Color { .orange }
    public static var yellow: Color { .yellow }
    public static var green: Color { .green }
    public static var mint: Color { .mint }
    public static var teal: Color { .teal }
    public static var cyan: Color { .cyan }
    public static var blue: Color { .blue }
    public static var indigo: Color { .indigo }
    public static var purple: Color { .purple }
    public static var pink: Color { .pink }
    public static var brown: Color { .brown }
    public static var white: Color { .white }
    public static var gray: Color { .gray }
    public static var black: Color { .black }
    public static var clear: Color { .clear }
    public static var accentColor: Color { .accentColor }
}

public struct HierarchicalShapeStyle: ShapeStyle {
    public static let primary = HierarchicalShapeStyle()
    public static let secondary = HierarchicalShapeStyle()
}

extension ShapeStyle where Self == HierarchicalShapeStyle {
    public static var primary: HierarchicalShapeStyle { .primary }
    public static var secondary: HierarchicalShapeStyle { .secondary }
}

public struct Font: Hashable {
    public static let largeTitle = Font(), title = Font(), title2 = Font(), title3 = Font()
    public static let headline = Font(), subheadline = Font(), body = Font(), callout = Font()
    public static let footnote = Font(), caption = Font(), caption2 = Font()
    public static func system(size: CGFloat, weight: Font.Weight? = nil, design: Font.Design? = nil) -> Font { Font() }
    public func weight(_ weight: Font.Weight) -> Font { self }
    public func bold() -> Font { self }

    public struct Weight: Hashable {
        public static let ultraLight = Weight(), thin = Weight(), light = Weight(), regular = Weight()
        public static let medium = Weight(), semibold = Weight(), bold = Weight(), heavy = Weight(), black = Weight()
    }

    public enum Design: Hashable {
        case `default`, serif, rounded, monospaced
    }
}

public struct HorizontalAlignment: Equatable {
    public static let leading = HorizontalAlignment(), center = HorizontalAlignment(), trailing = HorizontalAlignment()
}

public struct VerticalAlignment: Equatable {
    public static let top = VerticalAlignment(), center = VerticalAlignment(), bottom = VerticalAlignment()
    public static let firstTextBaseline = VerticalAlignment(), lastTextBaseline = VerticalAlignment()
}

public struct Alignment: Equatable {
    public static let center = Alignment(), leading = Alignment(), trailing = Alignment(), top = Alignment()
    public static let bottom = Alignment(), topLeading = Alignment(), topTrailing = Alignment()
    public static let bottomLeading = Alignment(), bottomTrailing = Alignment()
}

public enum Edge: Int8, CaseIterable {
    case top, leading, bottom, trailing

    public struct Set: OptionSet {
        public let rawValue: Int8
        public init(rawValue: Int8) { self.rawValue = rawValue }
        public static let top = Set(rawValue: 1), leading = Set(rawValue: 2), bottom = Set(rawValue: 4)
        public static let trailing = Set(rawValue: 8), all = Set(rawValue: 15)
        public static let horizontal = Set(rawValue: 10), vertical = Set(rawValue: 5)
    }
}

public enum Axis: Int8 {
    case horizontal, vertical

    public struct Set: OptionSet {
        public let rawValue: Int8
        public init(rawValue: Int8) { self.rawValue = rawValue }
        public static let horizontal = Set(rawValue: 1), vertical = Set(rawValue: 2)
    }
}

public enum TextAlignment: Hashable {
    case leading, center, trailing
}

public enum ContentMode: Hashable {
    case fit, fill
}

public enum ColorScheme: Hashable {
    case light, dark
}

public struct Angle {
    public static func degrees(_ value: Double) -> Angle { Angle() }
    public static func radians(_ value: Double) -> Angle { Angle() }
}

public struct Animation: Equatable {
    public static let `default` = Animation(), linear = Animation(), easeIn = Animation(), easeOut = Animation()
    public static let easeInOut = Animation(), spring = Animation(), bouncy = Animation(), smooth = Animation()
    public static let snappy = Animation()
}

public struct AnyTransition {
    public static let opacity = AnyTransition(), slide = AnyTransition(), scale = AnyTransition()
}

@discardableResult
public func withAnimation<Result>(_ animation: Animation? = .default, _ body: () throws -> Result) rethrows -> Result {
    try body()
}

public struct ButtonRole: Equatable {
    public static let cancel = ButtonRole(), destructive = ButtonRole()
}

public protocol PrimitiveButtonStyle {}
public struct DefaultButtonStyle: PrimitiveButtonStyle {}
public struct BorderedButtonStyle: PrimitiveButtonStyle {}
public struct BorderedProminentButtonStyle: PrimitiveButtonStyle {}
public struct BorderlessButtonStyle: PrimitiveButtonStyle {}
public struct PlainButtonStyle: PrimitiveButtonStyle {}
extension PrimitiveButtonStyle where Self == DefaultButtonStyle { public static var automatic: DefaultButtonStyle { .init() } }
extension PrimitiveButtonStyle where Self == BorderedButtonStyle { public static var bordered: BorderedButtonStyle { .init() } }
extension PrimitiveButtonStyle where Self == BorderedProminentButtonStyle { public static var borderedProminent: BorderedProminentButtonStyle { .init() } }
extension PrimitiveButtonStyle where Self == BorderlessButtonStyle { public static var borderless: BorderlessButtonStyle { .init() } }
extension PrimitiveButtonStyle where Self == PlainButtonStyle { public static var plain: PlainButtonStyle { .init() } }

public protocol TextFieldStyle {}
public struct DefaultTextFieldStyle: TextFieldStyle {}
public struct PlainTextFieldStyle: TextFieldStyle {}
public struct RoundedBorderTextFieldStyle: TextFieldStyle {}
extension TextFieldStyle where Self == DefaultTextFieldStyle { public static var automatic: DefaultTextFieldStyle { .init() } }
extension TextFieldStyle where Self == PlainTextFieldStyle { public static var plain: PlainTextFieldStyle { .init() } }
extension TextFieldStyle where Self == RoundedBorderTextFieldStyle { public static var roundedBorder: RoundedBorderTextFieldStyle { .init() } }

public protocol PickerStyle {}
public struct DefaultPickerStyle: PickerStyle {}
public struct MenuPickerStyle: PickerStyle {}
public struct SegmentedPickerStyle: PickerStyle {}
public struct InlinePickerStyle: PickerStyle {}
public struct WheelPickerStyle: PickerStyle {}
extension PickerStyle where Self == DefaultPickerStyle { public static var automatic: DefaultPickerStyle { .init() } }
extension PickerStyle where Self == MenuPickerStyle { public static var menu: MenuPickerStyle { .init() } }
extension PickerStyle where Self == SegmentedPickerStyle { public static var segmented: SegmentedPickerStyle { .init() } }
extension PickerStyle where Self == InlinePickerStyle { public static var inline: InlinePickerStyle { .init() } }
extension PickerStyle where Self == WheelPickerStyle { public static var wheel: WheelPickerStyle { .init() } }

public protocol ListStyle {}
public struct DefaultListStyle: ListStyle {}
public struct PlainListStyle: ListStyle {}
public struct InsetListStyle: ListStyle {}
public struct GroupedListStyle: ListStyle {}
public struct InsetGroupedListStyle: ListStyle {}
public struct SidebarListStyle: ListStyle {}
extension ListStyle where Self == DefaultListStyle { public static var automatic: DefaultListStyle { .init() } }
extension ListStyle where Self == PlainListStyle { public static var plain: PlainListStyle { .init() } }
extension ListStyle where Self == InsetListStyle { public static var inset: InsetListStyle { .init() } }
extension ListStyle where Self == GroupedListStyle { public static var grouped: GroupedListStyle { .init() } }
extension ListStyle where Self == InsetGroupedListStyle { public static var insetGrouped: InsetGroupedListStyle { .init() } }
extension ListStyle where Self == SidebarListStyle { public static var sidebar: SidebarListStyle { .init() } }

// MARK: - Shapes

public protocol Shape: View {}

extension Shape {
    public var body: Never { fatalError() }
    public func fill<S: ShapeStyle>(_ content: S) -> some View { EmptyView() }
    public func stroke<S: ShapeStyle>(_ content: S, lineWidth: CGFloat = 1) -> some View { EmptyView() }
}

public struct Rectangle: Shape { public typealias Body = Never; public init() {} }
public struct Circle: Shape { public typealias Body = Never; public init() {} }
public struct Capsule: Shape { public typealias Body = Never; public init() {} }
public struct Ellipse: Shape { public typealias Body = Never; public init() {} }
public struct RoundedRectangle: Shape {
    public typealias Body = Never
    public init(cornerRadius: CGFloat) {}
}

extension Shape where Self == Rectangle { public static var rect: Rectangle { .init() } }
extension Shape where Self == RoundedRectangle {
    public static func rect(cornerRadius: CGFloat) -> RoundedRectangle { .init(cornerRadius: cornerRadius) }
}
extension Shape where Self == Circle { public static var circle: Circle { .init() } }
extension Shape where Self == Capsule { public static var capsule: Capsule { .init() } }

// MARK: - Layout and containers

public struct VStack<Content: View>: _PrimitiveView {
    public init(alignment: HorizontalAlignment = .center, spacing: CGFloat? = nil, @ViewBuilder content: () -> Content) {}
}

public struct HStack<Content: View>: _PrimitiveView {
    public init(alignment: VerticalAlignment = .center, spacing: CGFloat? = nil, @ViewBuilder content: () -> Content) {}
}

public struct ZStack<Content: View>: _PrimitiveView {
    public init(alignment: Alignment = .center, @ViewBuilder content: () -> Content) {}
}

public struct LazyVStack<Content: View>: _PrimitiveView {
    public init(alignment: HorizontalAlignment = .center, spacing: CGFloat? = nil, @ViewBuilder content: () -> Content) {}
}

public struct LazyHStack<Content: View>: _PrimitiveView {
    public init(alignment: VerticalAlignment = .center, spacing: CGFloat? = nil, @ViewBuilder content: () -> Content) {}
}

public struct ScrollView<Content: View>: _PrimitiveView {
    public init(_ axes: Axis.Set = .vertical, showsIndicators: Bool = true, @ViewBuilder content: () -> Content) {}
}

public struct List<Content: View>: _PrimitiveView {
    public init(@ViewBuilder content: () -> Content) {}
}

public struct Form<Content: View>: _PrimitiveView {
    public init(@ViewBuilder content: () -> Content) {}
}

public struct Section<Content: View>: _PrimitiveView {
    public init(@ViewBuilder content: () -> Content) {}
    public init(_ titleKey: LocalizedStringKey, @ViewBuilder content: () -> Content) {}
    @_disfavoredOverload public init<S: StringProtocol>(_ title: S, @ViewBuilder content: () -> Content) {}
}

public struct Group<Content: View>: _PrimitiveView {
    public init(@ViewBuilder content: () -> Content) {}
}

public struct NavigationStack<Root: View>: _PrimitiveView {
    public init(@ViewBuilder root: () -> Root) {}
}

public struct TabView<Content: View>: _PrimitiveView {
    public init(@ViewBuilder content: () -> Content) {}
}

public struct DisclosureGroup<Content: View>: _PrimitiveView {
    public init(_ titleKey: LocalizedStringKey, @ViewBuilder content: @escaping () -> Content) {}
    @_disfavoredOverload public init<S: StringProtocol>(_ title: S, @ViewBuilder content: @escaping () -> Content) {}
}

public protocol DynamicViewContent: View {}

public struct ForEach<Data: RandomAccessCollection, ID: Hashable, Content: View>: _PrimitiveView, DynamicViewContent {
    public init(_ data: Data, id: KeyPath<Data.Element, ID>, @ViewBuilder content: @escaping (Data.Element) -> Content) {}
}

extension ForEach where ID == Data.Element.ID, Data.Element: Identifiable {
    public init(_ data: Data, @ViewBuilder content: @escaping (Data.Element) -> Content) {}
}

extension ForEach where Data == Range<Int>, ID == Int {
    public init(_ data: Range<Int>, @ViewBuilder content: @escaping (Int) -> Content) {}
}

extension ForEach {
    public init<C: MutableCollection & RandomAccessCollection>(
        _ data: Binding<C>,
        @ViewBuilder content: @escaping (Binding<C.Element>) -> Content
    ) where C.Element: Identifiable, Data == [C.Element], ID == C.Element.ID, C.Index: Hashable {}
}

extension DynamicViewContent {
    public func onDelete(perform action: ((IndexSet) -> Void)?) -> some DynamicViewContent { self }
    public func onMove(perform action: ((IndexSet, Int) -> Void)?) -> some DynamicViewContent { self }
}

extension MutableCollection where Self: RangeReplaceableCollection {
    public mutating func remove(atOffsets offsets: IndexSet) {}
}

extension MutableCollection {
    public mutating func move(fromOffsets source: IndexSet, toOffset destination: Int) {}
}

// MARK: - Controls

public struct Button<Label: View>: _PrimitiveView {
    public init(action: @escaping () -> Void, @ViewBuilder label: () -> Label) {}
}

extension Button where Label == Text {
    public init(_ titleKey: LocalizedStringKey, action: @escaping () -> Void) {}
    @_disfavoredOverload public init<S: StringProtocol>(_ title: S, action: @escaping () -> Void) {}
    public init(_ titleKey: LocalizedStringKey, role: ButtonRole?, action: @escaping () -> Void) {}
    @_disfavoredOverload public init<S: StringProtocol>(_ title: S, role: ButtonRole?, action: @escaping () -> Void) {}
}

public struct NavigationLink<Label: View, Destination: View>: _PrimitiveView {
    public init(@ViewBuilder destination: () -> Destination, @ViewBuilder label: () -> Label) {}
}

extension NavigationLink where Label == Text {
    public init(_ titleKey: LocalizedStringKey, @ViewBuilder destination: () -> Destination) {}
    @_disfavoredOverload public init<S: StringProtocol>(_ title: S, @ViewBuilder destination: () -> Destination) {}
}

public struct Spacer: _PrimitiveView {
    public init(minLength: CGFloat? = nil) {}
}

public struct Divider: _PrimitiveView {
    public init() {}
}

public struct Image: _PrimitiveView, Equatable {
    public init(_ name: String) {}
    public init(systemName: String) {}
    public func resizable() -> Image { self }

    public enum Scale: Hashable {
        case small, medium, large
    }
}

public struct Label<Title: View, Icon: View>: _PrimitiveView {}

extension Label where Title == Text, Icon == Image {
    public init(_ titleKey: LocalizedStringKey, systemImage name: String) {}
    @_disfavoredOverload public init<S: StringProtocol>(_ title: S, systemImage name: String) {}
}

public struct AsyncImage<Content: View>: _PrimitiveView {}

extension AsyncImage where Content == Image {
    public init(url: URL?, scale: CGFloat = 1) {}
}

public struct Link<Label: View>: _PrimitiveView {}

extension Link where Label == Text {
    public init(_ titleKey: LocalizedStringKey, destination: URL) {}
    @_disfavoredOverload public init<S: StringProtocol>(_ title: S, destination: URL) {}
}

public struct Toggle<Label: View>: _PrimitiveView {}

extension Toggle where Label == Text {
    public init(_ titleKey: LocalizedStringKey, isOn: Binding<Bool>) {}
    @_disfavoredOverload public init<S: StringProtocol>(_ title: S, isOn: Binding<Bool>) {}
}

public struct TextField<Label: View>: _PrimitiveView {}

extension TextField where Label == Text {
    public init(_ titleKey: LocalizedStringKey, text: Binding<String>) {}
    @_disfavoredOverload public init<S: StringProtocol>(_ title: S, text: Binding<String>) {}
}

public struct SecureField<Label: View>: _PrimitiveView {}

extension SecureField where Label == Text {
    public init(_ titleKey: LocalizedStringKey, text: Binding<String>) {}
    @_disfavoredOverload public init<S: StringProtocol>(_ title: S, text: Binding<String>) {}
}

public struct TextEditor: _PrimitiveView {
    public init(text: Binding<String>) {}
}

public struct Slider<Label: View>: _PrimitiveView {}

extension Slider where Label == EmptyView {
    public init<V: BinaryFloatingPoint>(value: Binding<V>, in bounds: ClosedRange<V> = 0 ... 1) where V.Stride: BinaryFloatingPoint {}
    public init<V: BinaryFloatingPoint>(value: Binding<V>, in bounds: ClosedRange<V>, step: V.Stride = 1) where V.Stride: BinaryFloatingPoint {}
}

public struct Stepper<Label: View>: _PrimitiveView {}

extension Stepper where Label == Text {
    public init<V: Strideable>(_ titleKey: LocalizedStringKey, value: Binding<V>, in bounds: ClosedRange<V>, step: V.Stride = 1) {}
    @_disfavoredOverload public init<S: StringProtocol, V: Strideable>(_ title: S, value: Binding<V>, in bounds: ClosedRange<V>, step: V.Stride = 1) {}
}

public struct Picker<Label: View, SelectionValue: Hashable, Content: View>: _PrimitiveView {}

extension Picker where Label == Text {
    public init(_ titleKey: LocalizedStringKey, selection: Binding<SelectionValue>, @ViewBuilder content: () -> Content) {}
    @_disfavoredOverload public init<S: StringProtocol>(_ title: S, selection: Binding<SelectionValue>, @ViewBuilder content: () -> Content) {}
}

public struct ProgressView<Label: View, CurrentValueLabel: View>: _PrimitiveView {}

extension ProgressView where Label == EmptyView, CurrentValueLabel == EmptyView {
    public init() {}
    public init<V: BinaryFloatingPoint>(value: V?, total: V = 1.0) {}
}

public struct ContentUnavailableView<Label: View, Description: View, Actions: View>: _PrimitiveView {}

extension ContentUnavailableView where Label == SwiftUI.Label<Text, Image>, Description == Text?, Actions == EmptyView {
    public init(_ title: LocalizedStringKey, systemImage name: String, description: Text? = nil) {}
    @_disfavoredOverload public init<S: StringProtocol>(_ title: S, systemImage name: String, description: Text? = nil) {}
}

// MARK: - Modifiers

extension View {
    public func padding(_ edges: Edge.Set = .all, _ length: CGFloat? = nil) -> some View { self }
    public func padding(_ length: CGFloat) -> some View { self }
    public func font(_ font: Font?) -> some View { self }
    public func fontWeight(_ weight: Font.Weight?) -> some View { self }
    public func fontDesign(_ design: Font.Design?) -> some View { self }
    public func bold(_ isActive: Bool = true) -> some View { self }
    public func italic(_ isActive: Bool = true) -> some View { self }
    public func monospaced(_ isActive: Bool = true) -> some View { self }
    public func underline(_ isActive: Bool = true, color: Color? = nil) -> some View { self }
    public func strikethrough(_ isActive: Bool = true, color: Color? = nil) -> some View { self }
    public func multilineTextAlignment(_ alignment: TextAlignment) -> some View { self }
    public func lineLimit(_ number: Int?) -> some View { self }
    public func foregroundStyle<S: ShapeStyle>(_ style: S) -> some View { self }
    public func background<S: ShapeStyle>(_ style: S, ignoresSafeAreaEdges edges: Edge.Set = .all) -> some View { self }
    public func background<V: View>(alignment: Alignment = .center, @ViewBuilder content: () -> V) -> some View { self }
    public func overlay<V: View>(alignment: Alignment = .center, @ViewBuilder content: () -> V) -> some View { self }
    public func tint(_ tint: Color?) -> some View { self }
    public func border<S: ShapeStyle>(_ content: S, width: CGFloat = 1) -> some View { self }
    public func clipShape<S: Shape>(_ shape: S, style: FillStyle = FillStyle()) -> some View { self }
    public func opacity(_ opacity: Double) -> some View { self }
    public func shadow(color: Color = .black, radius: CGFloat, x: CGFloat = 0, y: CGFloat = 0) -> some View { self }
    public func blur(radius: CGFloat, opaque: Bool = false) -> some View { self }
    public func rotationEffect(_ angle: Angle) -> some View { self }
    public func scaleEffect(_ s: CGFloat) -> some View { self }
    public func offset(x: CGFloat = 0, y: CGFloat = 0) -> some View { self }
    public func frame(width: CGFloat? = nil, height: CGFloat? = nil, alignment: Alignment = .center) -> some View { self }
    public func frame(
        minWidth: CGFloat? = nil, idealWidth: CGFloat? = nil, maxWidth: CGFloat? = nil,
        minHeight: CGFloat? = nil, idealHeight: CGFloat? = nil, maxHeight: CGFloat? = nil,
        alignment: Alignment = .center
    ) -> some View { self }
    public func ignoresSafeArea() -> some View { self }
    public func disabled(_ disabled: Bool) -> some View { self }
    public func hidden() -> some View { self }
    public func labelsHidden() -> some View { self }
    public func buttonStyle<S: PrimitiveButtonStyle>(_ style: S) -> some View { self }
    public func textFieldStyle<S: TextFieldStyle>(_ style: S) -> some View { self }
    public func pickerStyle<S: PickerStyle>(_ style: S) -> some View { self }
    public func listStyle<S: ListStyle>(_ style: S) -> some View { self }
    public func tag<V: Hashable>(_ tag: V) -> some View { self }
    public func id<ID: Hashable>(_ id: ID) -> some View { self }
    public func badge(_ count: Int) -> some View { self }
    public func accessibilityLabel(_ label: Text) -> some View { self }
    public func accessibilityLabel(_ labelKey: LocalizedStringKey) -> some View { self }
    @_disfavoredOverload public func accessibilityLabel<S: StringProtocol>(_ label: S) -> some View { self }
    public func help(_ textKey: LocalizedStringKey) -> some View { self }
    @_disfavoredOverload public func help<S: StringProtocol>(_ text: S) -> some View { self }
    public func preferredColorScheme(_ colorScheme: ColorScheme?) -> some View { self }
    public func navigationTitle(_ titleKey: LocalizedStringKey) -> some View { self }
    @_disfavoredOverload public func navigationTitle<S: StringProtocol>(_ title: S) -> some View { self }
    public func toolbar<Content: View>(@ViewBuilder content: () -> Content) -> some View { self }
    public func tabItem<V: View>(@ViewBuilder _ label: () -> V) -> some View { self }
    public func sheet<Content: View>(isPresented: Binding<Bool>, onDismiss: (() -> Void)? = nil, @ViewBuilder content: @escaping () -> Content) -> some View { self }
    public func alert<A: View>(_ titleKey: LocalizedStringKey, isPresented: Binding<Bool>, @ViewBuilder actions: () -> A) -> some View { self }
    @_disfavoredOverload public func alert<S: StringProtocol, A: View>(_ title: S, isPresented: Binding<Bool>, @ViewBuilder actions: () -> A) -> some View { self }
    public func searchable(text: Binding<String>) -> some View { self }
    public func swipeActions<T: View>(edge: HorizontalEdge = .trailing, allowsFullSwipe: Bool = true, @ViewBuilder content: () -> T) -> some View { self }
    public func contextMenu<MenuItems: View>(@ViewBuilder menuItems: () -> MenuItems) -> some View { self }
    public func onAppear(perform action: (() -> Void)? = nil) -> some View { self }
    public func onDisappear(perform action: (() -> Void)? = nil) -> some View { self }
    public func onTapGesture(count: Int = 1, perform action: @escaping () -> Void) -> some View { self }
    public func onSubmit(_ action: @escaping () -> Void) -> some View { self }
    public func onChange<V: Equatable>(of value: V, initial: Bool = false, _ action: @escaping () -> Void) -> some View { self }
    public func onChange<V: Equatable>(of value: V, initial: Bool = false, _ action: @escaping (V, V) -> Void) -> some View { self }
    public func animation<V: Equatable>(_ animation: Animation?, value: V) -> some View { self }
    public func transition(_ t: AnyTransition) -> some View { self }
    public func aspectRatio(_ aspectRatio: CGFloat? = nil, contentMode: ContentMode) -> some View { self }
    public func scaledToFit() -> some View { self }
    public func scaledToFill() -> some View { self }
    public func imageScale(_ scale: Image.Scale) -> some View { self }
    public func glassEffect() -> some View { self }
}

public struct FillStyle {
    public init() {}
}

public enum HorizontalEdge {
    case leading, trailing
}

// MARK: - Apps

public protocol Scene {
    associatedtype Body: Scene
    @SceneBuilder @MainActor var body: Body { get }
}

extension Never: Scene {}

@resultBuilder
public struct SceneBuilder {
    public static func buildBlock<Content: Scene>(_ content: Content) -> Content { content }
    public static func buildExpression<Content: Scene>(_ content: Content) -> Content { content }
}

public struct WindowGroup<Content: View>: Scene {
    public init(@ViewBuilder content: () -> Content) {}
    public var body: Never { fatalError() }
}

public protocol App {
    associatedtype Body: Scene
    @SceneBuilder @MainActor var body: Body { get }
    @MainActor init()
}

extension App {
    @MainActor public static func main() {}
}
