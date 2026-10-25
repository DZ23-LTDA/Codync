import Foundation
import Testing
@testable import CodyncUI

@Test func callKitStaysOffWhereTheAppStoreIsMainlandChinas() {
    #expect(!SystemCallPolicy.allowed(storefront: "CHN", region: "US"))
    #expect(SystemCallPolicy.allowed(storefront: "TWN", region: "CN"))
    // Without a storefront (no App Store account), the region decides.
    #expect(!SystemCallPolicy.allowed(storefront: nil, region: "CN"))
    #expect(SystemCallPolicy.allowed(storefront: nil, region: "HK"))
    #expect(SystemCallPolicy.allowed(storefront: nil, region: nil))
}

@MainActor @Test func theLiveActivityFollowsTheCallsPhase() {
    #expect(CallActivity.state(.starting) == "connecting")
    #expect(CallActivity.state(.reconnecting) == "connecting")
    #expect(CallActivity.state(.listening) == "listening")
    #expect(CallActivity.state(.speaking) == "speaking")
    #expect(CallActivity.state(.interrupted) == "paused")
    #expect(CallActivity.state(.failed("x", .connection)) == nil)
}
