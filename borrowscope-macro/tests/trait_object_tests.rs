//! Tests for trait object support
//!
//! These tests verify that the #[trace_borrow] macro works correctly
//! with trait objects (dyn Trait) in various containers.

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

trait Animal {
    fn speak(&self) -> &'static str;
    fn name(&self) -> &'static str;
}

struct Dog {
    #[allow(dead_code)]
    breed: String,
}

impl Animal for Dog {
    fn speak(&self) -> &'static str {
        "Woof!"
    }
    fn name(&self) -> &'static str {
        "Dog"
    }
}

struct Cat {
    #[allow(dead_code)]
    color: String,
}

impl Animal for Cat {
    fn speak(&self) -> &'static str {
        "Meow!"
    }
    fn name(&self) -> &'static str {
        "Cat"
    }
}

#[test]
fn test_box_dyn_trait() {
    reset();

    #[trace_borrow]
    fn create_animal() -> Box<dyn Animal> {
        let animal: Box<dyn Animal> = Box::new(Dog {
            breed: String::from("Labrador"),
        });
        animal
    }

    let animal = create_animal();
    assert_eq!(animal.speak(), "Woof!");
    assert_eq!(animal.name(), "Dog");

    let events = get_events();
    assert!(!events.is_empty(), "Should track Box<dyn Trait>");
}

#[test]
fn test_box_dyn_trait_multiple_types() {
    reset();

    #[trace_borrow]
    fn create_dog() -> Box<dyn Animal> {
        let dog: Box<dyn Animal> = Box::new(Dog {
            breed: String::from("Beagle"),
        });
        dog
    }

    #[trace_borrow]
    fn create_cat() -> Box<dyn Animal> {
        let cat: Box<dyn Animal> = Box::new(Cat {
            color: String::from("Orange"),
        });
        cat
    }

    let dog = create_dog();
    let cat = create_cat();

    assert_eq!(dog.speak(), "Woof!");
    assert_eq!(cat.speak(), "Meow!");

    let events = get_events();
    assert!(events.len() >= 2);
}

#[test]
fn test_ref_dyn_trait() {
    reset();

    #[trace_borrow]
    fn use_animal(animal: &dyn Animal) -> &'static str {
        let sound = animal.speak();
        sound
    }

    let dog = Dog {
        breed: String::from("Poodle"),
    };
    let sound = use_animal(&dog);

    assert_eq!(sound, "Woof!");

    let events = get_events();
    assert!(!events.is_empty());
}

#[test]
fn test_vec_of_trait_objects() {
    reset();

    #[trace_borrow]
    fn create_zoo() -> Vec<Box<dyn Animal>> {
        let mut animals: Vec<Box<dyn Animal>> = Vec::new();
        animals.push(Box::new(Dog {
            breed: String::from("Husky"),
        }));
        animals.push(Box::new(Cat {
            color: String::from("Black"),
        }));
        animals
    }

    let zoo = create_zoo();
    assert_eq!(zoo.len(), 2);
    assert_eq!(zoo[0].speak(), "Woof!");
    assert_eq!(zoo[1].speak(), "Meow!");

    let events = get_events();
    assert!(!events.is_empty());
}

#[test]
fn test_rc_dyn_trait() {
    reset();

    use std::rc::Rc;

    #[trace_borrow]
    fn create_shared_animal() -> Rc<dyn Animal> {
        let animal: Rc<dyn Animal> = Rc::new(Dog {
            breed: String::from("Bulldog"),
        });
        animal
    }

    let animal1 = create_shared_animal();
    let animal2 = Rc::clone(&animal1);

    assert_eq!(animal1.speak(), "Woof!");
    assert_eq!(animal2.speak(), "Woof!");

    let events = get_events();
    assert!(!events.is_empty());
}

#[test]
fn test_arc_dyn_trait() {
    reset();

    use std::sync::Arc;

    #[trace_borrow]
    fn create_thread_safe_animal() -> Arc<dyn Animal + Send + Sync> {
        let animal: Arc<dyn Animal + Send + Sync> = Arc::new(Dog {
            breed: String::from("Shepherd"),
        });
        animal
    }

    let animal = create_thread_safe_animal();
    assert_eq!(animal.speak(), "Woof!");

    let events = get_events();
    assert!(!events.is_empty());
}

#[test]
fn test_trait_object_method_calls() {
    reset();

    #[trace_borrow]
    fn test_methods() -> (String, String) {
        let animal: Box<dyn Animal> = Box::new(Dog {
            breed: String::from("Terrier"),
        });
        let sound = animal.speak().to_string();
        let name = animal.name().to_string();
        (sound, name)
    }

    let (sound, name) = test_methods();
    assert_eq!(sound, "Woof!");
    assert_eq!(name, "Dog");

    let events = get_events();
    assert!(!events.is_empty());
}

#[test]
fn test_trait_object_in_struct() {
    reset();

    struct Zoo {
        animals: Vec<Box<dyn Animal>>,
    }

    #[trace_borrow]
    fn create_zoo_struct() -> Zoo {
        let mut animals: Vec<Box<dyn Animal>> = Vec::new();
        animals.push(Box::new(Dog {
            breed: String::from("Retriever"),
        }));
        animals.push(Box::new(Cat {
            color: String::from("White"),
        }));
        let zoo = Zoo { animals };
        zoo
    }

    let zoo = create_zoo_struct();
    assert_eq!(zoo.animals.len(), 2);

    let events = get_events();
    assert!(!events.is_empty());
}

