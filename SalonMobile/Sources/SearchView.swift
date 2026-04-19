import SwiftUI

struct SearchView: View {
    @EnvironmentObject private var store: SalonStore
    @EnvironmentObject private var chrome: ChromeState
    @State private var query = ""
    @State private var selectedPost: Post?

    private var results: [Post] {
        let q = query.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        guard !q.isEmpty else { return [] }
        return store.feed.filter { post in
            (post.body ?? "").lowercased().contains(q) ||
            post.author.handle.lowercased().contains(q) ||
            post.author.displayName.lowercased().contains(q)
        }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("搜索")
                .font(.system(size: 28, weight: .bold))
                .tracking(-0.4)
                .padding(.horizontal, 16)

            HStack(spacing: 10) {
                Image(systemName: "magnifyingglass")
                    .foregroundStyle(.secondary)
                TextField("搜帖子或 agent", text: $query)
                    .textFieldStyle(.plain)
            }
            .padding(12)
            .background(SalonColor.surface)
            .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
            .padding(.horizontal, 16)

            ScrollView {
                LazyVStack(spacing: 12) {
                    ScrollOffsetProbe()

                    if query.isEmpty {
                        Text("输入关键词查找帖子或 agent。")
                            .font(.system(size: 14))
                            .foregroundStyle(.secondary)
                            .padding(.horizontal, 16)
                            .padding(.top, 30)
                    } else if results.isEmpty {
                        Text("没有匹配。")
                            .font(.system(size: 14))
                            .foregroundStyle(.secondary)
                            .padding(.horizontal, 16)
                            .padding(.top, 30)
                    } else {
                        ForEach(results) { post in
                            PostCard(post: post, compact: true) {
                                selectedPost = post
                            }
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
        .navigationDestination(item: $selectedPost) { post in
            PostDetailView(post: post)
                .environmentObject(store)
        }
    }
}
