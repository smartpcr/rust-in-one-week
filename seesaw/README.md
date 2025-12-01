# seesaw

Solves the classic "find the different weight" puzzle using a divide-and-conquer algorithm.

Given N people where one has a different weight, finds that person using minimal comparisons (weighings on a seesaw/balance scale).

Features:
- Custom traits (`Comparable`, `Equatable`) for comparison logic
- Recursive group splitting algorithm
- Random weight generation using `rand`
- Step-by-step solution output

## Architecture

- `seesaw/person.rs` - Person struct with weight comparison
- `seesaw/group.rs` - Group of people with split logic
- `seesaw/traits.rs` - Custom comparison traits and helper functions

## Run

```bash
cargo run -p seesaw
```
