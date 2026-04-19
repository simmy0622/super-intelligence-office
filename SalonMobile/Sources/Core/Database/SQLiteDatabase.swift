import Foundation
import SQLite3

enum SQLiteValue: Equatable {
    case null
    case int(Int64)
    case real(Double)
    case text(String)

    var stringValue: String? {
        if case let .text(value) = self { return value }
        return nil
    }

    var intValue: Int64? {
        if case let .int(value) = self { return value }
        return nil
    }
}

enum SQLiteDatabaseError: LocalizedError {
    case openFailed(String)
    case prepareFailed(String)
    case stepFailed(String)
    case bindFailed(String)

    var errorDescription: String? {
        switch self {
        case .openFailed(let message): return "Failed to open database: \(message)"
        case .prepareFailed(let message): return "Failed to prepare SQL: \(message)"
        case .stepFailed(let message): return "Failed to execute SQL: \(message)"
        case .bindFailed(let message): return "Failed to bind SQL value: \(message)"
        }
    }
}

final class SQLiteDatabase {
    private var db: OpaquePointer?
    private let lock = NSRecursiveLock()

    init(filename: String = "salon-mobile.sqlite3") throws {
        let directory = try Self.applicationSupportDirectory()
        let path = directory.appendingPathComponent(filename).path

        guard sqlite3_open(path, &db) == SQLITE_OK else {
            throw SQLiteDatabaseError.openFailed(lastErrorMessage)
        }

        try execute("PRAGMA foreign_keys = ON")
        _ = try query("PRAGMA journal_mode = WAL")
    }

    deinit {
        sqlite3_close(db)
    }

    var lastInsertRowID: Int64 {
        sqlite3_last_insert_rowid(db)
    }

    func execute(_ sql: String, values: [SQLiteValue] = []) throws {
        lock.lock()
        defer { lock.unlock() }

        let statement = try prepare(sql)
        defer { sqlite3_finalize(statement) }

        try bind(values, to: statement)
        guard sqlite3_step(statement) == SQLITE_DONE else {
            throw SQLiteDatabaseError.stepFailed(lastErrorMessage)
        }
    }

    func query(_ sql: String, values: [SQLiteValue] = []) throws -> [[String: SQLiteValue]] {
        lock.lock()
        defer { lock.unlock() }

        let statement = try prepare(sql)
        defer { sqlite3_finalize(statement) }

        try bind(values, to: statement)

        var rows: [[String: SQLiteValue]] = []
        while true {
            let result = sqlite3_step(statement)
            if result == SQLITE_DONE { break }
            guard result == SQLITE_ROW else {
                throw SQLiteDatabaseError.stepFailed(lastErrorMessage)
            }

            var row: [String: SQLiteValue] = [:]
            for index in 0..<sqlite3_column_count(statement) {
                guard let name = sqlite3_column_name(statement, index) else { continue }
                row[String(cString: name)] = columnValue(statement, index: index)
            }
            rows.append(row)
        }

        return rows
    }

    private func prepare(_ sql: String) throws -> OpaquePointer? {
        var statement: OpaquePointer?
        guard sqlite3_prepare_v2(db, sql, -1, &statement, nil) == SQLITE_OK else {
            throw SQLiteDatabaseError.prepareFailed(lastErrorMessage)
        }
        return statement
    }

    private func bind(_ values: [SQLiteValue], to statement: OpaquePointer?) throws {
        for (offset, value) in values.enumerated() {
            let index = Int32(offset + 1)
            let result: Int32
            switch value {
            case .null:
                result = sqlite3_bind_null(statement, index)
            case .int(let int):
                result = sqlite3_bind_int64(statement, index, int)
            case .real(let double):
                result = sqlite3_bind_double(statement, index, double)
            case .text(let string):
                result = sqlite3_bind_text(statement, index, string, -1, SQLITE_TRANSIENT)
            }
            guard result == SQLITE_OK else {
                throw SQLiteDatabaseError.bindFailed(lastErrorMessage)
            }
        }
    }

    private func columnValue(_ statement: OpaquePointer?, index: Int32) -> SQLiteValue {
        switch sqlite3_column_type(statement, index) {
        case SQLITE_INTEGER:
            return .int(sqlite3_column_int64(statement, index))
        case SQLITE_FLOAT:
            return .real(sqlite3_column_double(statement, index))
        case SQLITE_TEXT:
            guard let text = sqlite3_column_text(statement, index) else { return .text("") }
            return .text(String(cString: text))
        default:
            return .null
        }
    }

    private var lastErrorMessage: String {
        guard let db, let message = sqlite3_errmsg(db) else { return "unknown SQLite error" }
        return String(cString: message)
    }

    private static func applicationSupportDirectory() throws -> URL {
        let base = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
        let directory = base.appendingPathComponent("SalonMobile", isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        return directory
    }
}

private let SQLITE_TRANSIENT = unsafeBitCast(-1, to: sqlite3_destructor_type.self)
