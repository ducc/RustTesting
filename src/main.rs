use std::ops::Deref;

struct Dog {
    name: String,
    age: u8,
    owner: String,
}

impl Deref for Dog {
    type Target = String;

    fn deref(&self) -> &String {
        &self.name
    }
}

fn main() {
    let dog = Dog {
        name: "Billy".to_owned(),
        age: 4,
        owner: "Jacob".to_owned(),
    };

    println!("dog name: {}", *dog);
}