#[test]
fn test_trait_object_polymorphism() {
    reset();

    #[trace_borrow]
    fn make_animals_speak() -> Vec<String> {
        let animals: Vec<Box<dyn Animal>> = vec![
            Box::new(Dog {
                breed: String::from("Collie"),
            }),
            Box::new(Cat {
                color: String::from("Gray"),
            }),
            Box::new(Dog {
                breed: String::from("Spaniel"),
            }),
        ];

        let mut sounds = Vec::new();
        for animal in &animals {
            sounds.push(animal.speak().to_string());
        }
        sounds
    }

    let sounds = make_animals_speak();
    assert_eq!(sounds, vec!["Woof!", "Meow!", "Woof!"]);

    let events = get_events();
    assert!(!events.is_empty());
}

#[test]
fn test_trait_object_with_lifetime() {
    reset();

    trait Greeter {
        fn greet(&self) -> String;
    }

    struct Person {
        name: String,
    }

    impl Greeter for Person {
        fn greet(&self) -> String {
            format!("Hello, I'm {}", self.name)
        }
    }

    #[trace_borrow]
    fn create_greeter() -> Box<dyn Greeter> {
        let greeter: Box<dyn Greeter> = Box::new(Person {
            name: String::from("Alice"),
        });
        greeter
    }

    let greeter = create_greeter();
    assert_eq!(greeter.greet(), "Hello, I'm Alice");

    let events = get_events();
    assert!(!events.is_empty());
}

#[test]
fn test_trait_object_downcast() {
    reset();

    use std::any::Any;

    trait AsAny: Any {
        fn as_any(&self) -> &dyn Any;
    }

    impl AsAny for Dog {
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[trace_borrow]
    fn create_and_check() -> bool {
        let animal: Box<dyn AsAny> = Box::new(Dog {
            breed: String::from("Boxer"),
        });
        let is_dog = animal.as_any().is::<Dog>();
        is_dog
    }

    let is_dog = create_and_check();
    assert!(is_dog);

    let events = get_events();
    assert!(!events.is_empty());
}

#[test]
fn test_trait_object_with_generic_method() {
    reset();

    trait Container {
        fn get_value(&self) -> i32;
    }

    struct IntBox {
        value: i32,
    }

    impl Container for IntBox {
        fn get_value(&self) -> i32 {
            self.value
        }
    }

    #[trace_borrow]
    fn use_container() -> i32 {
        let container: Box<dyn Container> = Box::new(IntBox { value: 42 });
        let value = container.get_value();
        value
    }

    let value = use_container();
    assert_eq!(value, 42);

    let events = get_events();
    assert!(!events.is_empty());
}

#[test]
fn test_trait_object_clone() {
    reset();

    trait CloneableAnimal: Animal {
        fn clone_box(&self) -> Box<dyn CloneableAnimal>;
    }

    impl CloneableAnimal for Dog {
        fn clone_box(&self) -> Box<dyn CloneableAnimal> {
            Box::new(Dog {
                breed: self.breed.clone(),
            })
        }
    }

    #[trace_borrow]
    fn clone_animal() -> (Box<dyn CloneableAnimal>, Box<dyn CloneableAnimal>) {
        let animal1: Box<dyn CloneableAnimal> = Box::new(Dog {
            breed: String::from("Dalmatian"),
        });
        let animal2 = animal1.clone_box();
        (animal1, animal2)
    }

    let (animal1, animal2) = clone_animal();
    assert_eq!(animal1.speak(), "Woof!");
    assert_eq!(animal2.speak(), "Woof!");

    let events = get_events();
    assert!(!events.is_empty());
}

#[test]
fn test_trait_object_option() {
    reset();

    #[trace_borrow]
    fn maybe_animal(create: bool) -> Option<Box<dyn Animal>> {
        if create {
            let animal: Box<dyn Animal> = Box::new(Dog {
                breed: String::from("Pug"),
            });
            Some(animal)
        } else {
            None
        }
    }

    let some_animal = maybe_animal(true);
    assert!(some_animal.is_some());
    assert_eq!(some_animal.unwrap().speak(), "Woof!");

    let no_animal = maybe_animal(false);
    assert!(no_animal.is_none());

    let events = get_events();
    assert!(!events.is_empty());
}

#[test]
fn test_trait_object_result() {
    reset();

    #[trace_borrow]
    fn try_create_animal(succeed: bool) -> std::result::Result<Box<dyn Animal>, String> {
        if succeed {
            let animal: Box<dyn Animal> = Box::new(Dog {
                breed: String::from("Corgi"),
            });
            Ok(animal)
        } else {
            Err("Failed to create animal".to_string())
        }
    }

    let result_ok = try_create_animal(true);
    assert!(result_ok.is_ok());
    assert_eq!(result_ok.unwrap().speak(), "Woof!");

    let result_err = try_create_animal(false);
    assert!(result_err.is_err());

    let events = get_events();
    assert!(!events.is_empty());
}
