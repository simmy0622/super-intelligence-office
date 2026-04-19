import Foundation

struct ChatRequest: Encodable {
    var model: String
    var messages: [ChatMessage]
    var tools: [ToolDefinition]?
    var maxTokens: Int?

    enum CodingKeys: String, CodingKey {
        case model, messages, tools
        case maxTokens = "max_tokens"
    }
}

struct ChatResponse: Decodable {
    var choices: [ChatChoice]
    var usage: ChatUsage?
}

struct ChatChoice: Decodable {
    var message: ChatMessage
}

struct ChatUsage: Decodable {
    var promptTokens: Int64
    var completionTokens: Int64

    enum CodingKeys: String, CodingKey {
        case promptTokens = "prompt_tokens"
        case completionTokens = "completion_tokens"
    }
}

struct ChatMessage: Codable, Hashable {
    var role: String
    var content: String?
    var reasoningContent: String?
    var toolCalls: [ChatToolCall]?
    var toolCallId: String?

    enum CodingKeys: String, CodingKey {
        case role, content
        case reasoningContent = "reasoning_content"
        case toolCalls = "tool_calls"
        case toolCallId = "tool_call_id"
    }

    static func system(_ content: String) -> Self {
        Self(role: "system", content: content, reasoningContent: nil, toolCalls: nil, toolCallId: nil)
    }

    static func user(_ content: String) -> Self {
        Self(role: "user", content: content, reasoningContent: nil, toolCalls: nil, toolCallId: nil)
    }

    static func tool(id: String, content: String) -> Self {
        Self(role: "tool", content: content, reasoningContent: nil, toolCalls: nil, toolCallId: id)
    }
}

struct ChatToolCall: Codable, Hashable {
    var id: String
    var type: String
    var function: ChatToolFunctionCall
}

struct ChatToolFunctionCall: Codable, Hashable {
    var name: String
    var arguments: String
}

struct ToolDefinition: Codable, Hashable {
    var type: String = "function"
    var function: ToolFunction
}

struct ToolFunction: Codable, Hashable {
    var name: String
    var description: String
    var parameters: JSONValue
}

struct SearchResponse: Codable, Hashable {
    var provider: String
    var query: String
    var answer: String?
    var results: [SearchHit]
}

struct SearchHit: Codable, Hashable {
    var title: String
    var url: String
    var snippet: String
    var score: Double?
}

enum ProviderError: LocalizedError {
    case missingAPIKey(String)
    case badStatus(provider: String, status: Int, body: String)
    case decodeFailed(provider: String, body: String)

    var errorDescription: String? {
        switch self {
        case .missingAPIKey(let provider):
            return "Missing API key for \(provider)."
        case .badStatus(let provider, let status, let body):
            return "\(provider) request failed with HTTP \(status): \(body)"
        case .decodeFailed(let provider, let body):
            return "\(provider) response could not be decoded: \(body)"
        }
    }
}

enum JSONValue: Codable, Hashable {
    case string(String)
    case number(Double)
    case bool(Bool)
    case object([String: JSONValue])
    case array([JSONValue])
    case null

    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        if container.decodeNil() {
            self = .null
        } else if let value = try? container.decode(Bool.self) {
            self = .bool(value)
        } else if let value = try? container.decode(Double.self) {
            self = .number(value)
        } else if let value = try? container.decode(String.self) {
            self = .string(value)
        } else if let value = try? container.decode([JSONValue].self) {
            self = .array(value)
        } else {
            self = .object(try container.decode([String: JSONValue].self))
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        switch self {
        case .string(let value): try container.encode(value)
        case .number(let value): try container.encode(value)
        case .bool(let value): try container.encode(value)
        case .object(let value): try container.encode(value)
        case .array(let value): try container.encode(value)
        case .null: try container.encodeNil()
        }
    }
}
