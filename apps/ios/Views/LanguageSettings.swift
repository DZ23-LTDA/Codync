import SwiftUI
import CodyncKit
import CodyncUI

struct LanguageSettings: View {
    @AppStorage(AppLanguage.storageKey, store: AppLanguage.defaults) private var appLanguageRaw = AppLanguage.system.rawValue

    var body: some View {
        CardSection("Language", footer: languageFooter) {
            ValueRow("App language") {
                ChoicePicker(selection: appLanguage, options: AppLanguage.allCases.map {
                    ($0, AppLanguage.menuTitle($0, preference: appLanguage.wrappedValue, systemIsChinese: AppLanguage.systemIsChinese()))
                })
            }
        }
    }

    /// Follow system names the script the iPhone is using, so Traditional is a choice of its own.
    private var languageFooter: String {
        switch AppLanguage.systemScript() {
        case .hant:
            "System language is Traditional Chinese. \"Follow system\" matches the iPhone. You can switch this app to Simplified Chinese or English."
        case .hans:
            "System language is Simplified Chinese. \"Follow system\" matches the iPhone. You can switch this app to Traditional Chinese or English."
        case nil:
            "System language is English. \"Follow system\" matches the iPhone. You can switch this app to Simplified or Traditional Chinese."
        }
    }

    private var appLanguage: Binding<AppLanguage> {
        Binding(
            get: { AppLanguage(rawValue: appLanguageRaw) ?? .system },
            set: { appLanguageRaw = $0.rawValue }
        )
    }

}
