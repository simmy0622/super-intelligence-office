import SwiftUI
import UIKit

// MARK: - Avatar (color generated from handle, initial fallback)

struct Avatar: View {
    let seed: String
    var avatarSeed: String?
    var size: CGFloat = DJ.avatar

    init(seed: String, avatarSeed: String? = nil, size: CGFloat = DJ.avatar) {
        self.seed = seed
        self.avatarSeed = avatarSeed
        self.size = size
    }

    init(actor: Actor, size: CGFloat = DJ.avatar) {
        self.seed = actor.handle
        self.avatarSeed = actor.avatarSeed
        self.size = size
    }

    var body: some View {
        ZStack {
            if let image = avatarImage {
                Image(uiImage: image)
                    .resizable()
                    .scaledToFill()
                    .frame(width: size, height: size)
                    .clipShape(Circle())
            } else {
                Circle().fill(palette.background)
                Text(initials)
                    .font(.system(size: size * 0.42, weight: .semibold))
                    .foregroundStyle(palette.foreground)
            }
        }
        .frame(width: size, height: size)
        .overlay(Circle().stroke(Color.black.opacity(0.05), lineWidth: 1))
        .clipShape(Circle())
    }

    private var initials: String {
        let trimmed = seed.trimmingCharacters(in: .whitespacesAndNewlines)
        return trimmed.isEmpty ? "?" : String(trimmed.prefix(1)).uppercased()
    }

    private var palette: (background: Color, foreground: Color) {
        var hash = 0
        for ch in seed.unicodeScalars {
            hash = (hash &* 31 &+ Int(ch.value)) & 0xFFFFFF
        }
        let hue = Double(abs(hash) % 360) / 360.0
        return (Color(hue: hue, saturation: 0.45, brightness: 0.78),
                .white.opacity(0.92))
    }

    private var avatarImage: UIImage? {
        AvatarStorage.image(for: avatarSeed)
            ?? AvatarStorage.image(for: AvatarStorage.defaultSeed(for: seed))
    }
}

// MARK: - Agent verified badge (X-style blue check, repurposed)

struct AgentBadge: View {
    var size: CGFloat = 14
    var body: some View {
        Image(systemName: "checkmark.seal.fill")
            .font(.system(size: size, weight: .bold))
            .foregroundStyle(SalonColor.agentVerifiedBlue)
            .accessibilityLabel("Agent")
    }
}

// MARK: - Trigger pill (only shows when agent posts on schedule/reactive)

struct TriggerPill: View {
    let trigger: TriggerKind
    var body: some View {
        Text(trigger.label)
            .font(.system(size: 11, weight: .semibold))
            .foregroundStyle(.secondary)
            .padding(.horizontal, 8)
            .padding(.vertical, 3)
            .overlay(Capsule().stroke(Color.black.opacity(0.12), lineWidth: 1))
    }
}

// MARK: - Generic capsule pill

struct Pill: View {
    let text: String
    let systemImage: String?

    init(_ text: String, systemImage: String? = nil) {
        self.text = text
        self.systemImage = systemImage
    }

    var body: some View {
        HStack(spacing: 6) {
            if let symbol = systemImage {
                Image(systemName: symbol).font(.system(size: 11, weight: .semibold))
            }
            Text(text).font(.system(size: 12, weight: .semibold))
        }
        .foregroundStyle(.secondary)
        .padding(.horizontal, 10)
        .padding(.vertical, 6)
        .background(SalonColor.surface)
        .clipShape(Capsule())
    }
}

// MARK: - Action icon (small button on action row)

struct ActionIcon: View {
    let system: String
    var tint: Color = .secondary
    let onTap: () -> Void

    var body: some View {
        Button(action: onTap) {
            Image(systemName: system)
                .font(.system(size: 15, weight: .regular))
                .foregroundStyle(tint)
                .frame(width: 26, height: 26)
                .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
    }
}

// MARK: - Tabs

enum AppTab: Int, CaseIterable {
    case home, search, notifications, agents

    var title: String {
        switch self {
        case .home:          return "主页"
        case .search:        return "搜索"
        case .notifications: return "通知"
        case .agents:        return "Agents"
        }
    }
    var icon: String {
        switch self {
        case .home:          return "house"
        case .search:        return "magnifyingglass"
        case .notifications: return "bell"
        case .agents:        return "person.2"
        }
    }
    var iconFill: String {
        switch self {
        case .home:          return "house.fill"
        case .search:        return "magnifyingglass"  // 搜索使用 outline，选中时加粗
        case .notifications: return "bell.fill"
        case .agents:        return "person.2.fill"
        }
    }
    
    var useBoldInsteadOfFill: Bool {
        self == .search  // 搜索图标选中时用加粗而非填充
    }

    var selectedWeight: Font.Weight {
        switch self {
        case .search: return .bold
        default:      return .semibold
        }
    }
}

struct CustomTabBar: View {
    @Binding var selected: AppTab
    var notifBadge: Int = 0
    var isHidden: Bool = false

    var body: some View {
        ZStack {
            // 背景和内容一起移动
            VStack(spacing: 0) {
                HStack(spacing: 0) {
                    ForEach(AppTab.allCases, id: \.rawValue) { tab in
                        Button {
                            selected = tab
                            UIImpactFeedbackGenerator(style: .light).impactOccurred()
                        } label: {
                            ZStack(alignment: .topTrailing) {
                                Image(systemName: (selected == tab && !tab.useBoldInsteadOfFill) ? tab.iconFill : tab.icon)
                                    .font(.system(size: 22, weight: selected == tab ? .bold : .regular))
                                    .foregroundStyle(Color.primary)
                                    .frame(maxWidth: .infinity)
                                    .frame(height: DJ.tabContainerHeight)

                                if tab == .notifications && notifBadge > 0 {
                                    Text("\(min(notifBadge, 99))")
                                        .font(.system(size: 10, weight: .bold))
                                        .foregroundStyle(.white)
                                        .padding(.horizontal, 4)
                                        .padding(.vertical, 2)
                                        .background(Color.red)
                                        .clipShape(Capsule())
                                        .offset(x: -12, y: 8)
                                }
                            }
                        }
                        .buttonStyle(.plain)
                    }
                }
            }
            .frame(height: DJ.tabContainerHeight)
            .background(SalonColor.canvas)
        }
        .frame(height: DJ.tabContainerHeight, alignment: .bottom)
        .offset(y: isHidden ? DJ.tabContainerHeight + 120 : 0)  // 增加位移量确保完全隐藏 (考虑 Safe Area)
        .animation(.spring(response: 0.28, dampingFraction: 0.9), value: isHidden)
        .ignoresSafeArea(edges: .bottom)
    }
}
