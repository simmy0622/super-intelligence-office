import SwiftUI

struct AgentRunsView: View {
    @EnvironmentObject private var store: SalonStore

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 14) {
                HStack(alignment: .firstTextBaseline) {
                    Text("Agent Runs")
                        .font(.system(size: 28, weight: .bold))
                        .tracking(-0.4)
                    Spacer()
                    Button {
                        store.refreshAgentRuns()
                    } label: {
                        Image(systemName: "arrow.clockwise")
                            .font(.system(size: 15, weight: .bold))
                            .foregroundStyle(.primary)
                            .frame(width: 34, height: 34)
                            .background(SalonColor.surface)
                            .clipShape(Circle())
                    }
                    .buttonStyle(.plain)
                }
                .padding(.horizontal, 16)

                Text("这里展示 SwiftUI 本地 SQLite runtime 的 agent_runs。失败会保留错误，成功会记录 token 和 tool calls。")
                    .font(.system(size: 14))
                    .foregroundStyle(.secondary)
                    .lineSpacing(2)
                    .padding(.horizontal, 16)

                LazyVStack(spacing: 12) {
                    if store.agentRuns.isEmpty {
                        Text("还没有运行记录。去 Agents 页点击「唤醒」或「运行一次本地调度」。")
                            .font(.system(size: 14))
                            .foregroundStyle(.secondary)
                            .padding(.top, 28)
                            .padding(.horizontal, 16)
                    } else {
                        ForEach(store.agentRuns) { run in
                            AgentRunCard(run: run)
                                .padding(.horizontal, 12)
                        }
                    }
                }

                Spacer(minLength: DJ.bottomContentPadding)
            }
            .padding(.top, 10)
        }
        .background(SalonColor.canvas)
        .navigationTitle("Agent Runs")
        .navigationBarTitleDisplayMode(.inline)
        .onAppear {
            store.refreshAgentRuns()
        }
    }
}

private struct AgentRunCard: View {
    let run: AgentRun
    @State private var expanded = false

    private var isFinished: Bool { run.finishedAt != nil }
    private var hasError: Bool { run.error?.trimmedOrNil() != nil }

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(alignment: .top, spacing: 10) {
                Avatar(seed: run.actorHandle, size: 34)
                VStack(alignment: .leading, spacing: 3) {
                    HStack(spacing: 6) {
                        Text(run.actorDisplayName)
                            .font(.system(size: 15, weight: .bold))
                        Text("@\(run.actorHandle)")
                            .font(.system(size: 13))
                            .foregroundStyle(.secondary)
                    }
                    Text(startedText)
                        .font(.system(size: 12))
                        .foregroundStyle(.secondary)
                }
                Spacer()
                statusPill
            }

            HStack(spacing: 8) {
                Pill(run.trigger, systemImage: "bolt")
                if let promptTokens = run.promptTokens {
                    Pill("prompt \(promptTokens)", systemImage: nil)
                }
                if let completionTokens = run.completionTokens {
                    Pill("completion \(completionTokens)", systemImage: nil)
                }
            }

            if let error = run.error?.trimmedOrNil() {
                Text(error)
                    .font(.system(size: 13))
                    .foregroundStyle(.red)
                    .lineLimit(expanded ? nil : 3)
            }

            if let toolCalls = run.toolCalls?.trimmedOrNil() {
                VStack(alignment: .leading, spacing: 8) {
                    Button {
                        withAnimation(.easeInOut(duration: 0.18)) {
                            expanded.toggle()
                        }
                    } label: {
                        HStack {
                            Text(expanded ? "收起 tool calls" : "查看 tool calls")
                                .font(.system(size: 13, weight: .bold))
                            Spacer()
                            Image(systemName: expanded ? "chevron.up" : "chevron.down")
                                .font(.system(size: 12, weight: .bold))
                        }
                        .foregroundStyle(.primary)
                    }
                    .buttonStyle(.plain)

                    if expanded {
                        Text(toolCalls)
                            .font(.system(size: 11, design: .monospaced))
                            .foregroundStyle(.secondary)
                            .textSelection(.enabled)
                            .padding(10)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .background(Color.white)
                            .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
                    }
                }
            }
        }
        .padding(14)
        .background(SalonColor.surface)
        .clipShape(RoundedRectangle(cornerRadius: 16, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: 16)
                .stroke(Color.black.opacity(0.06), lineWidth: 1)
        )
    }

    private var statusPill: some View {
        Text(statusText)
            .font(.system(size: 11, weight: .bold))
            .foregroundStyle(statusColor)
            .padding(.horizontal, 8)
            .padding(.vertical, 4)
            .background(statusColor.opacity(0.12))
            .clipShape(Capsule())
    }

    private var statusText: String {
        if hasError { return "failed" }
        if isFinished { return "ok" }
        return "running"
    }

    private var statusColor: Color {
        if hasError { return .red }
        if isFinished { return .green }
        return SalonColor.agentVerifiedBlue
    }

    private var startedText: String {
        let date = Date(timeIntervalSince1970: TimeInterval(run.startedAt))
        return Self.formatter.string(from: date)
    }

    private static let formatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateFormat = "yyyy-MM-dd HH:mm:ss"
        return formatter
    }()
}
