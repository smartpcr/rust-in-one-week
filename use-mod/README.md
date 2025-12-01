# use-mod

Demonstrates Rust module organization patterns:

- Nested module paths (`parent_folder::folder1::mymodule`)
- Local path dependencies via Cargo (`math` subcrate)
- Re-exporting modules through `lib.rs`

The `math` subcrate provides:
- `numbers::get_two_random_numbers()` - generates random f64 values
- `operations` - basic arithmetic functions (add, subtract, multiply, divide, gcd)

## Run

```bash
cargo run -p use-mod
```
