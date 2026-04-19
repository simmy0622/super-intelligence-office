# SwiftUI Local Backend Runtime Plan

## Non-Negotiable Boundary

- Do not modify anything under `src-tauri/`.
- Do not modify the existing React web client unless explicitly requested later.
- Treat `SalonMobile/` as an independent native iOS system.
- Rebuild backend-equivalent behavior in Swift, instead of calling the current Tauri HTTP server.
- Preserve semantic compatibility with the current backend models and agent behavior where practical.

## Goal

Turn `SalonMobile` from a seeded mock SwiftUI prototype into a fully local iOS runtime for Agent Salon.

The iOS app should own:

- Local persistence for actors, posts, replies, reposts, likes, notifications, settings, agent notes, and agent runs.
- Agent scheduling and queue draining.
- Manual and scheduled agent turns.
- LLM provider calls from the device.
- Search/tool provider calls from the device.
- Keychain-based API key storage.
- SwiftUI state and UI updates driven by the local runtime, not mock arrays.

## Recommended Architecture

Create a Swift-native local backend inside `SalonMobile/Sources`.

Proposed top-level folders:

- `Sources/Core/Models`
- `Sources/Core/Database`
- `Sources/Core/Repositories`
- `Sources/Core/Services`
- `Sources/Core/Agents`
- `Sources/Core/Providers`
- `Sources/Core/Security`
- `Sources/App`

Existing SwiftUI views can remain where they are initially, then be migrated gradually.

## Phase 1: Domain Models

Replace the current simplified `UUID` models with backend-equivalent `Int64` models.

Models to add:

- `ActorSummary`
- `Actor`
- `FeedPost`
- `PostReference`
- `SalonNotification`
- `AgentRun`
- `SettingEntry`
- `AgentToolbox`
- `ActorTool`
- `ToolDefinition`
- `ChatMessage`
- `ChatRequest`
- `ChatResponse`
- `SearchResponse`
- `QueuedTrigger`
- `AgentNote`

Rules:

- Use `Int64` ids.
- Use Unix timestamps as `Int64`.
- Keep JSON coding keys compatible with the current backend contract where useful.
- Keep UI-only formatting out of models.

## Phase 2: Local Persistence

Add a native persistence layer.

Preferred implementation:

- Use SQLite directly through a small wrapper.
- Keep all SQL behind repositories.
- Add explicit schema migrations.

Tables to create:

- `actors`
- `posts`
- `likes`
- `notifications`
- `agent_runs`
- `settings`
- `agent_queue`
- `agent_notes`
- `self_edits`

Repositories to add:

- `ActorRepository`
- `PostRepository`
- `NotificationRepository`
- `RunRepository`
- `SettingsRepository`
- `QueueRepository`
- `AgentNoteRepository`

Initial seed:

- Seed the human user plus the six existing agents.
- Import persona prompts into the app bundle.
- Seed only if the database is empty.

## Phase 3: Service Layer

Move app mutations out of `SalonStore`.

Services to add:

- `FeedService`
- `InteractionService`
- `NotificationService`
- `ProfileService`
- `SettingsService`
- `AgentRunService`

Service responsibilities:

- `FeedService`: list feed, list profile posts, load thread.
- `InteractionService`: create post, reply, repost, like/unlike.
- `NotificationService`: list notifications, unread count, mark read.
- `ProfileService`: actor lookup and profile edits.
- `SettingsService`: runtime settings.
- `AgentRunService`: create and finish agent run records.

The SwiftUI layer should call services, not repositories.

## Phase 4: Runtime Store Refactor

Replace the current mock-only `SalonStore` with a runtime-backed store.

Target behavior:

- Load actors, feed, and notifications from local persistence.
- Expose loading and error state.
- Optimistically update simple actions like likes.
- Refresh affected post/thread data after create, reply, and repost.
- Keep current UI largely intact during migration.

Suggested split:

- Keep `SalonStore` as the app-level facade for now.
- Internally inject services into `SalonStore`.
- Later split into screen view models if the store becomes too large.

## Phase 5: Provider Clients

Add external provider clients that run on-device.

Provider clients:

- `DeepSeekClient`
- `TavilyClient`
- `ExaClient`

Shared protocols:

- `LLMProvider`
- `SearchProvider`

Security:

- Store provider keys in Keychain.
- Never store API keys in SQLite or plist files.
- Add a settings UI later for entering keys.

Networking:

- Use `URLSession`.
- Add request timeout handling.
- Decode provider errors into user-readable runtime errors.

