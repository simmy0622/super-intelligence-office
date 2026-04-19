import SwiftUI

struct HomeFeedView: View {
    @EnvironmentObject private var store: SalonStore
    @EnvironmentObject private var chrome: ChromeState
    @State private var showComposer = false
    @State private var selectedPost: Post?

    private var rootPosts: [Post] {
        store.feed
            .filter { $0.kind != .reply }
            .sorted { $0.createdAt > $1.createdAt }
    }

    var body: some View {
        GeometryReader { proxy in
            let topInset = proxy.safeAreaInsets.top

            ZStack(alignment: .top) {
                ScrollView {
                    LazyVStack(spacing: 12) {
                        ScrollOffsetProbe()

                        ForEach(rootPosts) { post in
                            PostCard(post: post, compact: true) {
                                selectedPost = post
                            }
                            .environmentObject(store)
                            .padding(.horizontal, 12)
                        }

                        Spacer(minLength: DJ.bottomContentPadding)
                    }
                    .padding(.top, topInset + DJ.homeHeaderHeight + 2) // 保持恒定间距
                }
                .background(SalonColor.canvas)
                .tracksTabBarOnScroll(tracksHomeHeader: true)
                .navigationBarHidden(true)

                fixedHeader(topInset: topInset)

                VStack {
                    Spacer()
                    HStack {
                        Spacer()
                        Button { showComposer = true } label: {
                            Image(systemName: "plus")
                                .font(.system(size: 24, weight: .bold))
                                .frame(width: 64, height: 64)
                                .background(Color.primary)
                                .foregroundStyle(SalonColor.canvas)
                                .clipShape(Circle())
                                .shadow(color: .black.opacity(0.22), radius: 14, y: 8)
                        }
                        .padding(.trailing, 18)
                        .padding(.bottom, chrome.isTabBarHidden ? DJ.composerBottomOffsetHidden : DJ.composerBottomOffset)
                    }
                }
            }
            .ignoresSafeArea(edges: .top)
        }
        .sheet(isPresented: $showComposer) {
            ComposerSheet()
                .environmentObject(store)
                .presentationDetents([.large])
                .presentationDragIndicator(.visible)
        }
        .navigationDestination(item: $selectedPost) { post in
            PostDetailView(post: post)
                .environmentObject(store)
        }
    }

    private func fixedHeader(topInset: CGFloat) -> some View {
        ZStack(alignment: .bottom) {
            SalonColor.canvas
                .ignoresSafeArea(edges: .top)
            
            headerContent
                .offset(y: chrome.isHomeHeaderHidden ? -DJ.homeHeaderHeight : 0)
                .opacity(chrome.isHomeHeaderHidden ? 0 : 1)
        }
        .frame(height: topInset + DJ.homeHeaderHeight)
        // 关键：向上收起必须是负值
        .offset(y: chrome.isHomeHeaderHidden ? -(DJ.homeHeaderHeight - 4) : 0)
        .clipped()
        .zIndex(10)
        .animation(.spring(response: 0.3, dampingFraction: 0.85), value: chrome.isHomeHeaderHidden)
    }

    private var headerContent: some View {
        ZStack {
            Text("ANL")
                .font(.system(size: 24, weight: .bold))
                .tracking(-0.6)

            HStack {
                NavigationLink {
                    ProfileView(actor: store.currentUser)
                        .environmentObject(store)
                } label: {
                    Avatar(actor: store.currentUser)
                }
                .buttonStyle(.plain)
                Spacer()
            }
        }
        .padding(.horizontal, 16)
        .padding(.bottom, 8)
        .frame(height: DJ.homeHeaderHeight)
    }
}
