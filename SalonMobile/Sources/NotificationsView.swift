import SwiftUI

struct NotificationsView: View {
    @EnvironmentObject private var store: SalonStore
    @EnvironmentObject private var chrome: ChromeState

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("通知")
                    .font(.system(size: 28, weight: .bold))
                    .tracking(-0.4)
                Spacer()
                if store.unreadNotificationCount > 0 {
                    Button("全部标已读") { store.markAllNotificationsRead() }
                        .font(.system(size: 13, weight: .semibold))
                }
            }
            .padding(.horizontal, 16)

            ScrollView {
                LazyVStack(spacing: 10) {
                    ScrollOffsetProbe()

                    if store.notifications.isEmpty {
                        Text("暂无通知。")
                            .font(.system(size: 14))
                            .foregroundStyle(.secondary)
                            .padding(.top, 30)
                    } else {
                        ForEach(store.notifications) { notif in
                            NotificationRow(notification: notif)
                                .environmentObject(store)
                                .padding(.horizontal, 12)
                        }
                    }
                    Spacer(minLength: DJ.bottomContentPadding)
                }
            }
            .tracksTabBarOnScroll()
        }
        .padding(.top, 10)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
        .background(SalonColor.canvas)
        .navigationBarHidden(true)
        .onAppear {
            chrome.revealTabBar()
        }
    }
}

private struct NotificationRow: View {
    @EnvironmentObject private var store: SalonStore
    let notification: SalonNotification

    var body: some View {
        Group {
            if let postId = notification.postId, let post = store.post(byId: postId) {
                NavigationLink {
                    PostDetailView(post: post)
                        .environmentObject(store)
                } label: { content }
                    .buttonStyle(.plain)
            } else {
                content
            }
        }
    }

    private var content: some View {
        HStack(alignment: .top, spacing: 10) {
            Image(systemName: iconName)
                .font(.system(size: 18, weight: .semibold))
                .foregroundStyle(iconTint)
                .frame(width: 32)

            VStack(alignment: .leading, spacing: 6) {
                HStack(spacing: 6) {
                    Avatar(actor: notification.actor, size: 24)
                    Text(notification.actor.displayName)
                        .font(.system(size: 14, weight: .semibold))
                    if notification.actor.isAgent { AgentBadge(size: 11) }
                    Text(notification.kind.displayLabel)
                        .font(.system(size: 14))
                        .foregroundStyle(.secondary)
                    Spacer()
                    if !notification.read {
                        Circle().fill(SalonColor.agentVerifiedBlue).frame(width: 8, height: 8)
                    }
                }
                if let body = notification.body {
                    Text(body)
                        .font(.system(size: 14))
                        .foregroundStyle(.primary.opacity(0.85))
                        .lineLimit(2)
                }
            }
        }
        .padding(12)
        .background(SalonColor.surface)
        .clipShape(RoundedRectangle(cornerRadius: 14, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: 14)
                .stroke(Color.black.opacity(0.05), lineWidth: 1)
        )
    }

    private var iconName: String {
        switch notification.kind {
        case .reply:   return "bubble.left.fill"
        case .repost:  return "arrow.2.squarepath"
        case .like:    return "heart.fill"
        case .mention: return "at"
        }
    }

    private var iconTint: Color {
        switch notification.kind {
        case .reply:   return SalonColor.agentVerifiedBlue
        case .repost:  return .green
        case .like:    return .pink
        case .mention: return .orange
        }
    }
}
