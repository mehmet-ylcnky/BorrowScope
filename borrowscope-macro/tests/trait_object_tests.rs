//! Tests for trait object support
//!
//! These tests verify that the #[trace_borrow] macro works correctly
//! with trait objects (dyn Trait) in various containers.

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;
use serial_test::serial;

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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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

// Advanced trait object patterns

#[test]
#[serial]
fn test_trait_object_with_multiple_traits() {
    reset();

    trait Speak {
        fn speak(&self) -> &'static str;
    }

    trait Walk {
        fn walk(&self) -> &'static str;
    }

    struct Robot;

    impl Speak for Robot {
        fn speak(&self) -> &'static str {
            "Beep boop"
        }
    }

    impl Walk for Robot {
        fn walk(&self) -> &'static str {
            "Rolling"
        }
    }

    #[trace_borrow]
    fn create_robot() -> (Box<dyn Speak>, Box<dyn Walk>) {
        let speaker: Box<dyn Speak> = Box::new(Robot);
        let walker: Box<dyn Walk> = Box::new(Robot);
        (speaker, walker)
    }

    let (speaker, walker) = create_robot();
    assert_eq!(speaker.speak(), "Beep boop");
    assert_eq!(walker.walk(), "Rolling");

    let events = get_events();
    assert!(events.len() >= 2);
}

#[test]
#[serial]
fn test_trait_object_with_associated_types() {
    reset();

    trait Container {
        type Item;
        fn get(&self) -> &Self::Item;
    }

    struct IntContainer {
        value: i32,
    }

    impl Container for IntContainer {
        type Item = i32;
        fn get(&self) -> &Self::Item {
            &self.value
        }
    }

    #[trace_borrow]
    fn use_container() -> i32 {
        let container = IntContainer { value: 42 };
        let value = *container.get();
        value
    }

    let result = use_container();
    assert_eq!(result, 42);

    let events = get_events();
    assert!(!events.is_empty());
}

#[test]
#[serial]
fn test_nested_trait_objects() {
    reset();

    trait Inner {
        fn inner_value(&self) -> i32;
    }

    trait Outer {
        fn get_inner(&self) -> Box<dyn Inner>;
    }

    struct InnerImpl {
        value: i32,
    }

    impl Inner for InnerImpl {
        fn inner_value(&self) -> i32 {
            self.value
        }
    }

    struct OuterImpl {
        inner: i32,
    }

    impl Outer for OuterImpl {
        fn get_inner(&self) -> Box<dyn Inner> {
            Box::new(InnerImpl { value: self.inner })
        }
    }

    #[trace_borrow]
    fn nested_access() -> i32 {
        let outer: Box<dyn Outer> = Box::new(OuterImpl { inner: 42 });
        let inner = outer.get_inner();
        let value = inner.inner_value();
        value
    }

    let result = nested_access();
    assert_eq!(result, 42);

    let events = get_events();
    assert!(!events.is_empty());
}

#[test]
#[serial]
fn test_trait_object_with_default_methods() {
    reset();

    trait Describable {
        fn name(&self) -> &str;

        fn description(&self) -> String {
            format!("This is a {}", self.name())
        }
    }

    struct Thing {
        thing_name: String,
    }

    impl Describable for Thing {
        fn name(&self) -> &str {
            &self.thing_name
        }
    }

    #[trace_borrow]
    fn use_default_method() -> String {
        let thing: Box<dyn Describable> = Box::new(Thing {
            thing_name: String::from("widget"),
        });
        let desc = thing.description();
        desc
    }

    let result = use_default_method();
    assert_eq!(result, "This is a widget");

    let events = get_events();
    assert!(!events.is_empty());
}

#[test]
#[serial]
fn test_trait_object_with_supertrait() {
    reset();

    trait Base {
        fn base_method(&self) -> i32;
    }

    trait Derived: Base {
        fn derived_method(&self) -> i32;
    }

    struct Impl;

    impl Base for Impl {
        fn base_method(&self) -> i32 {
            10
        }
    }

    impl Derived for Impl {
        fn derived_method(&self) -> i32 {
            20
        }
    }

    #[trace_borrow]
    fn use_supertrait() -> (i32, i32) {
        let obj: Box<dyn Derived> = Box::new(Impl);
        let base = obj.base_method();
        let derived = obj.derived_method();
        (base, derived)
    }

    let (base, derived) = use_supertrait();
    assert_eq!(base, 10);
    assert_eq!(derived, 20);

    let events = get_events();
    assert!(!events.is_empty());
}

