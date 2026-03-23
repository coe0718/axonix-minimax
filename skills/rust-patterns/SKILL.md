---
name: Rust Patterns
description: Rust patterns Axonix gets wrong repeatedly. Consult this before writing Rust code.
---

# Rust Patterns

Reference guide for Rust patterns that are easy to get wrong. Check this before
writing any non-trivial Rust code.

## Ownership and Cloning

### Arc/Tools Must Be Cloned

`Arc<T>` does not implement `Copy`. You MUST call `.clone()` for each thread/agent:

```rust
// WRONG: all agents share the same Arc — mutation races
let shared = Arc::new(state.clone());
spawn(agent(shared)); // only one Arc

// RIGHT: clone for each consumer
let shared = Arc::new(state);
let a = shared.clone();
let b = shared.clone();
spawn(agent(a));
spawn(agent(b));
```

### Vec<Box<dyn T>> vs Vec<Arc<dyn T>>

- `Box<dyn Trait>` — single owner, no shared access across threads
- `Arc<dyn Trait>` — shared ownership across threads

```rust
// Single-threaded or single-owner use:
let items: Vec<Box<dyn Processor>> = vec![];

// Multi-threaded shared access:
let items: Vec<Arc<dyn Processor + Send + Sync>> = vec![];
```

### Moving vs Borrowing in Match Arms

Each match arm can either move or borrow — not both implicitly:

```rust
let s = String::from("hello");

// WRONG: s is moved into this arm, then borrowed
match x {
    A => println!("{}", s),  // s borrowed here
    B => drop(s),            // s moved here — COMPILE ERROR if combined
}

// RIGHT: clone if you need both
match x {
    A => println!("{}", s),
    B => drop(s.clone()),
}
```

### Clone vs Copy Semantics

- `Copy` types are implicitly duplicated (i32, bool, char, etc.)
- `Clone` types must be explicitly cloned
- `Arc` is `Clone` but NOT `Copy` — calling `.clone()` is cheap (reference count increment)
- Never `.clone()` on large data structures unless you mean it

## Error Handling

### Use `?` for Propagation

```rust
// WRONG: verbose nested match
fn read_config() -> Result<Config, Box<dyn Error>> {
    let file = match File::open("config.toml") {
        Ok(f) => f,
        Err(e) => return Err(e.into()),
    };
    // ...
}

// RIGHT: ? operator propagates
fn read_config() -> Result<Config, Box<dyn Error>> {
    let file = File::open("config.toml")?;
    let mut buf = String::new();
    BufReader::new(file).read_to_string(&mut buf)?;
    Ok(toml::from_str(&buf)?)
}
```

### Error Type Consistency

If a function uses `?`, all intermediate errors must be convertible:

```rust
// Both errors must implement Into<MyError>
fn parse_input(s: &str) -> Result<Value, MyError> {
    let n: i64 = s.trim().parse::<i64>()?; // parse error -> MyError via From
    Ok(Value::Number(n))
}
```

### Don't Use unwrap() in Library Code

```rust
// WRONG: panics on invalid input
fn get_first(items: &[i32]) -> i32 {
    items.first().unwrap()
}

// RIGHT: return Result or provide a default
fn get_first(items: &[i32]) -> Option<i32> {
    items.first().copied()
}
```

## Async Patterns

### Spawning Tasks with Shared State

```rust
// Every task that needs shared state needs its own Arc clone
let state = Arc::new(AppState::new());

for task in tasks {
    let state = state.clone(); // each iteration gets its own Arc
    tokio::spawn(async move {
        state.do_work().await;
    });
}
```

### Avoid Blocking in Async Context

```rust
// WRONG: blocks the async runtime
async fn bad() {
    let data = std::fs::read("file.txt").unwrap();
}

// RIGHT: use async I/O
async fn good() {
    let data = tokio::fs::read("file.txt").await.unwrap();
}
```

## Common API Patterns

### Derive Debug for Error Types

```rust
#[derive(Debug)]
enum MyError {
    Io(std::io::Error),
    Parse(String),
}

impl std::error::Error for MyError {}
```

### Implementing From for Error Conversion

```rust
impl From<std::io::Error> for MyError {
    fn from(err: std::io::Error) -> Self {
        MyError::Io(err)
    }
}

// Now ? works automatically
fn might_io() -> Result<(), MyError> {
    let f = File::open("foo")?; // io::Error -> MyError
    Ok(())
}
```

### Iterating with Iterators

```rust
// WRONG: reborrow in loop
for item in &items {
    items.push(new_item()); // can't mutably borrow while immutably borrowed
}

// RIGHT: collect indices or use indices
for i in 0..items.len() {
    if condition(&items[i]) {
        items.push(new_item());
    }
}
```

### Option and Result Chaining

```rust
// Chaining with and_then / map
let value = config
    .get("database")
    .and_then(|db| db.get("port"))
    .and_then(|p| p.as_str().parse::<u16>().ok())
    .unwrap_or(5432);

// Filter map pattern
let valid = items.iter().filter_map(|item| validate(item).ok()).collect();
```

## String and Str Patterns

### Building Strings

```rust
// String + &str: coerces
let s = String::from("hello");
let combined = s + " world";     // takes ownership of s
let combined2 = format!("{} {}", s, "world"); // keeps s

// Avoid: repeated allocation in loop
let mut result = String::new();
for item in items {
    result.push_str(&format!("{}, ", item)); // allocates every iteration
}

// Better: collect into Vec then join
let parts: Vec<_> = items.iter().map(|i| i.to_string()).collect();
let result = parts.join(", ");
```

### String Slices

```rust
let s = String::from("hello world");
let slice: &str = &s;     // &String coerces to &str
let bytes: &[u8] = s.as_bytes();
```

## Lifetime Patterns

### Avoid Unnecessary Lifetimes

```rust
// WRONG: unnecessary lifetime annotation
fn first_word<'a>(s: &'a str) -> &'a str { ... }

// RIGHT: elision rules handle it
fn first_word(s: &str) -> &str { ... }
```

### Struct Lifetimes

```rust
// If a struct holds a reference, it needs a lifetime
struct Parser<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Parser { input, position: 0 }
    }
}
```

## Testing Patterns

### Testing Private Functions

Test the module, not just public APIs:

```rust
// In tests, you can access private items
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_function() {
        assert_eq!(internal_helper(10), 20);
    }
}
```

### Asserting with Messages

```rust
// WRONG: cryptic test failure
assert_eq!(result, 42);

// RIGHT: helpful message
assert_eq!(result, 42, "expected 42 for input {}, got {}", input, result);
```

## When to Ask for Help

If you're unsure which pattern applies, prefer:
1. `Result<T, E>` over `Option<T>` when the caller needs to know why
2. `Clone` over `Rc` when you might need `Send + Sync` later
3. `async fn` over `fn -> Future` for clarity
4. `Box<dyn Trait>` over trait objects in generics when the trait is used in multiple places
