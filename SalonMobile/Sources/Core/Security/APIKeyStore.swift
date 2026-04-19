import Foundation
import Security

enum APIKeyStoreError: LocalizedError {
    case unexpectedStatus(OSStatus)
    case invalidData

    var errorDescription: String? {
        switch self {
        case .unexpectedStatus(let status):
            return "Keychain operation failed with status \(status)."
        case .invalidData:
            return "Stored API key could not be decoded."
        }
    }
}

final class APIKeyStore: @unchecked Sendable {
    static let shared = APIKeyStore()

    private let service = "com.shinmuchen.SalonMobile.provider-keys"

    func save(_ key: String, provider: String) throws {
        let account = normalized(provider)
        let data = Data(key.utf8)
        let query = baseQuery(account: account)

        let update: [String: Any] = [
            kSecValueData as String: data,
            kSecAttrAccessible as String: kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly,
        ]

        let status = SecItemUpdate(query as CFDictionary, update as CFDictionary)
        if status == errSecItemNotFound {
            var addQuery = query
            addQuery[kSecValueData as String] = data
            addQuery[kSecAttrAccessible as String] = kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly
            let addStatus = SecItemAdd(addQuery as CFDictionary, nil)
            guard addStatus == errSecSuccess else {
                throw APIKeyStoreError.unexpectedStatus(addStatus)
            }
            return
        }

        guard status == errSecSuccess else {
            throw APIKeyStoreError.unexpectedStatus(status)
        }
    }

    func load(provider: String) throws -> String? {
        var query = baseQuery(account: normalized(provider))
        query[kSecReturnData as String] = true
        query[kSecMatchLimit as String] = kSecMatchLimitOne

        var item: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &item)
        if status == errSecItemNotFound { return nil }
        guard status == errSecSuccess else {
            throw APIKeyStoreError.unexpectedStatus(status)
        }
        guard let data = item as? Data, let value = String(data: data, encoding: .utf8) else {
            throw APIKeyStoreError.invalidData
        }
        return value
    }

    func hasKey(provider: String) throws -> Bool {
        var query = baseQuery(account: normalized(provider))
        query[kSecReturnAttributes as String] = true
        query[kSecMatchLimit as String] = kSecMatchLimitOne

        let status = SecItemCopyMatching(query as CFDictionary, nil)
        if status == errSecItemNotFound { return false }
        guard status == errSecSuccess else {
            throw APIKeyStoreError.unexpectedStatus(status)
        }
        return true
    }

    func delete(provider: String) throws {
        let status = SecItemDelete(baseQuery(account: normalized(provider)) as CFDictionary)
        guard status == errSecSuccess || status == errSecItemNotFound else {
            throw APIKeyStoreError.unexpectedStatus(status)
        }
    }

    private func baseQuery(account: String) -> [String: Any] {
        [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: account,
        ]
    }

    private func normalized(_ provider: String) -> String {
        provider.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
    }
}
