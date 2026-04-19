import SwiftUI

struct ComposerSheet: View {
    @Environment(\.dismiss) private var dismiss
    @EnvironmentObject private var store: SalonStore
    
    @State private var text = ""
    @State private var isPosting = false
    @State private var errorMessage: String?
    @FocusState private var isEditorFocused: Bool
    
    private let characterLimit = 280
    private var isTextEmpty: Bool { text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty }
    private var isOverLimit: Bool { text.count > characterLimit }
    private var canPost: Bool { !isTextEmpty && !isOverLimit && !isPosting }

    var body: some View {
        NavigationStack {
            VStack(alignment: .leading, spacing: 0) {
                if let error = errorMessage {
                    Text(error)
                        .font(.system(size: 13, weight: .medium))
                        .foregroundStyle(.white)
                        .padding(.vertical, 8)
                        .padding(.horizontal, 16)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(Color.red.opacity(0.8))
                        .transition(.move(edge: .top).combined(with: .opacity))
                }

                HStack(alignment: .top, spacing: 12) {
                    Avatar(actor: store.currentUser)
                        .padding(.top, 4)
                    
                    VStack(alignment: .leading, spacing: 4) {
                        TextEditor(text: $text)
                            .font(.system(size: 17))
                            .scrollContentBackground(.hidden)
                            .frame(maxWidth: .infinity, minHeight: 120)
                            .focused($isEditorFocused)
                            .disabled(isPosting)
                        
                        if isOverLimit || text.count > characterLimit - 20 {
                            Text("\(characterLimit - text.count)")
                                .font(.system(size: 13, weight: .medium, design: .monospaced))
                                .foregroundStyle(isOverLimit ? .red : .secondary)
                                .frame(maxWidth: .infinity, alignment: .trailing)
                                .padding(.trailing, 8)
                        }
                    }
                }
                .padding(16)

                Spacer()
            }
            .background(SalonColor.canvas)
            .navigationTitle("发布新帖")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    Button("取消") {
                        if !text.isEmpty {
                            // 可选：添加确认弹窗
                        }
                        dismiss()
                    }
                    .font(.system(size: 16, weight: .medium))
                    .disabled(isPosting)
                }
                
                ToolbarItem(placement: .topBarTrailing) {
                    Button {
                        handlePost()
                    } label: {
                        if isPosting {
                            ProgressView()
                                .controlSize(.small)
                        } else {
                            Text("发布")
                                .font(.system(size: 16, weight: .bold))
                        }
                    }
                    .disabled(!canPost)
                }
            }
            .onAppear {
                isEditorFocused = true
            }
        }
    }

    private func handlePost() {
        Task {
            isPosting = true
            errorMessage = nil
            do {
                try await store.createPost(body: text)
                dismiss()
            } catch {
                errorMessage = error.localizedDescription
                isPosting = false
            }
        }
    }
}
