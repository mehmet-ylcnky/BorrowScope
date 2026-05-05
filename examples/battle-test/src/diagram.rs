use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

#[trace_borrow]
fn ownership_flow() {
    let data = vec![1, 2, 3, 4, 5];
    let slice = &data;
    let first = slice.first();
    let owned = data.clone();
    let moved = owned;
    println!("{:?} {:?} {:?}", slice, first, moved);
}

fn main() {
    reset();
    ownership_flow();
    let events = get_events();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║         BORROWSCOPE: Ownership Flow Diagram                 ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║                                                              ║");
    println!("║  Source: fn ownership_flow()                                 ║");
    println!("║                                                              ║");
    println!("║  let data = vec![1,2,3,4,5];                                 ║");
    println!("║  let slice = &data;                                          ║");
    println!("║  let first = slice.first();                                  ║");
    println!("║  let owned = data.clone();                                   ║");
    println!("║  let moved = owned;                                          ║");
    println!("║                                                              ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║                                                              ║");
    println!("║  Timeline ({} events captured):                       ║", events.len());
    println!("║                                                              ║");

    for (i, event) in events.iter().enumerate() {
        let desc = match event {
            Event::New { var_id, .. } => format!("CREATE  ─── {} ───────────────── [owned]", var_id),
            Event::Borrow { borrower_id, owner_id, .. } => format!("BORROW  ─── {} ──── &{} ──── [shared ref]", borrower_id, owner_id),
            Event::Move { from_id, to_id, .. } => format!("MOVE    ─── {} ═══▶ {} ──── [ownership transferred]", from_id, to_id),
            Event::Clone { var_name, .. } => format!("CLONE   ─── {} ───────────────── [deep copy]", var_name),
            Event::Drop { var_id, .. } => format!("DROP    ─── {} ───────────────── [freed]", var_id),
            Event::Call { fn_name, .. } => format!("CALL    ─── {} ─────────── [method]", fn_name),
            _ => format!("EVENT   ─── {:?}", std::mem::discriminant(event)),
        };
        println!("║  {:>2}. {}", i + 1, desc);
    }

    println!("║                                                              ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║                                                              ║");
    println!("║  Ownership Graph:                                            ║");
    println!("║                                                              ║");
    println!("║    data ─────┬──── &data ──── slice                          ║");
    println!("║              │                   │                            ║");
    println!("║              │                   └──── .first() ──── first    ║");
    println!("║              │                                                ║");
    println!("║              ├──── .clone() ──── owned                        ║");
    println!("║              │                     │                          ║");
    println!("║              │                     └════▶ moved (ownership)   ║");
    println!("║              │                                                ║");
    println!("║              └──── [drop at end of scope]                     ║");
    println!("║                                                              ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}
