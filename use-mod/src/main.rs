mod parent_folder;

fn main() {
    println!("Hello, world!");
    parent_folder::folder1::mymodule::function();

    let (a, b) = math::numbers::get_two_numbers();
    println!("{a:?} + {b:?} = {:2}", math::operations::add(a, b));
    println!("{a:?} - {b:?} = {:2}", math::operations::subtract(a, b));
    println!("{a:?} * {b:?} = {:2}", math::operations::multiply(a, b));
    println!("{a:?} / {b:?} = {:2}", math::operations::divide(a, b));
}
