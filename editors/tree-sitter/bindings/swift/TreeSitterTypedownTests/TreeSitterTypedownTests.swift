import XCTest
import SwiftTreeSitter
import TreeSitterTypedown

final class TreeSitterTypedownTests: XCTestCase {
    func testCanLoadGrammar() throws {
        let parser = Parser()
        let language = Language(language: tree_sitter_typedown())
        XCTAssertNoThrow(try parser.setLanguage(language),
                         "Error loading Typedown grammar")
    }
}
