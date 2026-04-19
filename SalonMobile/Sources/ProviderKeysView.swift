import SwiftUI

struct ProviderKeysView: View {
    private let providers: [ProviderKeySpec] = [
        ProviderKeySpec(
            provider: "deepseek",
            title: "DeepSeek",
            purpose: "agent reasoning + tool calling",
            placeholder: "sk-..."
        ),
        ProviderKeySpec(
            provider: "tavily",
            title: "Tavily",
            purpose: "web_search primary provider",
            placeholder: "tvly-..."
        ),
        ProviderKeySpec(
            provider: "exa",
            title: "Exa",
            purpose: "web_search fallback provider",
            placeholder: "exa..."
        ),
    ]

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 14) {
                Text("Provider Keys")
                    .font(.system(size: 28, weight: .bold))
                    .tracking(-0.4)
                    .padding(.horizontal, 16)

                Text("Key 只保存在 iOS Keychain。SwiftUI 本地 runtime 读取这些 key 调用 provider，不写入现有 Tauri 后端。")
                    .font(.system(size: 14))
                    .foregroundStyle(.secondary)
                    .lineSpacing(2)
                    .padding(.horizontal, 16)

                LazyVStack(spacing: 12) {
                    ForEach(providers) { spec in
                        ProviderKeyCard(spec: spec)
                            .padding(.horizontal, 12)
                    }
                }

                Spacer(minLength: DJ.bottomContentPadding)
            }
            .padding(.top, 10)
        }
        .background(SalonColor.canvas)
        .navigationTitle("Provider Keys")
        .navigationBarTitleDisplayMode(.inline)
    }
}

private struct ProviderKeySpec: Identifiable {
    var provider: String
    var title: String
    var purpose: String
    var placeholder: String

    var id: String { provider }
}

private struct ProviderKeyCard: View {
    let spec: ProviderKeySpec

    @State private var key = ""
    @State private var hasSavedKey = false
    @State private var message: String?

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack(alignment: .top, spacing: 10) {
                VStack(alignment: .leading, spacing: 4) {
                    HStack(spacing: 8) {
                        Text(spec.title)
                            .font(.system(size: 17, weight: .bold))
                        statusPill
                    }
                    Text(spec.purpose)
                        .font(.system(size: 13))
                        .foregroundStyle(.secondary)
                }
                Spacer()
            }

            SecureField(hasSavedKey ? "已保存，输入新 key 可覆盖" : spec.placeholder, text: $key)
                .textInputAutocapitalization(.never)
                .autocorrectionDisabled()
                .font(.system(size: 14, design: .monospaced))
                .padding(12)
                .background(Color.white)
                .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
                .overlay(
                    RoundedRectangle(cornerRadius: 12)
                        .stroke(Color.black.opacity(0.08), lineWidth: 1)
                )

            HStack(spacing: 10) {
                Button {
                    save()
                } label: {
                    Text("保存")
                        .font(.system(size: 14, weight: .bold))
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 10)
                        .foregroundStyle(.white)
                        .background(Color.black)
                        .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
                }
                .buttonStyle(.plain)
                .disabled(key.trimmedOrNil() == nil)

                Button {
                    delete()
                } label: {
                    Text("删除")
                        .font(.system(size: 14, weight: .bold))
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 10)
                        .foregroundStyle(.red)
                        .background(Color.red.opacity(0.08))
                        .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
                }
                .buttonStyle(.plain)
                .disabled(!hasSavedKey)
            }

            if let message {
                Text(message)
                    .font(.system(size: 12))
                    .foregroundStyle(.secondary)
            }
        }
        .padding(14)
        .background(SalonColor.surface)
        .clipShape(RoundedRectangle(cornerRadius: 16, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: 16)
                .stroke(Color.black.opacity(0.06), lineWidth: 1)
        )
        .onAppear(perform: reloadStatus)
    }

    private var statusPill: some View {
        Text(hasSavedKey ? "已保存" : "未配置")
            .font(.system(size: 11, weight: .bold))
            .foregroundStyle(hasSavedKey ? .green : .secondary)
            .padding(.horizontal, 8)
            .padding(.vertical, 4)
            .background((hasSavedKey ? Color.green : Color.gray).opacity(0.12))
            .clipShape(Capsule())
    }

    private func reloadStatus() {
        do {
            hasSavedKey = try APIKeyStore.shared.hasKey(provider: spec.provider)
            message = nil
        } catch {
            message = error.localizedDescription
        }
    }

    private func save() {
        guard let trimmed = key.trimmedOrNil() else { return }
        do {
            try APIKeyStore.shared.save(trimmed, provider: spec.provider)
            key = ""
            hasSavedKey = true
            message = "\(spec.title) key 已保存。"
        } catch {
            message = error.localizedDescription
        }
    }

    private func delete() {
        do {
            try APIKeyStore.shared.delete(provider: spec.provider)
            key = ""
            hasSavedKey = false
            message = "\(spec.title) key 已删除。"
        } catch {
            message = error.localizedDescription
        }
    }
}
