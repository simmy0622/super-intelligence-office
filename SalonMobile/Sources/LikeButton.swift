import SwiftUI

struct LikeButton: View {
    let isLiked: Bool
    let count: Int
    let onTap: () -> Void

    @State private var scale: CGFloat = 1.0

    var body: some View {
        Button {
            withAnimation(.spring(response: 0.25, dampingFraction: 0.55)) {
                scale = 1.25
            }
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.12) {
                withAnimation(.spring(response: 0.25, dampingFraction: 0.7)) {
                    scale = 1.0
                }
            }
            onTap()
        } label: {
            HStack(spacing: 6) {
                Image(systemName: isLiked ? "heart.fill" : "heart")
                    .font(.system(size: 15, weight: .regular))
                    .foregroundStyle(isLiked ? Color.pink : Color.secondary)
                    .scaleEffect(scale)
                if count > 0 {
                    Text("\(count)")
                        .font(.system(size: 13, weight: .semibold))
                        .foregroundStyle(.secondary)
                }
            }
            .frame(height: 26)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
    }
}

struct LikeBurst: View {
    @State private var show = false

    var body: some View {
        Image(systemName: "heart.fill")
            .font(.system(size: 44, weight: .bold))
            .foregroundStyle(Color.pink.opacity(0.22))
            .scaleEffect(show ? 1.25 : 0.6)
            .opacity(show ? 0.0 : 0.9)
            .onAppear {
                withAnimation(.easeOut(duration: 0.35)) { show = true }
            }
            .allowsHitTesting(false)
    }
}
