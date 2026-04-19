import SwiftUI
import UIKit

struct PostCard: View {
    @EnvironmentObject private var store: SalonStore
    let post: Post
    var compact: Bool = false
    var onOpenDetail: (() -> Void)? = nil

    @State private var likeBurst = false
    @State private var showReply = false

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            VStack(alignment: .leading, spacing: 10) {
                topRow
                contentBlock
                if let referenced = referencedPost {
                    ReferencedPostPreview(post: referenced)
                }
            }
            .contentShape(Rectangle())
            .onTapGesture {
                onOpenDetail?()
            }

            actionRow
        }
        .padding(DJ.padding)
        .background(
            RoundedRectangle(cornerRadius: DJ.corner, style: .continuous)
                .fill(SalonColor.surface)
        )
        .overlay(
            RoundedRectangle(cornerRadius: DJ.corner, style: .continuous)
                .stroke(Color.black.opacity(DJ.borderOpacity), lineWidth: 1)
        )
        .clipShape(RoundedRectangle(cornerRadius: DJ.corner, style: .continuous))
        .shadow(color: .black.opacity(DJ.shadowOpacity), radius: 14, y: 10)
        .overlay(alignment: .center) {
            if likeBurst { LikeBurst().transition(.opacity) }
        }
        .sheet(isPresented: $showReply) {
            ReplySheet(parentId: post.id)
                .environmentObject(store)
                .presentationDetents([.large])
                .presentationDragIndicator(.visible)
        }
    }

    // MARK: Subviews

    private var topRow: some View {
        HStack(alignment: .top, spacing: 10) {
            Avatar(actor: post.author)

            VStack(alignment: .leading, spacing: 2) {
                HStack(spacing: 4) {
                    Text(post.author.displayName)
                        .font(.system(size: 15, weight: .semibold))
                        .lineLimit(1)
                    if post.author.isAgent { AgentBadge(size: 13) }
                    Spacer(minLength: 4)
                    if post.author.isAgent && post.trigger != .manual {
                        TriggerPill(trigger: post.trigger)
                    }
                }
                HStack(spacing: 4) {
                    Text("@\(post.author.handle)")
                        .font(.system(size: 13))
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                    Text("·")
                        .font(.system(size: 13))
                        .foregroundStyle(.secondary)
                    Text(relativeTime)
                        .font(.system(size: 13))
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                    Spacer(minLength: 0)
                }
                if let specialty = post.author.specialty, !specialty.isEmpty {
                    Text(specialty)
                        .font(.system(size: 12))
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }
                if post.kind == .reply, let parentId = post.parentId,
                   let parent = store.post(byId: parentId) {
                    Text("回复 @\(parent.author.handle)")
                        .font(.system(size: 13))
                        .foregroundStyle(SalonColor.agentVerifiedBlue)
                        .lineLimit(1)
                }
            }
        }
    }

    private var contentBlock: some View {
        VStack(alignment: .leading, spacing: 8) {
            if let quote = post.quoteBody, !quote.isEmpty {
                Text(quote)
                    .font(.system(size: 16))
                    .lineSpacing(DJ.lineSpacing)
                    .foregroundStyle(.primary)
            }
            if let body = post.body, !body.isEmpty {
                Text(body)
                    .font(.system(size: 16))
                    .lineSpacing(DJ.lineSpacing)
                    .foregroundStyle(.primary)
                    .lineLimit(compact ? 8 : nil)
            }
        }
    }

    private var actionRow: some View {
        HStack(spacing: 22) {
            ActionIcon(system: "bubble.left") {
                showReply = true
            }
                .overlay(alignment: .trailing) {
                    if post.replyCount > 0 {
                        Text("\(post.replyCount)")
                            .font(.system(size: 12, weight: .semibold))
                            .foregroundStyle(.secondary)
                            .offset(x: 14, y: 0)
                    }
                }

            ActionIcon(system: "arrow.2.squarepath") {
                store.repost(post.id, quoteBody: nil)
            }
            .overlay(alignment: .trailing) {
                if post.repostCount > 0 {
                    Text("\(post.repostCount)")
                        .font(.system(size: 12, weight: .semibold))
                        .foregroundStyle(.secondary)
                        .offset(x: 14, y: 0)
                }
            }

            LikeButton(isLiked: post.likedByYou, count: post.likeCount) {
                triggerLike()
            }

            ActionIcon(system: "square.and.arrow.up") { /* share placeholder */ }

            Spacer()
        }
        .padding(.top, 4)
    }

    // MARK: Helpers

    private var referencedPost: Post? {
        guard let id = post.referencedPostId, post.kind == .repost else { return nil }
        return store.post(byId: id)
    }

    private var relativeTime: String {
        let formatter = RelativeDateTimeFormatter()
        formatter.unitsStyle = .short
        return formatter.localizedString(for: post.createdDate, relativeTo: Date())
    }

    private func triggerLike() {
        let willLike = !post.likedByYou
        store.toggleLike(postId: post.id)
        UIImpactFeedbackGenerator(style: .light).impactOccurred()
        if willLike {
            withAnimation(.easeOut(duration: 0.12)) { likeBurst = true }
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.28) {
                withAnimation(.easeIn(duration: 0.18)) { likeBurst = false }
            }
        }
    }
}

private struct ReferencedPostPreview: View {
    let post: Post

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(spacing: 6) {
                Avatar(actor: post.author, size: 22)
                Text(post.author.displayName)
                    .font(.system(size: 13, weight: .semibold))
                if post.author.isAgent { AgentBadge(size: 11) }
                Text("@\(post.author.handle)")
                    .font(.system(size: 12))
                    .foregroundStyle(.secondary)
                Spacer()
            }
            if let body = post.body {
                Text(body)
                    .font(.system(size: 14))
                    .foregroundStyle(.primary.opacity(0.85))
                    .lineLimit(3)
            }
        }
        .padding(10)
        .overlay(
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .stroke(Color.black.opacity(0.08), lineWidth: 1)
        )
    }
}
