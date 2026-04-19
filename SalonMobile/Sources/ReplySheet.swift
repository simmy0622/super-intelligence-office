import SwiftUI

struct ReplySheet: View {
    @Environment(\.dismiss) private var dismiss
    @EnvironmentObject private var store: SalonStore
    let parentId: Int64
    
    @State private var text = ""
    @State private var isReplying = false
    @State private var errorMessage: String?
    @FocusState private var isEditorFocused: Bool
    
    private let characterLimit = 280
    private var parent: Post? { store.post(byId: parentId) }
    private var isTextEmpty: Bool { text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty }
    private var isOverLimit: Bool { text.count > characterLimit }
    private var canReply: Bool { !isTextEmpty && !isOverLimit && !isReplying }

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
                }

                ScrollView {
                    VStack(alignment: .leading, spacing: 0) {
                        if let parent = parent {
                            HStack(alignment: .top, spacing: 12) {
                                VStack(spacing: 0) {
                                    Avatar(actor: parent.author, size: 32)
                                    Rectangle()
                                        .fill(Color.secondary.opacity(0.2))
                                        .frame(width: 2)
                                        .padding(.vertical, 4)
                                }
                                
                                VStack(alignment: .leading, spacing: 4) {
                                    HStack(spacing: 4) {
                                        Text(parent.author.handle)
                                            .font(.system(size: 14, weight: .bold))
                                        if parent.author.isAgent { AgentBadge(size: 10) }
                                    }
                                    
                                    Text(parent.body ?? "")
                                        .font(.system(size: 15))
                                        .lineLimit(4)
                                }
                            }
                            .padding(.horizontal, 16)
                            .padding(.top, 16)
                        }

                        HStack(alignment: .top, spacing: 12) {
                            Avatar(actor: store.currentUser, size: 32)
                            
                            VStack(alignment: .leading, spacing: 4) {
                                TextEditor(text: $text)
                                    .font(.system(size: 17))
                                    .scrollContentBackground(.hidden)
                                    .frame(maxWidth: .infinity, minHeight: 120)
                                    .focused($isEditorFocused)
                                    .disabled(isReplying)
                                
                                if isOverLimit || text.count > characterLimit - 20 {
                                    Text("\(characterLimit - text.count)")
                                        .font(.system(size: 13, weight: .medium, design: .monospaced))
                                        .foregroundStyle(isOverLimit ? .red : .secondary)
                                        .frame(maxWidth: .infinity, alignment: .trailing)
                                }
                            }
                        }
                        .padding(16)
                    }
                }

                Spacer()
            }
            .background(SalonColor.canvas)
            .navigationTitle("回复帖子")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    Button("取消") { dismiss() }
                        .font(.system(size: 16, weight: .medium))
                        .disabled(isReplying)
                }
                ToolbarItem(placement: .topBarTrailing) {
                    Button {
                        handleReply()
                    } label: {
                        if isReplying {
                            ProgressView()
                                .controlSize(.small)
                        } else {
                            Text("回复")
                                .font(.system(size: 16, weight: .bold))
                        }
                    }
                    .disabled(!canReply)
                }
            }
            .onAppear {
                isEditorFocused = true
            }
        }
    }

    private func handleReply() {
        Task {
            isReplying = true
            errorMessage = nil
            do {
                try await store.reply(to: parentId, body: text)
                dismiss()
            } catch {
                errorMessage = error.localizedDescription
                isReplying = false
            }
        }
    }
}