#[test]
#[serial]
fn test_trait_object_in_hashmap() {
    reset();

    use std::collections::HashMap;

    trait Plugin {
        fn execute(&self) -> String;
    }

    struct PluginA;
    impl Plugin for PluginA {
        fn execute(&self) -> String {
            "Plugin A executed".to_string()
        }
    }

    struct PluginB;
    impl Plugin for PluginB {
        fn execute(&self) -> String {
            "Plugin B executed".to_string()
        }
    }

    #[trace_borrow]
    fn create_plugin_registry() -> HashMap<String, Box<dyn Plugin>> {
        let mut registry: HashMap<String, Box<dyn Plugin>> = HashMap::new();
        registry.insert("a".to_string(), Box::new(PluginA));
        registry.insert("b".to_string(), Box::new(PluginB));
        registry
    }

    let registry = create_plugin_registry();
    assert_eq!(registry.len(), 2);
    assert_eq!(registry["a"].execute(), "Plugin A executed");
    assert_eq!(registry["b"].execute(), "Plugin B executed");

    let events = get_events();
    assert!(!events.is_empty());
}

#[test]
#[serial]
fn test_trait_object_with_lifetime_bounds() {
    reset();

    trait Processor<'a> {
        fn process(&self, data: &'a str) -> String;
    }

    struct Uppercaser;

    impl<'a> Processor<'a> for Uppercaser {
        fn process(&self, data: &'a str) -> String {
            data.to_uppercase()
        }
    }

    #[trace_borrow]
    fn process_data() -> String {
        let processor = Uppercaser;
        let data = "hello";
        let result = processor.process(data);
        result
    }

    let result = process_data();
    assert_eq!(result, "HELLO");

    let events = get_events();
    assert!(!events.is_empty());
}

#[test]
#[serial]
fn test_trait_object_factory_pattern() {
    reset();

    trait Shape {
        fn area(&self) -> f64;
    }

    struct Circle {
        radius: f64,
    }

    impl Shape for Circle {
        fn area(&self) -> f64 {
            std::f64::consts::PI * self.radius * self.radius
        }
    }

    struct Rectangle {
        width: f64,
        height: f64,
    }

    impl Shape for Rectangle {
        fn area(&self) -> f64 {
            self.width * self.height
        }
    }

    #[trace_borrow]
    fn create_shape(shape_type: &str) -> Box<dyn Shape> {
        match shape_type {
            "circle" => {
                let shape: Box<dyn Shape> = Box::new(Circle { radius: 5.0 });
                shape
            }
            "rectangle" => {
                let shape: Box<dyn Shape> = Box::new(Rectangle {
                    width: 4.0,
                    height: 6.0,
                });
                shape
            }
            _ => {
                let shape: Box<dyn Shape> = Box::new(Circle { radius: 1.0 });
                shape
            }
        }
    }

    let circle = create_shape("circle");
    let rectangle = create_shape("rectangle");

    assert!(circle.area() > 78.0 && circle.area() < 79.0);
    assert_eq!(rectangle.area(), 24.0);

    let events = get_events();
    assert!(events.len() >= 2);
}

#[test]
#[serial]
fn test_trait_object_with_mutable_methods() {
    reset();

    trait Counter {
        fn value(&self) -> i32;
    }

    struct SimpleCounter {
        count: i32,
    }

    impl Counter for SimpleCounter {
        fn value(&self) -> i32 {
            self.count
        }
    }

    #[trace_borrow]
    fn use_counter() -> i32 {
        let counter: Box<dyn Counter> = Box::new(SimpleCounter { count: 3 });
        let value = counter.value();
        value
    }

    let result = use_counter();
    assert_eq!(result, 3);

    let events = get_events();
    assert!(!events.is_empty());
}

#[test]
#[serial]
fn test_trait_object_iterator() {
    reset();

    trait NumberGenerator {
        fn get_numbers(&self) -> Vec<i32>;
    }

    struct RangeGenerator {
        max: i32,
    }

    impl NumberGenerator for RangeGenerator {
        fn get_numbers(&self) -> Vec<i32> {
            (0..self.max).collect()
        }
    }

    #[trace_borrow]
    fn generate_numbers() -> Vec<i32> {
        let gen: Box<dyn NumberGenerator> = Box::new(RangeGenerator { max: 5 });
        let numbers = gen.get_numbers();
        numbers
    }

    let numbers = generate_numbers();
    assert_eq!(numbers, vec![0, 1, 2, 3, 4]);

    let events = get_events();
    assert!(!events.is_empty());
}

#[test]
#[serial]
fn test_trait_object_with_generic_trait() {
    reset();

    trait Converter<T> {
        fn convert(&self, value: T) -> String;
    }

    struct IntConverter;

    impl Converter<i32> for IntConverter {
        fn convert(&self, value: i32) -> String {
            format!("Integer: {}", value)
        }
    }

    #[trace_borrow]
    fn use_converter() -> String {
        let converter = IntConverter;
        let result = converter.convert(42);
        result
    }

    let result = use_converter();
    assert_eq!(result, "Integer: 42");

    let events = get_events();
    assert!(!events.is_empty());
}

