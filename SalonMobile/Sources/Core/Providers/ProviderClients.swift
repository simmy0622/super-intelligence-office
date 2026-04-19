import Foundation

protocol LLMProvider: Sendable {
    func chatCompletion(_ request: ChatRequest) async throws -> ChatResponse
}

protocol SearchProvider: Sendable {
    func search(query: String, maxResults: Int) async throws -> SearchResponse
}

final class DeepSeekClient: LLMProvider {
    static let endpoint = URL(string: "https://api.deepseek.com/v1/chat/completions")!
    static let defaultModel = "deepseek-reasoner"

    private let apiKeyStore: APIKeyStore
    private let session: URLSession
    private let endpoint: URL

    init(
        apiKeyStore: APIKeyStore = .shared,
        session: URLSession = .shared,
        endpoint: URL = DeepSeekClient.endpoint
    ) {
        self.apiKeyStore = apiKeyStore
        self.session = session
        self.endpoint = endpoint
    }

    func chatCompletion(_ request: ChatRequest) async throws -> ChatResponse {
        guard let apiKey = try apiKeyStore.load(provider: "deepseek"), !apiKey.isEmpty else {
            throw ProviderError.missingAPIKey("deepseek")
        }

        var urlRequest = URLRequest(url: endpoint)
        urlRequest.httpMethod = "POST"
        urlRequest.timeoutInterval = 90
        urlRequest.setValue("Bearer \(apiKey)", forHTTPHeaderField: "Authorization")
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.httpBody = try JSONEncoder().encode(request)

        let (data, response) = try await session.data(for: urlRequest)
        return try decodeProviderResponse(
            data: data,
            response: response,
            provider: "DeepSeek",
            as: ChatResponse.self
        )
    }
}

final class TavilyClient: SearchProvider {
    static let endpoint = URL(string: "https://api.tavily.com/search")!

    private let apiKeyStore: APIKeyStore
    private let session: URLSession
    private let endpoint: URL

    init(
        apiKeyStore: APIKeyStore = .shared,
        session: URLSession = .shared,
        endpoint: URL = TavilyClient.endpoint
    ) {
        self.apiKeyStore = apiKeyStore
        self.session = session
        self.endpoint = endpoint
    }

    func search(query: String, maxResults: Int) async throws -> SearchResponse {
        guard let apiKey = try apiKeyStore.load(provider: "tavily"), !apiKey.isEmpty else {
            throw ProviderError.missingAPIKey("tavily")
        }

        let body = TavilySearchRequest(
            apiKey: apiKey,
            query: query,
            maxResults: max(1, min(maxResults, 10)),
            searchDepth: "basic",
            includeAnswer: true
        )

        var urlRequest = URLRequest(url: endpoint)
        urlRequest.httpMethod = "POST"
        urlRequest.timeoutInterval = 30
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.httpBody = try JSONEncoder().encode(body)

        let (data, response) = try await session.data(for: urlRequest)
        let raw = try decodeProviderResponse(
            data: data,
            response: response,
            provider: "Tavily",
            as: TavilySearchResponse.self
        )

        return SearchResponse(
            provider: "tavily",
            query: query,
            answer: raw.answer,
            results: raw.results.map {
                SearchHit(title: $0.title, url: $0.url, snippet: $0.content, score: $0.score)
            }
        )
    }
}

final class ExaClient: SearchProvider {
    static let endpoint = URL(string: "https://api.exa.ai/search")!

    private let apiKeyStore: APIKeyStore
    private let session: URLSession
    private let endpoint: URL

    init(
        apiKeyStore: APIKeyStore = .shared,
        session: URLSession = .shared,
        endpoint: URL = ExaClient.endpoint
    ) {
        self.apiKeyStore = apiKeyStore
        self.session = session
        self.endpoint = endpoint
    }

    func search(query: String, maxResults: Int) async throws -> SearchResponse {
        guard let apiKey = try apiKeyStore.load(provider: "exa"), !apiKey.isEmpty else {
            throw ProviderError.missingAPIKey("exa")
        }

        let body = ExaSearchRequest(
            query: query,
            numResults: max(1, min(maxResults, 10)),
            contents: ExaSearchRequest.Contents(text: .init(maxCharacters: 800))
        )

        var urlRequest = URLRequest(url: endpoint)
        urlRequest.httpMethod = "POST"
        urlRequest.timeoutInterval = 30
        urlRequest.setValue("application/json", forHTTPHeaderField: "Content-Type")
        urlRequest.setValue(apiKey, forHTTPHeaderField: "x-api-key")
        urlRequest.httpBody = try JSONEncoder().encode(body)

        let (data, response) = try await session.data(for: urlRequest)
        let raw = try decodeProviderResponse(
            data: data,
            response: response,
            provider: "Exa",
            as: ExaSearchResponse.self
        )

        return SearchResponse(
            provider: "exa",
            query: query,
            answer: nil,
            results: raw.results.map {
                SearchHit(
                    title: $0.title ?? "",
                    url: $0.url ?? "",
                    snippet: $0.text ?? $0.snippet ?? "",
                    score: $0.score
                )
            }
        )
    }
}

private func decodeProviderResponse<T: Decodable>(
    data: Data,
    response: URLResponse,
    provider: String,
    as type: T.Type
) throws -> T {
    let body = String(data: data, encoding: .utf8) ?? ""
    guard let http = response as? HTTPURLResponse else {
        throw ProviderError.badStatus(provider: provider, status: -1, body: body)
    }
    guard (200..<300).contains(http.statusCode) else {
        throw ProviderError.badStatus(provider: provider, status: http.statusCode, body: body)
    }

    do {
        return try JSONDecoder().decode(type, from: data)
    } catch {
        throw ProviderError.decodeFailed(provider: provider, body: body)
    }
}

private struct TavilySearchRequest: Encodable {
    var apiKey: String
    var query: String
    var maxResults: Int
    var searchDepth: String
    var includeAnswer: Bool

    enum CodingKeys: String, CodingKey {
        case apiKey = "api_key"
        case query
        case maxResults = "max_results"
        case searchDepth = "search_depth"
        case includeAnswer = "include_answer"
    }
}

private struct TavilySearchResponse: Decodable {
    var answer: String?
    var results: [ResultItem]

    struct ResultItem: Decodable {
        var title: String
        var url: String
        var content: String
        var score: Double?
    }
}

private struct ExaSearchRequest: Encodable {
    var query: String
    var numResults: Int
    var contents: Contents

    struct Contents: Encodable {
        var text: Text
    }

    struct Text: Encodable {
        var maxCharacters: Int
    }
}

private struct ExaSearchResponse: Decodable {
    var results: [ResultItem]

    struct ResultItem: Decodable {
        var title: String?
        var url: String?
        var text: String?
        var snippet: String?
        var score: Double?
    }
}
