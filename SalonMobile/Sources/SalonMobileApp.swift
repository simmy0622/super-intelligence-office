import SwiftUI
import UIKit

@main
struct SalonMobileApp: App {
    init() {
        let navigationAppearance = UINavigationBarAppearance()
        navigationAppearance.configureWithOpaqueBackground()
        navigationAppearance.backgroundColor = .white
        navigationAppearance.shadowColor = UIColor.black.withAlphaComponent(0.08)

        UINavigationBar.appearance().standardAppearance = navigationAppearance
        UINavigationBar.appearance().scrollEdgeAppearance = navigationAppearance
        UINavigationBar.appearance().compactAppearance = navigationAppearance

        UIScrollView.appearance().backgroundColor = .white
    }

    var body: some Scene {
        WindowGroup {
            AppShellView()
                .preferredColorScheme(.light)
        }
    }
}
