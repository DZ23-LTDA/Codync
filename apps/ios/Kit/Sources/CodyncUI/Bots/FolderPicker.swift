import CodyncKit
import SwiftUI

/// Browses folders on the host, drilling down in place.
struct FolderPicker: View {
    let start: String?
    let onPick: (String) -> Void
    /// The folders opened so far; the last one is shown.
    @State private var stack: [String?]
    @State private var forward = true
    /// Where the first folder sits, so Back can climb above the starting folder ("" lists the drives).
    @State private var parent: String?
    @Environment(\.accessibilityReduceMotion) private var reduceMotion

    init(start: String?, onPick: @escaping (String) -> Void) {
        self.start = start
        self.onPick = onPick
        _stack = State(initialValue: [start])
    }

    var body: some View {
        let current = stack.last ?? nil
        VStack(spacing: 0) {
            HStack(spacing: 0) {
                if stack.count > 1 || parent != nil {
                    BackButton { go(forward: false) { stack = stack.count > 1 ? Array(stack.dropLast()) : [parent] } }
                        .padding(.leading, 12)
                        .transition(.opacity)
                }
                ModalHeader(current.flatMap(folderName) ?? "Folders")
            }
            ZStack {
                FolderLevel(path: current, onPick: onPick, onParent: stack.count > 1 ? nil : { parent = $0 }) { path in
                    go(forward: true) { stack.append(path) }
                }
                .id("\(stack.count):\(current ?? "")")
                .transition(.asymmetric(
                    insertion: .move(edge: forward ? .trailing : .leading),
                    removal: .move(edge: forward ? .leading : .trailing)
                ).combined(with: .opacity))
            }
            .frame(maxHeight: .infinity, alignment: .top)
            .clipped()
        }
        .background(Palette.background)
    }

    /// Sets the slide direction first, so the leaving level takes it too, then moves.
    private func go(forward: Bool, _ change: @escaping () -> Void) {
        self.forward = forward
        Task { @MainActor in
            withAnimation(Motion.reduced(Motion.layout, reduceMotion), change)
        }
    }
}

/// One folder's subfolders.
private struct FolderLevel: View {
    let path: String?
    let onPick: (String) -> Void
    /// Hears where this folder sits; only the first level reports it.
    let onParent: ((String?) -> Void)?
    let open: (String) -> Void
    @Environment(BotStore.self) private var model
    @State private var listing: DirListing?
    @State private var error: String?
    @State private var filter = ""

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 22) {
                SearchField(text: $filter)
                if let listing {
                    // The drive list (Windows) has no folder of its own to use.
                    if !listing.path.isEmpty {
                        CardSection(footer: listing.path) {
                            Button {
                                onPick(listing.path)
                            } label: {
                                Label("Use “\(folderName(listing.path) ?? listing.path)”", systemImage: "checkmark.circle.fill")
                                    .font(.headline)
                                    .foregroundStyle(Palette.accent)
                                    .frame(maxWidth: .infinity, alignment: .leading)
                                    .contentShape(Rectangle())
                            }
                            .buttonStyle(PressScale())
                        }
                    }
                    let dirs = listing.dirs.filter { filter.isEmpty || $0.name.localizedCaseInsensitiveContains(filter) }
                    if !dirs.isEmpty {
                        CardSection {
                            ForEach(dirs) { dir in
                                Button {
                                    open(dir.path)
                                } label: {
                                    HStack(spacing: 10) {
                                        Image(systemName: dir.isGit ? "arrow.triangle.branch" : "folder")
                                            .foregroundStyle(dir.isGit ? Palette.accent : Palette.secondary)
                                            .frame(width: 20)
                                        Text(dir.name).foregroundStyle(Palette.text).lineLimit(1)
                                        Spacer(minLength: 8)
                                        Image(systemName: "chevron.right")
                                            .font(.caption2.weight(.semibold))
                                            .foregroundStyle(Palette.tertiary)
                                    }
                                    .contentShape(Rectangle())
                                }
                                .buttonStyle(PressScale())
                            }
                        }
                    }
                } else if let error {
                    Text(L10n.tr(error)).foregroundStyle(Palette.danger)
                } else {
                    Spinner(size: 20).frame(maxWidth: .infinity)
                }
            }
            .font(.body)
            .padding(20)
        }
        .scrollDismissesKeyboard(.interactively)
        .background(Palette.background)
        .task {
            do {
                let listing = try await model.listDirs(path)
                self.listing = listing
                onParent?(listing.parent)
            } catch { self.error = error.localizedDescription }
        }
    }
}

/// A path's last folder, for `/` and Windows `\` paths alike.
private func folderName(_ path: String) -> String? {
    path.split(whereSeparator: { $0 == "/" || $0 == "\\" }).last.map(String.init)
}
