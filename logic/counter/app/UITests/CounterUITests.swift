// End-to-end proof of the whole nui loop: tap (SwiftUI) → event → Rust
// counter_handle via UniFFI → new state → re-render.

import XCTest

final class CounterUITests: XCTestCase {
    func testCountLivesInRust() {
        let app = XCUIApplication()
        app.launch()

        XCTAssertTrue(app.staticTexts["Count: 0"].waitForExistence(timeout: 5))

        app.buttons["+"].tap()
        app.buttons["+"].tap()
        XCTAssertTrue(app.staticTexts["Count: 2"].waitForExistence(timeout: 3))

        app.buttons["-"].tap()
        XCTAssertTrue(app.staticTexts["Count: 1"].waitForExistence(timeout: 3))
    }
}
