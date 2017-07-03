trait Animal {
    fn meme(&self);
}

trait AnimalFactory<T: Animal> {
    fn new() -> T;
}

struct Dog {

}

impl Animal for Dog {
    fn meme(&self) {
        println!("dogs like memes!");
    }
}

impl AnimalFactory<Dog> for Dog {
    fn new() -> Dog {
        Dog{}
    }
}

struct Cat {

}

impl Animal for Cat {
    fn meme(&self) {
        println!("cats like memes!");
    }
}

impl AnimalFactory<Cat> for Cat {
    fn new() -> Cat {
        Cat{}
    }
}

fn run<T: Animal>(animal: T) {
    animal.meme();
}

fn main() {
    let dog = Dog::new();
    run(dog);

    let cat = Cat::new();
    run(cat);
}
