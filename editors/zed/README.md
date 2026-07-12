# Typedown for Zed

Language server integration for the Typedown language (`.tdr` files).

## Semantic Tokens

Syntax highlighting is provided via LSP semantic tokens. Zed disables semantic tokens by default, so you must enable them in your `settings.json`:

```json
{
  "semantic_tokens": "combined"
}
```

Or to enable only for Typedown:

```json
{
  "languages": {
    "Typedown": {
      "semantic_tokens": "combined"
    }
  }
}
```
