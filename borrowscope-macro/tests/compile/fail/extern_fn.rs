use borrowscope_macro::trace_borrow;

#[trace_borrow]
extern "C" fn example() {
    let x = 42;
}

fn main() {}
