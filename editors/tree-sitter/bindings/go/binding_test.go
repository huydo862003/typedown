package tree_sitter_typedown_test

import (
	"testing"

	tree_sitter "github.com/tree-sitter/go-tree-sitter"
	tree_sitter_typedown "github.com/huydo862003/typedown/tree/main/editors/tree-sitter/bindings/go"
)

func TestCanLoadGrammar(t *testing.T) {
	language := tree_sitter.NewLanguage(tree_sitter_typedown.Language())
	if language == nil {
		t.Errorf("Error loading Typedown grammar")
	}
}
