pub mod parent_folder {
    pub mod folder1 {
        pub mod mymodule;
    }
}

use parent_folder::folder1::mymodule;

fn main() {
    println!("Hello, world!");
    mymodule::function();
}
