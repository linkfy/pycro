# readmouse

A very small library for reading the mouse location and mouse button presses on macOS.

# Example usage

Continuously print the mouse location:

```rust
use readmouse::Mouse;

fn main() {
    loop {
        println!(
            "L={:?} R={:?} C={:?} (x,y)={:?}",
            Mouse::Left.is_pressed(),
            Mouse::Right.is_pressed(),
            Mouse::Center.is_pressed(),
            Mouse::location()
        );
    }
}
```

# Related

[readkey](https://crates.io/crates/readkey) - Find out if a key on the keyboard is currently pressed on macOS.