#[test]
#[serial]
fn test_trait_object_state_machine() {
    reset();

    trait State {
        fn handle(&self) -> &'static str;
    }

    struct StateA;
    struct StateB;
    struct StateC;

    impl State for StateA {
        fn handle(&self) -> &'static str {
            "State A"
        }
    }

    impl State for StateB {
        fn handle(&self) -> &'static str {
            "State B"
        }
    }

    impl State for StateC {
        fn handle(&self) -> &'static str {
            "State C"
        }
    }

    #[trace_borrow]
    fn create_states() -> Vec<Box<dyn State>> {
        let states: Vec<Box<dyn State>> = vec![
            Box::new(StateA),
            Box::new(StateB),
            Box::new(StateC),
            Box::new(StateA),
        ];
        states
    }

    let states = create_states();
    let results: Vec<String> = states.iter().map(|s| s.handle().to_string()).collect();
    assert_eq!(results, vec!["State A", "State B", "State C", "State A"]);

    let events = get_events();
    assert!(!events.is_empty());
}

#[tokio::test]
#[serial]
async fn test_trait_object_with_async() {
    reset();

    trait AsyncTask {
        fn name(&self) -> &'static str;
    }

    struct Task1;
    impl AsyncTask for Task1 {
        fn name(&self) -> &'static str {
            "Task 1"
        }
    }

    #[trace_borrow]
    async fn create_async_task() -> Box<dyn AsyncTask> {
        let task: Box<dyn AsyncTask> = Box::new(Task1);
        task
    }

    let task = create_async_task().await;
    assert_eq!(task.name(), "Task 1");

    let events = get_events();
    assert!(!events.is_empty());
}

#[test]
#[serial]
fn test_trait_object_builder_pattern() {
    reset();

    trait Builder {
        fn build(&self) -> String;
    }

    struct ConfigBuilder {
        name: String,
        value: i32,
    }

    impl Builder for ConfigBuilder {
        fn build(&self) -> String {
            format!("{}={}", self.name, self.value)
        }
    }

    #[trace_borrow]
    fn use_builder() -> String {
        let builder: Box<dyn Builder> = Box::new(ConfigBuilder {
            name: String::from("timeout"),
            value: 30,
        });
        let config = builder.build();
        config
    }

    let result = use_builder();
    assert_eq!(result, "timeout=30");

    let events = get_events();
    assert!(!events.is_empty());
}

#[test]
#[serial]
fn test_trait_object_command_pattern() {
    reset();

    trait Command {
        fn execute(&self) -> i32;
    }

    struct AddCommand {
        a: i32,
        b: i32,
    }

    impl Command for AddCommand {
        fn execute(&self) -> i32 {
            self.a + self.b
        }
    }

    struct MultiplyCommand {
        a: i32,
        b: i32,
    }

    impl Command for MultiplyCommand {
        fn execute(&self) -> i32 {
            self.a * self.b
        }
    }

    #[trace_borrow]
    fn execute_commands() -> Vec<i32> {
        let commands: Vec<Box<dyn Command>> = vec![
            Box::new(AddCommand { a: 5, b: 3 }),
            Box::new(MultiplyCommand { a: 4, b: 7 }),
            Box::new(AddCommand { a: 10, b: 20 }),
        ];

        let mut results = Vec::new();
        for cmd in &commands {
            results.push(cmd.execute());
        }
        results
    }

    let results = execute_commands();
    assert_eq!(results, vec![8, 28, 30]);

    let events = get_events();
    assert!(!events.is_empty());
}

#[test]
#[serial]
fn test_trait_object_observer_pattern() {
    reset();

    trait Observer {
        fn notify(&self, message: &str) -> String;
    }

    struct EmailObserver;
    impl Observer for EmailObserver {
        fn notify(&self, message: &str) -> String {
            format!("Email: {}", message)
        }
    }

    struct SmsObserver;
    impl Observer for SmsObserver {
        fn notify(&self, message: &str) -> String {
            format!("SMS: {}", message)
        }
    }

    #[trace_borrow]
    fn notify_observers() -> Vec<String> {
        let observers: Vec<Box<dyn Observer>> =
            vec![Box::new(EmailObserver), Box::new(SmsObserver)];

        let mut notifications = Vec::new();
        for observer in &observers {
            notifications.push(observer.notify("Alert!"));
        }
        notifications
    }

    let notifications = notify_observers();
    assert_eq!(notifications, vec!["Email: Alert!", "SMS: Alert!"]);

    let events = get_events();
    assert!(!events.is_empty());
}

#[test]
#[serial]
fn test_trait_object_with_closure_like_behavior() {
    reset();

    trait Callable {
        fn call(&self, x: i32) -> i32;
    }

    struct Doubler;
    impl Callable for Doubler {
        fn call(&self, x: i32) -> i32 {
            x * 2
        }
    }

    struct Incrementer;
    impl Callable for Incrementer {
        fn call(&self, x: i32) -> i32 {
            x + 1
        }
    }

    #[trace_borrow]
    fn apply_functions() -> Vec<i32> {
        let functions: Vec<Box<dyn Callable>> = vec![Box::new(Doubler), Box::new(Incrementer)];

        let mut results = Vec::new();
        for func in &functions {
            results.push(func.call(10));
        }
        results
    }

    let results = apply_functions();
    assert_eq!(results, vec![20, 11]);

    let events = get_events();
    assert!(!events.is_empty());
}
