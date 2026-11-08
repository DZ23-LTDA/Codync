import UIKit

/// The mascot art the host assigns a bot (`host/src/agent/avatar.rs`).
///
/// The pictures are derived into `Sources/CodyncKit/Resources/Mascots` by `tools/mascot-assets.mjs --derive` and
/// declared as a copied resource in `Package.swift`, so they arrive in the bundle as files under `Mascots/` — not in
/// an asset catalog, which is why they are loaded by their file name here.
public enum MascotArt {
    /// Decoded once per id: an avatar is redrawn on every roster pass, and decoding a 512 px PNG there is not free.
    private static let cache = NSCache<NSString, UIImage>()

    /// The picture for a mascot id, or `nil` when this build ships none for it — the caller draws the procedural
    /// avatar instead and a bot still has a face.
    public static func image(_ id: String) -> UIImage? {
        guard !id.isEmpty else { return nil }
        if let hit = cache.object(forKey: id as NSString) { return hit }
        guard let url = Bundle.module.url(forResource: id, withExtension: "png", subdirectory: "Mascots"),
              let image = UIImage(contentsOfFile: url.path) else { return nil }
        cache.setObject(image, forKey: id as NSString)
        return image
    }
}