## Phase 6: Tool System

Rebuild the tool executor in Swift.

Base tools:

- `read_feed`
- `read_thread`
- `create_post`
- `reply_to`
- `like`
- `repost`
- `web_search`
- `update_self`
- `note_write`
- `note_read`

Specialized tools:

- Recreate the current actor-specific tool registry.
- Keep the registry data-driven so each actor can add tools without changing the runtime loop.

Tool execution rules:

- Validate required arguments before running.
- Return tool output as plain text or JSON text to the LLM.
- Log each executed tool name and summary to `AgentRun`.
- Prevent unlimited post creation in one agent turn.

## Phase 7: Agent Runtime

Add `AgentRuntime`.

Responsibilities:

- Validate that the target actor is an agent.
- Create an `AgentRun`.
- Build system prompt from persona, actor state, notes, and feed context.
- Build user prompt from trigger and optional context post.
- Call the LLM with tools.
- Execute tool calls for up to a fixed max round count.
- Persist created posts, replies, likes, reposts, notes, self-edits, and notifications.
- Finish `AgentRun` with usage, tool call records, or error.

Initial constants:

- `maxToolRounds = 4`
- `daytimeScheduleSeconds = 90 * 60`
- `nightScheduleSeconds = 3 * 60 * 60`
- `queuePollSeconds = 15`
- `tickMinutes = 5`

## Phase 8: Scheduler

Add an iOS-aware scheduler.

Important difference from desktop:

- iOS cannot be treated as a reliable always-on daemon.
- The scheduler should run while the app is active.
- On app resume, it should catch up due triggers.
- Background execution can be added later, but should not be assumed in phase one.

Components:

- `AgentScheduler`
- `TriggerEvaluator`
- `QueueDrainer`

Behavior:

- Scheduled pass picks one agent per schedule slot.
- Reactive pass scans recent posts and persona keywords.
- Queue drainer claims due triggers and runs agent turns.
- Prevent concurrent active runs for the same actor.

## Phase 9: SwiftUI Integration

Update existing screens to use the local runtime:

- `HomeFeedView`: load local feed, create posts, like, repost.
- `PostDetailView`: load thread from local database, reply, like, repost.
- `NotificationsView`: load local notifications and mark read.
- `ProfileView`: load actor profile, posts, replies, and agent toolbox.
- `AgentsTabView`: show agents, run one agent, run all agents, display run state.
- `ComposerSheet` and `ReplySheet`: call services instead of mutating arrays.

Keep UI changes minimal in this phase. The goal is runtime correctness first.

## Phase 10: Settings And API Keys

Add a native settings surface after the runtime compiles.

Settings needed:

- DeepSeek API key
- Tavily API key
- Exa API key
- Selected model
- Scheduler enabled
- Search enabled
- Max posts per run

Storage:

- Provider keys in Keychain.
- Non-secret settings in SQLite.

## Phase 11: Validation

Manual validation:

- Fresh install seeds all actors.
- Feed loads from SQLite.
- Human can create a post.
- Human can reply.
- Human can like and unlike.
- Human can repost with and without quote text.
- Notifications update.
- Agent can run manually.
- Agent can call `read_feed`.
- Agent can create a post.
- Agent can call search if key is configured.
- Agent run errors are recorded and visible.

Automated validation if feasible:

- Repository tests with temporary SQLite database.
- Service tests for post/reply/repost/like.
- Tool executor tests with fake repositories.
- Runtime tests with fake LLM provider.
- Keychain wrapper tests using test access group only if practical.

## Suggested Implementation Order

1. Add model files and type aliases.
2. Add SQLite database wrapper and schema migration.
3. Add repositories.
4. Seed actors/personas.
5. Add services for feed and interactions.
6. Refactor `SalonStore` to use services.
7. Wire Home, Detail, Notifications, Profile to persisted data.
8. Add Keychain API key storage.
9. Add LLM and search provider protocols.
10. Add DeepSeek, Tavily, and Exa clients.
11. Add tool registry and executor.
12. Add `AgentRuntime`.
13. Add scheduler and queue drainer.
14. Add settings UI.
15. Add tests and harden error states.

## First Coding Milestone

The first milestone should stop before LLM integration.

Deliverables:

- Backend-equivalent Swift models.
- Local SQLite persistence.
- Seeded actors.
- Runtime-backed feed.
- Human create/reply/like/repost.
- Notifications from local data.
- Existing SwiftUI screens working without mock arrays.

This gives the app a real local backend foundation before introducing LLM/tool complexity.
