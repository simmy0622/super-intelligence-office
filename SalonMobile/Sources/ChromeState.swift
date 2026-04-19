import SwiftUI

@MainActor
final class ChromeState: ObservableObject {
    static let scrollSpaceName = "tab-bar-scroll"

    @Published var isTabBarHidden = false
    @Published var isHomeHeaderHidden = false

    func revealTabBar() {
        guard isTabBarHidden else { return }
        withAnimation(.spring(response: 0.28, dampingFraction: 0.9)) {
            isTabBarHidden = false
        }
    }

    func hideTabBar() {
        guard !isTabBarHidden else { return }
        withAnimation(.spring(response: 0.28, dampingFraction: 0.9)) {
            isTabBarHidden = true
        }
    }

    func revealHomeHeader() {
        guard isHomeHeaderHidden else { return }
        withAnimation(.spring(response: 0.28, dampingFraction: 0.92)) {
            isHomeHeaderHidden = false
        }
    }

    func hideHomeHeader() {
        guard !isHomeHeaderHidden else { return }
        withAnimation(.spring(response: 0.28, dampingFraction: 0.92)) {
            isHomeHeaderHidden = true
        }
    }
}

private struct ScrollOffsetPreferenceKey: PreferenceKey {
    static let defaultValue: CGFloat = 0

    static func reduce(value: inout CGFloat, nextValue: () -> CGFloat) {
        value = nextValue()
    }
}

struct ScrollOffsetProbe: View {
    var body: some View {
        GeometryReader { proxy in
            Color.clear
                .preference(
                    key: ScrollOffsetPreferenceKey.self,
                    value: proxy.frame(in: .named(ChromeState.scrollSpaceName)).minY
                )
        }
        .frame(height: 0)
    }
}

private struct TabBarScrollTrackingModifier: ViewModifier {
    @EnvironmentObject private var chrome: ChromeState
    @State private var lastOffset: CGFloat?
    @State private var lastDragY: CGFloat = 0

    let tracksHomeHeader: Bool

    private let dragHideStepThreshold: CGFloat = -8
    private let dragHideProjectedThreshold: CGFloat = -26
    private let dragRevealStepThreshold: CGFloat = 4

    func body(content: Content) -> some View {
        content
            .coordinateSpace(name: ChromeState.scrollSpaceName)
            .onAppear {
                chrome.revealTabBar()
                if tracksHomeHeader {
                    chrome.revealHomeHeader()
                }
            }
            .onPreferenceChange(ScrollOffsetPreferenceKey.self) { offset in
                guard let last = self.lastOffset else {
                    self.lastOffset = offset
                    return
                }

                let delta = offset - last
                
                // 1. 顶部强制显示逻辑 (防止回弹)
                if offset > 20 {
                    if chrome.isHomeHeaderHidden || chrome.isTabBarHidden {
                        chrome.revealTabBar()
                        chrome.revealHomeHeader()
                    }
                    self.lastOffset = offset
                    return
                }

                // 2. 正常滚动收起逻辑
                // 手指上滑 (delta < 0) -> Hide
                if delta < -6 {
                    if self.tracksHomeHeader { chrome.hideHomeHeader() }
                    chrome.hideTabBar()
                } 
                // 手指下滑 (delta > 0) -> Reveal
                else if delta > 12 {
                    if self.tracksHomeHeader { chrome.revealHomeHeader() }
                    chrome.revealTabBar()
                }

                self.lastOffset = offset
            }
            .simultaneousGesture(
                DragGesture(minimumDistance: 4)
                    .onChanged { value in
                        let delta = value.translation.height - self.lastDragY
                        let projectedDelta = value.predictedEndTranslation.height - value.translation.height

                        if delta < self.dragHideStepThreshold && projectedDelta < self.dragHideProjectedThreshold {
                            chrome.hideTabBar()
                            if self.tracksHomeHeader { chrome.hideHomeHeader() }
                        } else if delta > self.dragRevealStepThreshold {
                            chrome.revealTabBar()
                            if self.tracksHomeHeader { chrome.revealHomeHeader() }
                        }

                        self.lastDragY = value.translation.height
                    }
                    .onEnded { _ in
                        self.lastDragY = 0
                    }
            )
    }
}

extension View {
    func tracksTabBarOnScroll(tracksHomeHeader: Bool = false) -> some View {
        modifier(TabBarScrollTrackingModifier(tracksHomeHeader: tracksHomeHeader))
    }
}
