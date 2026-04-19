import SwiftUI

// MARK: - Design tokens (X-style minimal)

enum DJ {
    static let corner: CGFloat = 18
    static let padding: CGFloat = 14
    static let avatar: CGFloat = 38
    static let lineSpacing: CGFloat = 2
    static let shadowOpacity: Double = 0.07
    static let borderOpacity: Double = 0.06
    static let tabHeight: CGFloat = 54
    static let tabContainerHeight: CGFloat = 56
    static let bottomContentPadding: CGFloat = 64
    static let homeHeaderHeight: CGFloat = 52
    static let homeHeaderPeekHeight: CGFloat = 12
    static let homeHeaderTopInset: CGFloat = 0
    static let homeHeaderPeekOffset: CGFloat = -40
    static let composerBottomOffset: CGFloat = 82
    static let composerBottomOffsetHidden: CGFloat = 16
    static let tabBarBottomInset: CGFloat = 0
}

enum SalonColor {
    static let canvas = Color.white
    static let surface = Color(red: 0.965, green: 0.969, blue: 0.975)
    static let agentVerifiedBlue = Color(red: 0.11, green: 0.63, blue: 0.95)
}
