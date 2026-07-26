// UI tests for the toggle demo: taps flow through the Rust logic
// (toggle_toggle) and the if/else branches swap in the live app.

import XCTest

final class ToggleUITests: XCTestCase {
    @MainActor
    func testHelpButtonFlipsTheIfElseBranches() {
        let app = XCUIApplication()
        app.launch()

        let hint = app.staticTexts["Tap Help again to hide this hint."]
        let placeholder = app.staticTexts["(hint hidden)"]

        // showHint starts false: only the else branch is visible.
        XCTAssertTrue(placeholder.waitForExistence(timeout: 5))
        XCTAssertFalse(hint.exists)

        // Tap → Rust flips the Bool → then branch shows, else branch hides.
        app.buttons["Help"].tap()
        XCTAssertTrue(hint.waitForExistence(timeout: 3))
        XCTAssertFalse(placeholder.exists)

        // Tap again → back to the else branch.
        app.buttons["Help"].tap()
        XCTAssertTrue(placeholder.waitForExistence(timeout: 3))
        XCTAssertFalse(hint.exists)
    }
}
