use mylib::{add, greet, math};

fn main() {
    println!("{}", greet("User"));
    println!("2 + 3 = {}", add(2, 3));
    println!("4 * 5 = {}", math::multiply(4, 5));
    println!("10 / 2 = {:?}", math::divide(10, 2));
    println!("10 / 0 = {:?}", math::divide(10, 0));
}
