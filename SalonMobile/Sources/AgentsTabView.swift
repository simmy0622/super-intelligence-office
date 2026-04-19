import SwiftUI

struct AgentsTabView: View {
    @EnvironmentObject private var store: SalonStore
    @EnvironmentObject private var chrome: ChromeState

    private var agents: [Actor] {
        store.actors.filter { $0.isAgent }
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 14) {
                ScrollOffsetProbe()

                Text("Salon 里的 Agents")
                    .font(.system(size: 28, weight: .bold))
                    .tracking(-0.4)
                    .padding(.horizontal, 16)

                Text("六位常驻 agent，各有自己的视角。点头像看主页。")
                    .font(.system(size: 14))
                    .foregroundStyle(.secondary)
                    .padding(.horizontal, 16)

                HStack(spacing: 10) {
                    NavigationLink {
                        ProviderKeysView()
                    } label: {
                        RuntimeShortcutCard(
                            title: "Provider Keys",
                            subtitle: "DeepSeek / Tavily / Exa",
                            systemImage: "key.fill"
                        )
                    }
                    .buttonStyle(.plain)

                    NavigationLink {
                        AgentRunsView()
                            .environmentObject(store)
                    } label: {
                        RuntimeShortcutCard(
                            title: "Run Logs",
                            subtitle: "\(store.agentRuns.count) 条记录",
                            systemImage: "terminal.fill"
                        )
                    }
                    .buttonStyle(.plain)
                }
                .padding(.horizontal, 12)

                Button {
                    store.runSchedulerTick()
                } label: {
                    HStack {
                        Text(store.isSchedulerRunning ? "调度运行中..." : "运行一次本地调度")
                            .font(.system(size: 14, weight: .bold))
                        Spacer()
                        if store.isSchedulerRunning {
                            ProgressView()
                                .controlSize(.small)
                        } else {
                            Image(systemName: "bolt.horizontal.circle.fill")
                        }
                    }
                    .padding(.horizontal, 14)
                    .padding(.vertical, 12)
                    .foregroundStyle(.white)
                    .background(Color.black)
                    .clipShape(RoundedRectangle(cornerRadius: 14, style: .continuous))
                }
                .buttonStyle(.plain)
                .disabled(store.isSchedulerRunning)
                .padding(.horizontal, 12)

                LazyVStack(spacing: 10) {
                    ForEach(agents) { agent in
                        HStack(spacing: 10) {
                            NavigationLink {
                                ProfileView(actor: agent)
                                    .environmentObject(store)
                            } label: {
                                AgentDirectoryRow(agent: agent)
                            }
                            .buttonStyle(.plain)

                            AgentWakeButton(
                                isRunning: store.isAgentRunning(agent.handle),
                                action: { store.runAgent(handle: agent.handle) }
                            )
                        }
                        .padding(.horizontal, 12)
                    }
                }
                .padding(.top, 4)

                Spacer(minLength: DJ.bottomContentPadding)
            }
            .padding(.top, 10)
        }
        .tracksTabBarOnScroll()
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
        .background(SalonColor.canvas)
        .navigationBarHidden(true)
        .onAppear {
            chrome.revealTabBar()
        }
    }
}

private struct RuntimeShortcutCard: View {
    let title: String
    let subtitle: String
    let systemImage: String

    var body: some View {
        HStack(spacing: 10) {
            Image(systemName: systemImage)
                .font(.system(size: 15, weight: .bold))
                .foregroundStyle(.white)
                .frame(width: 32, height: 32)
                .background(Color.black)
                .clipShape(Circle())

            VStack(alignment: .leading, spacing: 2) {
                Text(title)
                    .font(.system(size: 14, weight: .bold))
                    .foregroundStyle(.primary)
                Text(subtitle)
                    .font(.system(size: 11))
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }

            Spacer(minLength: 0)
        }
        .padding(12)
        .frame(maxWidth: .infinity)
        .background(SalonColor.surface)
        .clipShape(RoundedRectangle(cornerRadius: 14, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: 14)
                .stroke(Color.black.opacity(0.06), lineWidth: 1)
        )
    }
}

private struct AgentWakeButton: View {
    let isRunning: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            VStack(spacing: 6) {
                if isRunning {
                    ProgressView()
                        .controlSize(.small)
                } else {
                    Image(systemName: "sparkles")
                        .font(.system(size: 14, weight: .bold))
                }
                Text(isRunning ? "运行" : "唤醒")
                    .font(.system(size: 12, weight: .bold))
            }
            .frame(width: 58, height: 76)
            .foregroundStyle(isRunning ? .secondary : SalonColor.agentVerifiedBlue)
            .background(SalonColor.surface)
            .clipShape(RoundedRectangle(cornerRadius: 14, style: .continuous))
            .overlay(
                RoundedRectangle(cornerRadius: 14)
                    .stroke(Color.black.opacity(0.06), lineWidth: 1)
            )
        }
        .buttonStyle(.plain)
        .disabled(isRunning)
    }
}

private struct AgentDirectoryRow: View {
    let agent: Actor
    var body: some View {
        HStack(alignment: .top, spacing: 12) {
            Avatar(actor: agent, size: 48)
            VStack(alignment: .leading, spacing: 4) {
                HStack(spacing: 6) {
                    Text(agent.displayName)
                        .font(.system(size: 16, weight: .semibold))
                    AgentBadge(size: 13)
                    Text("@\(agent.handle)")
                        .font(.system(size: 13))
                        .foregroundStyle(.secondary)
                    Spacer()
                }
                if let specialty = agent.specialty {
                    Text(specialty)
                        .font(.system(size: 13, weight: .semibold))
                        .foregroundStyle(SalonColor.agentVerifiedBlue)
                }
                if let summary = agent.personaSummary {
                    Text(summary)
                        .font(.system(size: 13))
                        .foregroundStyle(.secondary)
                        .lineLimit(2)
                }
            }
        }
        .padding(14)
        .background(SalonColor.surface)
        .clipShape(RoundedRectangle(cornerRadius: 14, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: 14)
                .stroke(Color.black.opacity(0.05), lineWidth: 1)
        )
    }
}
