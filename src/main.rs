use std::rc::Rc;

#[derive(Debug)]
struct Animal {
    name: String,
}

struct Zoo {
    animals: Vec<Rc<Animal>>,
}

impl Zoo {
    fn add_animal(&mut self, animal: Animal) {
        self.animals.push(Rc::new(animal));
    }

    fn get_animal(&self, name: &str) -> Rc<Animal> {
        self.animals.iter().find(|a| a.name == name).unwrap().clone()
    }
}

fn main() {
    let mut zoo = Zoo {
        animals: Vec::new(),
    };

    let lion = Animal { name: "Lion".to_owned(), };

    zoo.add_animal(lion);

    let lion = zoo.get_animal("Lion");
    println!("lion = {:?} rc = {}", lion, Rc::strong_count(&lion));

    {
        let lion2 = zoo.get_animal("Lion");
        println!("lion = {:?} rc = {}", lion2, Rc::strong_count(&lion));
    }

    println!("lion = {:?} rc = {}", lion, Rc::strong_count(&lion));
}