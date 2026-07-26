// UI tests for the profile demo: a whole record crosses the FFI boundary
// (Person → ProfilePerson and back) and the dotted-path labels update.

import XCTest

final class ProfileUITests: XCTestCase {
    @MainActor
    func testNextButtonCyclesPeopleThroughRust() {
        let app = XCUIApplication()
        app.launch()

        // Initial state comes from the .nui record literal.
        XCTAssertTrue(app.staticTexts["Ada Lovelace"].waitForExistence(timeout: 5))

        // Each tap sends the current Person to Rust and shows the returned one.
        app.buttons["Next"].tap()
        XCTAssertTrue(app.staticTexts["Grace Hopper"].waitForExistence(timeout: 3))
        XCTAssertTrue(
            app.staticTexts["Built the first compiler and coined the bug."].exists
        )

        app.buttons["Next"].tap()
        XCTAssertTrue(app.staticTexts["Alan Turing"].waitForExistence(timeout: 3))

        // The cycle wraps back to the start.
        app.buttons["Next"].tap()
        XCTAssertTrue(app.staticTexts["Ada Lovelace"].waitForExistence(timeout: 3))
    }
}
