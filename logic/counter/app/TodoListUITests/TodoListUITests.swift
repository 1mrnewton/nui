// UI test for the todo-list demo: taps flow UI → store → Rust (whole
// Vec<TodoListTodo> across the FFI, both directions) → rows rebuilt.

import XCTest

final class TodoListUITests: XCTestCase {
    @MainActor
    func testAddAndClearRebuildRowsThroughRust() {
        let app = XCUIApplication()
        app.launch()

        // The seeded list literal from the .nui file.
        XCTAssertTrue(app.staticTexts["Learn nui"].waitForExistence(timeout: 5))

        // Each tap sends the whole list to Rust and renders what comes back.
        app.buttons["Add"].tap()
        XCTAssertTrue(app.staticTexts["Todo #2"].waitForExistence(timeout: 3))
        app.buttons["Add"].tap()
        XCTAssertTrue(app.staticTexts["Todo #3"].waitForExistence(timeout: 3))
        XCTAssertTrue(app.staticTexts["Learn nui"].exists)

        // Clear returns an empty list; every row disappears.
        app.buttons["Clear"].tap()
        XCTAssertTrue(app.staticTexts["Todo #2"].waitForNonExistence(timeout: 3))
        XCTAssertFalse(app.staticTexts["Learn nui"].exists)
        XCTAssertFalse(app.staticTexts["Todo #3"].exists)

        // Adding after a clear starts numbering from an empty list — the
        // logic is pure, a function of whatever list the UI holds.
        app.buttons["Add"].tap()
        XCTAssertTrue(app.staticTexts["Todo #1"].waitForExistence(timeout: 3))
    }
}
