# Section 59: Trait Objects and Dynamic Dispatch

## Learning Objectives

By the end of this section, you will:
- Understand trait objects and dyn Trait
- Track Box<dyn Trait> allocations
- Handle dynamic dispatch
- Recognize vtable implications
- Track trait object lifetimes

## Prerequisites

- Completed Section 58 (Async Fundamentals)
- Understanding of traits and generics
- Familiarity with static vs dynamic dispatch

---

## Trait Objects

```rust
trait Animal {
    fn speak(&self);
}

struct Dog;
impl Animal for Dog {
    fn speak(&self) { println!("Woof!"); }
}

// Trait object
let animal: Box<dyn Animal> = Box::new(Dog);
animal.speak();  // Dynamic dispatch
```

**Key insight:** `dyn Trait` is a trait object, enabling runtime polymorphism.

---

## Static vs Dynamic Dispatch

### Static Dispatch (Monomorphization)

```rust
fn print_animal<T: Animal>(animal: &T) {
    animal.speak();  // Compiler knows exact type
}
```

**Compiled to:**
```rust
fn print_animal_dog(animal: &Dog) { animal.speak(); }
fn print_animal_cat(animal: &Cat) { animal.speak(); }
```

### Dynamic Dispatch (Trait Objects)

```rust
fn print_animal(animal: &dyn Animal) {
    animal.speak();  // Lookup in vtable at runtime
}
```

**Runtime:** Function pointer looked up in vtable.

---

## Tracking Strategy

Trait objects are typically used with:
- `Box<dyn Trait>` - Owned trait object
- `&dyn Trait` - Borrowed trait object
- `Rc<dyn Trait>` - Shared trait object

**Approach:** Track the container (Box, Rc, etc.), not the trait itself.

---

## Detection

```rust
impl OwnershipVisitor {
    fn is_trait_object(&self, ty: &Type) -> bool {
        if let Type::TraitObject(_) = ty {
            return true;
        }
        
        // Check for Box<dyn Trait>
        if let Type::Path(type_path) = ty {
            let path_str = quote!(#type_path).to_string();
            if path_str.contains("dyn") {
                return true;
            }
        }
        
        false
    }
}
```

---

## Transformation

### Box<dyn Trait>

**Input:**
```rust
let animal: Box<dyn Animal> = Box::new(Dog);
```

**Output:**
```rust
let animal: Box<dyn Animal> = track_new(
    1,
    "animal",
    "Box<dyn Animal>",
    "line:1:9",
    Box::new(Dog)
);
```

**Note:** Track like any Box allocation.

### Reference to Trait Object

**Input:**
```rust
fn process(animal: &dyn Animal) {
    animal.speak();
}

let dog = Dog;
process(&dog);
```

**Output:**
```rust
fn process(animal: &dyn Animal) {
    animal.speak();
}

let dog = track_new(1, "dog", "Dog", "line:6:9", Dog);
process(track_borrow(2, 1, false, "line:7:9", &dog));
```

---

## Implementation

```rust
impl OwnershipVisitor {
    fn transform_local(&mut self, local: &mut Local) {
        if let Some(init) = &mut local.init {
            let id = self.next_id();
            let var_name = self.extract_var_name(&local.pat);
            let location = self.get_location(local.pat.span());
            
            // Check if type is trait object
            let type_name = if self.is_trait_object_pattern(&local.pat) {
                self.extract_trait_object_type(&local.pat)
            } else {
                self.extract_type_name(&local.pat)
            };
            
            self.var_ids.insert(var_name.clone(), id);
            
            if let Some(current_scope) = self.scope_stack.last_mut() {
                current_scope.push(id);
            }
            
            let original_expr = &init.expr;
            
            let new_expr: Expr = syn::parse_quote! {
                borrowscope_runtime::track_new(
                    #id,
                    #var_name,
                    #type_name,
                    #location,
                    #original_expr
                )
            };
            
            *init.expr = new_expr;
        }
    }
    
    fn is_trait_object_pattern(&self, pat: &Pat) -> bool {
        if let Pat::Type(pat_type) = pat {
            return self.is_trait_object(&pat_type.ty);
        }
        false
    }
    
    fn extract_trait_object_type(&self, pat: &Pat) -> String {
        if let Pat::Type(pat_type) = pat {
            return quote!(#pat_type.ty).to_string();
        }
        "dyn Trait".to_string()
    }
}
```

---

## Testing

```rust
#[test]
fn test_trait_object() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let animal: Box<dyn Animal> = Box::new(Dog);
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    assert!(output.contains("track_new"));
    assert!(output.contains("Box < dyn Animal >"));
}
```

---

## Integration Test

```rust
trait Animal {
    fn speak(&self) -> &str;
}

struct Dog;
impl Animal for Dog {
    fn speak(&self) -> &str { "Woof!" }
}

#[test]
fn test_trait_object_tracking() {
    reset_tracker();
    
    let animal: Box<dyn Animal> = track_new(
        1,
        "animal",
        "Box<dyn Animal>",
        "test.rs:1:1",
        Box::new(Dog)
    );
    
    assert_eq!(animal.speak(), "Woof!");
    
    track_drop(1, "scope_end");
    
    let events = get_events();
    assert_eq!(events.len(), 2);
}
```

---

## Vtable Considerations

**What happens at runtime:**

```rust
let animal: Box<dyn Animal> = Box::new(Dog);
```

**Memory layout:**
```
animal (fat pointer)
├── data pointer ──> Dog instance on heap
└── vtable pointer ──> Animal vtable
                       ├── speak function pointer
                       ├── drop function pointer
                       └── size/alignment info
```

**Tracking:** We track the Box, not the vtable.

---

## Lifetime Bounds

```rust
trait Animal: 'static {
    fn speak(&self);
}

// Trait object with lifetime
let animal: Box<dyn Animal + 'static> = Box::new(Dog);
```

**Tracking:** Same as regular trait objects.

---

## Key Takeaways

✅ **Trait objects enable polymorphism** - Runtime dispatch via vtable  
✅ **Track the container** - Box, Rc, etc., not the trait  
✅ **Fat pointers** - Data + vtable pointer  
✅ **Lifetime bounds** - Can have explicit lifetimes  
✅ **Same tracking approach** - No special runtime support needed  

---

## Further Reading

- [Trait objects](https://doc.rust-lang.org/book/ch17-02-trait-objects.html)
- [Dynamic dispatch](https://doc.rust-lang.org/reference/items/traits.html#trait-objects)
- [Object safety](https://doc.rust-lang.org/reference/items/traits.html#object-safety)

---

**Previous:** [58-async-rust-fundamentals.md](./58-async-rust-fundamentals.md)  
**Next:** [60-const-and-static-variables.md](./60-const-and-static-variables.md)

**Progress:** 9/15 ⬛⬛⬛⬛⬛⬛⬛⬛⬛⬜⬜⬜⬜⬜⬜
