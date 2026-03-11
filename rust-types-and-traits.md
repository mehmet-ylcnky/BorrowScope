# Elements of Rust – Core Types and Traits

A reference guide to the Rust type system, covering all built-in types and traits.

Source: [rustcurious.com/elements/](https://rustcurious.com/elements/) by Ben Williamson.

---

## Types

### Scalar Types

| Type | Size | Description | Example |
|------|------|-------------|---------|
| `u8` | 8-bit | Unsigned integer | 0 … 255 |
| `i8` | 8-bit | Signed integer | -128 … 127 |
| `bool` | — | Boolean | `true`, `false` |
| `u16` | 16-bit | Unsigned integer | 0 … 65535 |
| `i16` | 16-bit | Signed integer | -32768 … 32767 |
| `f16` | 16-bit | Float (reserved) | — |
| `u32` | 32-bit | Unsigned integer | 0 … 0xffffffff |
| `i32` | 32-bit | Signed integer | Default integer type |
| `f32` | 32-bit | Float | `1.0` |
| `u64` | 64-bit | Unsigned integer | — |
| `i64` | 64-bit | Signed integer | — |
| `f64` | 64-bit | Float | `1.0` (default float) |
| `u128` | 128-bit | Unsigned integer | — |
| `i128` | 128-bit | Signed integer | — |
| `f128` | 128-bit | Float (reserved) | — |
| `usize` | Pointer-sized | Unsigned integer | `array.len()` |
| `isize` | Pointer-sized | Signed integer | — |
| `char` | 4 bytes | Unicode scalar value | `'a'`, `'🚀'` |

### Compound Types

| Type | Syntax | Description | Example |
|------|--------|-------------|---------|
| Tuple | `(T, U…)` | Fixed-size heterogeneous collection | `(0, true)` |
| Struct | `struct Foo { … }` | Named fields | `Point { x: 1, y: 2 }` |
| Enum | `enum Foo { … }` | Sum type with variants | `Some(val)` |
| Union | `union Foo { … }` | Unsafe overlapping fields | — |
| Array | `[T; N]` | Fixed-size homogeneous collection | `[2, 3, 5]` |
| Unit | `()` | Zero-size type, single value | `()` |

### Unsized Types

| Type | Syntax | Description | Example |
|------|--------|-------------|---------|
| Slice | `[T]` | Unsized array slice | — |
| String slice | `str` | Unsized string slice | — |
| Trait object | `dyn Trait` | Unsized trait object | — |
| Shared slice ref | `&[T]` | Shared array slice | `&arr[i..j]` |
| Mutable slice ref | `&mut [T]` | Mutable array slice | `&mut arr[i..j]` |
| Shared string ref | `&str` | Shared string slice | `"text"` |
| Mutable string ref | `&mut str` | Mutable string slice | — |
| Shared trait obj ref | `&dyn Trait` | Shared trait object | `&err as &dyn Error` |
| Mutable trait obj ref | `&mut dyn Trait` | Mutable trait object | — |

### Borrowed Reference Types

| Type | Syntax | Description | Example |
|------|--------|-------------|---------|
| Shared reference | `&T` | Immutable borrow | `&x` |
| Mutable reference | `&mut T` | Mutable borrow | `&mut x` |

### Range Types

| Type | Syntax | Description |
|------|--------|-------------|
| `Range<T>` | `i..j` | Half-open range |
| `RangeTo<T>` | `..j` | Bounded above, open |
| `RangeFrom<T>` | `i..` | Bounded below |
| `RangeInclusive<T>` | `i..=j` | Closed range |
| `RangeToInclusive<T>` | `..=j` | Bounded above, closed |
| `RangeFull` | `..` | Unbounded |

### Utility Types

| Type | Description | Values |
|------|-------------|--------|
| `Option<T>` | Optional value | `Some(val)`, `None` |
| `Result<T, E>` | Success or error | `Ok(val)`, `Err(e)` |
| `Ordering` | Comparison result | `Less`, `Equal`, `Greater` |
| `Arguments` | Precompiled format string | `"x is {}"` |

### Async Support Types

| Type | Description | Values |
|------|-------------|--------|
| `Poll<T>` | Future completion status | `Pending`, `Ready(x)` |
| `Context<'a>` | Task context | — |
| `Pin<T>` | Immovable object pointer | — |

### Anonymous Types

| Type | Description | Example |
|------|-------------|---------|
| Function item | Named function | `fn foo()` |
| Closure | Anonymous function | `\|x\| x > threshold` |
| Async function | Async named function | `async fn foo()` |
| Async closure | Async anonymous function | `async \|\| f.await` |
| `impl Trait` | Existential type | `fn f() -> impl Trait` |

### Unsafe Support Types

| Type | Description |
|------|-------------|
| `UnsafeCell<T>` | Interior mutability primitive |
| `ManuallyDrop<T>` | Inhibit destructor |
| `PhantomData<T>` | Act like you own a `T` |

### Raw Pointer Types

| Type | Description |
|------|-------------|
| `*const T` | Const raw pointer |
| `*mut T` | Mutable raw pointer |

### Function Pointers

| Type | Syntax | Description | Example |
|------|--------|-------------|---------|
| Function pointer | `fn(T…) -> U` | Pointer to a function | `foo as fn()` |

### Panic Support

| Type | Description |
|------|-------------|
| `PanicInfo` | Info about a panic |
| `Location` | Location of a panic |

### Uninhabited Type

| Type | Syntax | Description | Example |
|------|--------|-------------|---------|
| Never | `!` | Type with no values | `fn exit(i32) -> !` |

---

## Traits

### Access Operator Traits

| Trait | Syntax | Description | Example |
|-------|--------|-------------|---------|
| `Deref` | `*p` | Immutable dereference | `x = *p` |
| `Index` | `arr[i]` | Immutable index | `x = arr[i]` |
| `RangeBounds<T>` | `arr[i..j]` | Range as index | `arr[i..j]` |
| `DerefMut` | `*p = x` | Mutable dereference | `*p = x` |
| `IndexMut` | `arr[i] = x` | Mutable index | `arr[i] = x` |

### Comparison Operator Traits

| Trait | Syntax | Description | Example |
|-------|--------|-------------|---------|
| `PartialOrd<T>` | `<`, `>`, `<=`, `>=` | Partial ordering | `x < y` |
| `PartialEq<T>` | `==`, `!=` | Partial equivalence | `x == y` |
| `Ord<T>` | — | Total ordering | — |
| `Eq<T>` | — | Full equivalence | — |

### Arithmetic Operator Traits

| Trait | Syntax | Description | Example |
|-------|--------|-------------|---------|
| `Add<T>` | `+` | Addition | `x + y` |
| `Sub<T>` | `-` | Subtraction | `x - y` |
| `Mul<T>` | `*` | Multiplication | `x * y` |
| `Div<T>` | `/` | Division | `x / y` |
| `Rem<T>` | `%` | Remainder | `x % y` |
| `Neg` | `-` | Negation | `-x` |
| `AddAssign<T>` | `+=` | Addition assignment | `x += y` |
| `SubAssign<T>` | `-=` | Subtraction assignment | `x -= y` |
| `MulAssign<T>` | `*=` | Multiplication assignment | `x *= y` |
| `DivAssign<T>` | `/=` | Division assignment | `x /= y` |
| `RemAssign<T>` | `%=` | Remainder assignment | `x %= y` |

### Bitwise Operator Traits

| Trait | Syntax | Description | Example |
|-------|--------|-------------|---------|
| `BitAnd<T>` | `&` | Bitwise AND | `x & y` |
| `BitOr<T>` | `\|` | Bitwise OR | `x \| y` |
| `BitXor<T>` | `^` | Bitwise XOR | `x ^ y` |
| `Shl<T>` | `<<` | Shift left | `x << y` |
| `Shr<T>` | `>>` | Shift right | `x >> y` |
| `Not` | `!` | Bitwise NOT | `!x` |
| `BitAndAssign<T>` | `&=` | Bitwise AND assignment | `x &= y` |
| `BitOrAssign<T>` | `\|=` | Bitwise OR assignment | `x \|= y` |
| `BitXorAssign<T>` | `^=` | Bitwise XOR assignment | `x ^= y` |
| `ShlAssign<T>` | `<<=` | Shift left assignment | `x <<= y` |
| `ShrAssign<T>` | `>>=` | Shift right assignment | `x >>= y` |

### Callable Traits

| Trait | Description | Example |
|-------|-------------|---------|
| `FnOnce(T…) -> U` | Callable if owned (consumes self) | `move \|\| s` |
| `FnMut(T…) -> U` | Callable if mutable | `\|\| x += 1` |
| `Fn(T…) -> U` | Callable while shared | `\|x\| x + 1` |
| `AsyncFnOnce(T…) -> U` | Async closure, callable if owned | — |
| `AsyncFnMut(T…) -> U` | Async closure, callable if mutable | — |
| `AsyncFn(T…) -> U` | Async closure, callable while shared | — |

### Memory Management Traits

| Trait | Description |
|-------|-------------|
| `Sized` | Type has a known size at compile time |
| `Copy` | Implicit bitwise duplication |
| `Drop` | Custom destructor logic |
| `Clone` | Explicit duplication via `.clone()` |

### Iteration Traits

| Trait | Description | Example |
|-------|-------------|---------|
| `Iterator` | Produces a sequence of values | `for x in 0..10` |
| `IntoIterator` | Can be converted into an iterator | `for x in [2, 3, 5]` |

### Thread Safety Traits

| Trait | Description |
|-------|-------------|
| `Send` | Type can be transferred across thread boundaries |
| `Sync` | Type can be shared across threads via references |

### Async Support Traits

| Trait | Description | Example |
|-------|-------------|---------|
| `Future` | Represents an async computation | `foo().await` |
| `Unpin` | Future that doesn't need `Pin` | — |

### Panic Support Traits

| Trait | Description |
|-------|-------------|
| `UnwindSafe` | Types safe to hold across a panic boundary |
| `RefUnwindSafe` | Helper trait for `UnwindSafe` via references |

### Termination Trait

| Trait | Description |
|-------|-------------|
| `Termination` | Valid return types from `main()` |
