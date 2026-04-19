import SwiftUI

struct PostDetailView: View {
    @Environment(\.dismiss) private var dismiss
    @EnvironmentObject private var store: SalonStore
    @EnvironmentObject private var chrome: ChromeState
    let post: Post
    @State private var showReply = false

    private var currentPost: Post {
        store.post(byId: post.id) ?? post
    }

    private var replies: [Post] {
        store.replies(to: post.id).sorted { $0.createdAt < $1.createdAt }
    }

    var body: some View {
        VStack(spacing: 0) {
            topBar

            ScrollView {
                VStack(spacing: 12) {
                    PostCard(post: currentPost)
                        .environmentObject(store)
                        .padding(.horizontal, 12)
                        .padding(.top, 10)

                    if !replies.isEmpty {
                        HStack {
                            Text("回复")
                                .font(.system(size: 18, weight: .bold))
                                .tracking(-0.2)
                            Spacer()
                        }
                        .padding(.horizontal, 16)
                        .padding(.top, 8)

                        ForEach(replies) { reply in
                            PostCard(post: reply)
                                .environmentObject(store)
                                .padding(.horizontal, 12)
                        }
                    }

                    Spacer(minLength: DJ.bottomContentPadding)
                }
            }

            bottomComposerBar
        }
        .background(SalonColor.canvas)
        .toolbar(.hidden, for: .navigationBar)
        .sheet(isPresented: $showReply) {
            ReplySheet(parentId: currentPost.id)
                .environmentObject(store)
                .presentationDetents([.large])
                .presentationDragIndicator(.visible)
        }
        .onAppear {
            chrome.revealTabBar()
        }
    }

    private var topBar: some View {
        HStack(spacing: 10) {
            Button {
                dismiss()
            } label: {
                HStack(spacing: 4) {
                    Image(systemName: "chevron.left")
                        .font(.system(size: 16, weight: .semibold))
                    Text("返回")
                        .font(.system(size: 16, weight: .semibold))
                }
                .foregroundStyle(.primary)
                .frame(minWidth: 60, alignment: .leading)
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)

            Spacer()

            Text("帖子")
                .font(.system(size: 17, weight: .semibold))

            Spacer()

            Image(systemName: "ellipsis")
                .font(.system(size: 17, weight: .semibold))
                .foregroundStyle(.secondary)
                .frame(minWidth: 60, alignment: .trailing)
        }
        .padding(.horizontal, 16)
        .padding(.top, 10)
        .padding(.bottom, 8)
    }

    private var bottomComposerBar: some View {
        HStack(spacing: 10) {
            Button { showReply = true } label: {
                HStack(spacing: 8) {
                    Image(systemName: "bubble.left")
                    Text("写回复").font(.system(size: 15, weight: .semibold))
                }
                .padding(.horizontal, 14)
                .padding(.vertical, 10)
                .background(SalonColor.surface)
                .clipShape(Capsule())
            }
            .buttonStyle(.plain)

            Spacer()

            if currentPost.author.isAgent {
                HStack(spacing: 6) {
                    AgentBadge(size: 12)
                    Text("@\(currentPost.author.handle)")
                        .font(.system(size: 13, weight: .semibold))
                        .foregroundStyle(.secondary)
                }
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 10)
        .background(SalonColor.canvas.opacity(0.96))
    }
}
