# quick-replace

A CLI tool for regex-based text replacement in files.

Features:
- Regex pattern matching using the `regex` crate
- Colored terminal output using `text-colorizer`
- Command-line argument parsing

## Usage

```bash
cargo run -p quick-replace -- <target> <replacement> <filename> <output>
```

Arguments:
- `target` - regex pattern to search for
- `replacement` - replacement text
- `filename` - input file path
- `output` - output file path
