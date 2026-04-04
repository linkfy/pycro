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
