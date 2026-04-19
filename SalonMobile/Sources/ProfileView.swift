import SwiftUI
import PhotosUI

enum ProfileSegment: CaseIterable, Identifiable {
    case posts, replies, likes
    var id: String { title }
    var title: String {
        switch self {
        case .posts:   return "Crads"
        case .replies: return "回复"
        case .likes:   return "喜欢"
        }
    }
}

struct ProfileView: View {
    @EnvironmentObject private var store: SalonStore
    @EnvironmentObject private var chrome: ChromeState
    let actor: Actor
    @State private var segment: ProfileSegment = .posts
    @State private var selectedAvatarItem: PhotosPickerItem?

    private var allPosts: [Post] {
        store.posts(by: actor.id).sorted { $0.createdAt > $1.createdAt }
    }
    private var rootPosts: [Post] { allPosts.filter { $0.kind != .reply } }
    private var replyPosts: [Post] { allPosts.filter { $0.kind == .reply } }
    private var displayedActor: Actor {
        actor.id == store.currentUser.id ? store.currentUser : actor
    }

    var body: some View {
        ScrollView {
            VStack(spacing: 14) {
                header
                segmentBar

                LazyVStack(spacing: 12) {
                    let visible: [Post] = {
                        switch segment {
                        case .posts:   return rootPosts
                        case .replies: return replyPosts
                        case .likes:   return []
                        }
                    }()

                    if visible.isEmpty {
                        Text(segment == .likes ? "暂未追踪喜欢" : "暂无内容")
                            .font(.system(size: 14))
                            .foregroundStyle(.secondary)
                            .padding(.top, 30)
                    } else {
                        ForEach(visible) { post in
                            NavigationLink {
                                PostDetailView(post: post)
                                    .environmentObject(store)
                            } label: {
                                PostCard(post: post, compact: true)
                                    .environmentObject(store)
                            }
                            .buttonStyle(.plain)
                            .padding(.horizontal, 12)
                        }
                    }
                }

                Spacer(minLength: DJ.bottomContentPadding)
            }
            .padding(.top, 10)
        }
        .navigationTitle("")
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
        .background(SalonColor.canvas)
        .navigationBarTitleDisplayMode(.inline)
        .onAppear {
            chrome.revealTabBar()
        }
        .onChange(of: selectedAvatarItem) { _, item in
            guard let item else { return }
            Task {
                if let data = try? await item.loadTransferable(type: Data.self) {
                    await MainActor.run {
                        store.updateCurrentUserAvatar(imageData: data)
                        selectedAvatarItem = nil
                    }
                }
            }
        }
    }

    // MARK: Header

    private var header: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(alignment: .top) {
                Avatar(actor: displayedActor, size: 56)
                Spacer()
                if actor.id == store.currentUser.id {
                    HStack(spacing: 8) {
                        PhotosPicker(selection: $selectedAvatarItem, matching: .images) {
                            Text("编辑头像")
                                .font(.system(size: 15, weight: .semibold))
                                .padding(.horizontal, 14)
                                .padding(.vertical, 9)
                                .background(SalonColor.surface)
                                .clipShape(Capsule())
                                .overlay(Capsule().stroke(Color.black.opacity(0.06), lineWidth: 1))
                        }
                        .buttonStyle(.plain)

                        if displayedActor.avatarSeed?.trimmedOrNil() != nil {
                            Button {
                                store.removeCurrentUserAvatar()
                            } label: {
                                Image(systemName: "trash")
                                    .font(.system(size: 14, weight: .bold))
                                    .foregroundStyle(.red)
                                    .frame(width: 36, height: 36)
                                    .background(Color.red.opacity(0.08))
                                    .clipShape(Circle())
                            }
                            .buttonStyle(.plain)
                        }
                    }
                } else {
                    Button {
                        // follow placeholder
                    } label: {
                        Text("关注")
                            .font(.system(size: 15, weight: .semibold))
                            .padding(.horizontal, 14)
                            .padding(.vertical, 9)
                            .background(SalonColor.surface)
                            .clipShape(Capsule())
                            .overlay(Capsule().stroke(Color.black.opacity(0.06), lineWidth: 1))
                    }
                    .buttonStyle(.plain)
                }
            }

            HStack(spacing: 6) {
                Text(displayedActor.displayName)
                    .font(.system(size: 22, weight: .bold))
                    .tracking(-0.2)
                if displayedActor.isAgent { AgentBadge(size: 16) }
            }

            Text("@\(displayedActor.handle)")
                .font(.system(size: 15, weight: .semibold))
                .foregroundStyle(.secondary)

            if let specialty = displayedActor.specialty, !specialty.isEmpty {
                Text(specialty)
                    .font(.system(size: 13, weight: .semibold))
                    .foregroundStyle(SalonColor.agentVerifiedBlue)
            }

            if !displayedActor.bio.isEmpty {
                Text(displayedActor.bio)
                    .font(.system(size: 15))
                    .foregroundStyle(.primary)
                    .lineSpacing(2)
            }

            if let summary = displayedActor.personaSummary, !summary.isEmpty {
                Text(summary)
                    .font(.system(size: 13))
                    .foregroundStyle(.secondary)
                    .padding(10)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(SalonColor.surface)
                    .clipShape(RoundedRectangle(cornerRadius: 10))
            }

            HStack(spacing: 14) {
                StatChip(value: "\(rootPosts.count)", label: "Crads")
                StatChip(value: "\(replyPosts.count)", label: "回复")
            }
            .padding(.top, 2)
        }
        .padding(.horizontal, 16)
    }

    private var segmentBar: some View {
        HStack(spacing: 0) {
            ForEach(ProfileSegment.allCases) { seg in
                Button {
                    withAnimation(.easeInOut(duration: 0.18)) { segment = seg }
                } label: {
                    VStack(spacing: 8) {
                        Text(seg.title)
                            .font(.system(size: 15, weight: .semibold))
                            .foregroundStyle(segment == seg ? .primary : .secondary)
                        Rectangle()
                            .fill(segment == seg ? Color.primary : Color.clear)
                            .frame(height: 2)
                    }
                    .frame(maxWidth: .infinity)
                    .padding(.top, 8)
                }
                .buttonStyle(.plain)
            }
        }
        .padding(.horizontal, 12)
    }
}

private struct StatChip: View {
    let value: String
    let label: String
    var body: some View {
        HStack(spacing: 6) {
            Text(value).font(.system(size: 14, weight: .semibold))
            Text(label).font(.system(size: 14)).foregroundStyle(.secondary)
        }
    }
}
