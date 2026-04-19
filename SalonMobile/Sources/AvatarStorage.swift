import Foundation
import UIKit

enum AvatarStorage {
    private static let filename = "current-user-avatar.jpg"
    private static let localPrefix = "local:"

    static func saveUserAvatar(_ data: Data) throws -> String {
        let directory = try avatarDirectory()
        let url = directory.appendingPathComponent(filename)
        let normalized = normalizedJPEGData(from: data) ?? data
        try normalized.write(to: url, options: [.atomic])
        return localPrefix + filename
    }

    static func deleteUserAvatar() throws {
        let url = try avatarDirectory().appendingPathComponent(filename)
        if FileManager.default.fileExists(atPath: url.path) {
            try FileManager.default.removeItem(at: url)
        }
    }

    static func image(for avatarSeed: String?) -> UIImage? {
        guard let avatarSeed = avatarSeed?.trimmedOrNil() else { return nil }

        if avatarSeed.hasPrefix(localPrefix) {
            let filename = String(avatarSeed.dropFirst(localPrefix.count))
            guard let url = try? avatarDirectory().appendingPathComponent(filename) else { return nil }
            return UIImage(contentsOfFile: url.path)
        }

        if let url = Bundle.main.url(forResource: avatarSeed, withExtension: "jpg") {
            return UIImage(contentsOfFile: url.path)
        }

        if let url = Bundle.main.url(
            forResource: avatarSeed,
            withExtension: "jpg",
            subdirectory: "AgentAvatars"
        ) {
            return UIImage(contentsOfFile: url.path)
        }

        return nil
    }

    static func defaultSeed(for handle: String) -> String? {
        switch handle.lowercased() {
        case "jasmine": return "agent-1"
        case "marc": return "agent-2"
        case "angel": return "agent-3"
        case "mike": return "agent-4"
        case "jasper": return "agent-5"
        case "alex": return "agent-6"
        default: return nil
        }
    }

    private static func avatarDirectory() throws -> URL {
        let base = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
        let directory = base.appendingPathComponent("SalonMobile/Avatars", isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        return directory
    }

    private static func normalizedJPEGData(from data: Data) -> Data? {
        guard let image = UIImage(data: data) else { return nil }
        let targetSize = CGSize(width: 512, height: 512)
        let renderer = UIGraphicsImageRenderer(size: targetSize)
        let rendered = renderer.image { _ in
            let scale = max(targetSize.width / image.size.width, targetSize.height / image.size.height)
            let size = CGSize(width: image.size.width * scale, height: image.size.height * scale)
            let origin = CGPoint(
                x: (targetSize.width - size.width) / 2,
                y: (targetSize.height - size.height) / 2
            )
            image.draw(in: CGRect(origin: origin, size: size))
        }
        return rendered.jpegData(compressionQuality: 0.88)
    }
}
