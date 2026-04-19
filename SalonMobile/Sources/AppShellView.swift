import SwiftUI

struct AppShellView: View {
    @StateObject private var store = SalonStore()
    @StateObject private var chrome = ChromeState()
    @State private var tab: AppTab = .home

    var body: some View {
        ZStack(alignment: .bottom) {
            SalonColor.canvas
                .ignoresSafeArea()

            currentTabView
                .environmentObject(store)
                .environmentObject(chrome)
                .frame(maxWidth: .infinity, maxHeight: .infinity)

            CustomTabBar(
                selected: $tab,
                notifBadge: store.unreadNotificationCount,
                isHidden: chrome.isTabBarHidden
            )
            .padding(.bottom, DJ.tabBarBottomInset)
        }
        .ignoresSafeArea(.keyboard, edges: .bottom)
        .onChange(of: tab) { _, _ in
            chrome.revealTabBar()
            chrome.revealHomeHeader()
        }
    }

    @ViewBuilder
    private var currentTabView: some View {
        switch tab {
        case .home:
            tabContainer { HomeFeedView() }
        case .search:
            tabContainer { SearchView() }
        case .notifications:
            tabContainer { NotificationsView() }
        case .agents:
            tabContainer { AgentsTabView() }
        }
    }

    private func tabContainer<Content: View>(@ViewBuilder content: () -> Content) -> some View {
        NavigationStack {
            content()
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
                .background(SalonColor.canvas)
                .environmentObject(store)
                .environmentObject(chrome)
        }
        .background(SalonColor.canvas)
    }
}
