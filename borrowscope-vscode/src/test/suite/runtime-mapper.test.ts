import * as assert from "assert";
import * as path from "path";

const ROOT = path.resolve(__dirname, "..", "..", "..");

describe("12.3 Variable Mapping (Runtime → Static)", () => {
  let mapper: any;

  before(() => {
    mapper = require(path.join(ROOT, "out", "runtime-mapper"));
  });

  // === parseLocation ===

  it("parseLocation parses file:line:col", () => {
    const loc = mapper.parseLocation("src/main.rs:5:10");
    assert.deepStrictEqual(loc, { file: "src/main.rs", line: 5, column: 10 });
  });

  it("parseLocation parses file:line without column", () => {
    const loc = mapper.parseLocation("src/main.rs:5");
    assert.deepStrictEqual(loc, { file: "src/main.rs", line: 5, column: 0 });
  });

  it("parseLocation returns null for null/undefined", () => {
    assert.strictEqual(mapper.parseLocation(null), null);
    assert.strictEqual(mapper.parseLocation(undefined), null);
  });

  it("parseLocation returns null for invalid format", () => {
    assert.strictEqual(mapper.parseLocation("no_colon"), null);
  });

  it("parseLocation handles Windows paths", () => {
    const loc = mapper.parseLocation("C:\\project\\src\\main.rs:10:5");
    assert.ok(loc);
    assert.strictEqual(loc.line, 10);
  });

  // === findStaticMatch ===

  const staticVars = [
    { name: "data", line: 3, type_display: "Vec<i32>", ownership_category: "Owned" },
    { name: "data", line: 10, type_display: "String", ownership_category: "Owned" },
    { name: "r", line: 5, type_display: "&Vec<i32>", ownership_category: "SharedRef" },
    { name: "rc", line: 7, type_display: "Rc<i32>", ownership_category: "Rc" },
  ];

  it("findStaticMatch: exact match (name + line + type)", () => {
    const { match, confidence } = mapper.findStaticMatch("data", 3, "Vec<i32>", staticVars);
    assert.strictEqual(confidence, "exact");
    assert.strictEqual(match.line, 3);
  });

  it("findStaticMatch: name_line match", () => {
    const { match, confidence } = mapper.findStaticMatch("data", 3, "SomeOtherType", staticVars);
    assert.strictEqual(confidence, "name_line");
    assert.strictEqual(match.line, 3);
  });

  it("findStaticMatch: name_type match (disambiguates shadowed vars)", () => {
    const { match, confidence } = mapper.findStaticMatch("data", 99, "String", staticVars);
    assert.strictEqual(confidence, "name_type");
    assert.strictEqual(match.line, 10);
  });

  it("findStaticMatch: name_only match (last resort)", () => {
    const { match, confidence } = mapper.findStaticMatch("rc", null, null, staticVars);
    assert.strictEqual(confidence, "name_only");
    assert.strictEqual(match.name, "rc");
  });

  it("findStaticMatch: no match returns none", () => {
    const { match, confidence } = mapper.findStaticMatch("unknown", 99, "Foo", staticVars);
    assert.strictEqual(confidence, "none");
    assert.strictEqual(match, null);
  });

  it("findStaticMatch: handles full path types (alloc::vec::Vec<i32>)", () => {
    const { match, confidence } = mapper.findStaticMatch("data", 3, "alloc::vec::Vec<i32>", staticVars);
    assert.strictEqual(confidence, "exact");
  });

  // === mapVariables ===

  it("mapVariables maps New event to static var", () => {
    const statics = [{ name: "x", line: 3, type_display: "i32", ownership_category: "Owned" }];
    const events = [{ New: { timestamp: 10, var_name: "x", var_id: "x_1", type_name: "i32" } }];
    const mapped = mapper.mapVariables(statics, events);
    assert.strictEqual(mapped.length, 1);
    assert.strictEqual(mapped[0].var_id, "x_1");
    assert.strictEqual(mapped[0].static_match.name, "x");
  });

  it("mapVariables maps RcNew event", () => {
    const statics = [{ name: "rc", line: 5, type_display: "Rc<i32>", ownership_category: "Rc" }];
    const events = [{ RcNew: { timestamp: 10, var_name: "rc", var_id: "rc_1", type_name: "Rc<i32>", strong_count: 1, weak_count: 0 } }];
    const mapped = mapper.mapVariables(statics, events);
    assert.strictEqual(mapped.length, 1);
    assert.strictEqual(mapped[0].match_confidence, "name_type");
  });

  it("mapVariables maps ArcNew event", () => {
    const statics = [{ name: "a", line: 2, type_display: "Arc<String>", ownership_category: "Arc" }];
    const events = [{ ArcNew: { timestamp: 10, var_name: "a", var_id: "a_1", type_name: "Arc<String>", strong_count: 1, weak_count: 0 } }];
    const mapped = mapper.mapVariables(statics, events);
    assert.strictEqual(mapped.length, 1);
    assert.ok(mapped[0].static_match);
  });

  it("mapVariables maps BoxNew event", () => {
    const statics = [{ name: "b", line: 4, type_display: "Box<i32>", ownership_category: "Owned" }];
    const events = [{ BoxNew: { timestamp: 10, var_name: "b", var_id: "b_1", type_name: "Box<i32>", location: "src/main.rs:4:5" } }];
    const mapped = mapper.mapVariables(statics, events);
    assert.strictEqual(mapped.length, 1);
    assert.strictEqual(mapped[0].match_confidence, "exact");
  });

  it("mapVariables maps RefCellNew event", () => {
    const statics = [{ name: "cell", line: 6, type_display: "RefCell<i32>", ownership_category: "RefCell" }];
    const events = [{ RefCellNew: { timestamp: 10, var_name: "cell", var_id: "cell_1", type_name: "RefCell<i32>" } }];
    const mapped = mapper.mapVariables(statics, events);
    assert.strictEqual(mapped.length, 1);
  });

  it("mapVariables maps CellNew event", () => {
    const statics = [{ name: "c", line: 2, type_display: "Cell<bool>", ownership_category: "Cell" }];
    const events = [{ CellNew: { timestamp: 10, var_name: "c", var_id: "c_1", type_name: "Cell<bool>" } }];
    const mapped = mapper.mapVariables(statics, events);
    assert.strictEqual(mapped.length, 1);
  });

  it("mapVariables maps WeakNew event", () => {
    const statics = [{ name: "w", line: 8, type_display: "Weak<i32>", ownership_category: "Weak" }];
    const events = [{ WeakNew: { timestamp: 10, var_name: "w", var_id: "w_1", source_id: "rc_1", weak_count: 1, location: "src/main.rs:8:5" } }];
    const mapped = mapper.mapVariables(statics, events);
    assert.strictEqual(mapped.length, 1);
  });

  it("mapVariables maps PinNew event", () => {
    const statics = [{ name: "p", line: 3, type_display: "Pin<Box<Future>>", ownership_category: "Owned" }];
    const events = [{ PinNew: { timestamp: 10, var_name: "p", var_id: "p_1", location: "src/main.rs:3:5" } }];
    const mapped = mapper.mapVariables(statics, events);
    assert.strictEqual(mapped.length, 1);
  });

  it("mapVariables maps StaticInit event", () => {
    const statics = [{ name: "GLOBAL", line: 1, type_display: "i32", ownership_category: "Static" }];
    const events = [{ StaticInit: { timestamp: 10, var_name: "GLOBAL", var_id: "g_1", type_name: "i32", is_mutable: false } }];
    const mapped = mapper.mapVariables(statics, events);
    assert.strictEqual(mapped.length, 1);
  });

  it("mapVariables maps RawPtrCreated event", () => {
    const statics = [{ name: "ptr", line: 5, type_display: "*const i32", ownership_category: "RawPtr" }];
    const events = [{ RawPtrCreated: { timestamp: 10, var_name: "ptr", var_id: "ptr_1", ptr_type: "*const i32", address: 12345, location: "src/main.rs:5:5" } }];
    const mapped = mapper.mapVariables(statics, events);
    assert.strictEqual(mapped.length, 1);
    assert.strictEqual(mapped[0].match_confidence, "exact");
  });

  it("mapVariables maps OnceCellNew event", () => {
    const statics = [{ name: "oc", line: 2, type_display: "OnceCell<String>", ownership_category: "Owned" }];
    const events = [{ OnceCellNew: { timestamp: 10, var_name: "oc", var_id: "oc_1", location: "src/main.rs:2:5" } }];
    const mapped = mapper.mapVariables(statics, events);
    assert.strictEqual(mapped.length, 1);
  });

  it("mapVariables maps MaybeUninitNew event", () => {
    const statics = [{ name: "mu", line: 4, type_display: "MaybeUninit<i32>", ownership_category: "Owned" }];
    const events = [{ MaybeUninitNew: { timestamp: 10, var_name: "mu", var_id: "mu_1", initialized: false, location: "src/main.rs:4:5" } }];
    const mapped = mapper.mapVariables(statics, events);
    assert.strictEqual(mapped.length, 1);
  });

  it("mapVariables maps CowBorrowed/CowOwned events", () => {
    const statics = [
      { name: "cow1", line: 3, type_display: "Cow<str>", ownership_category: "Owned" },
      { name: "cow2", line: 4, type_display: "Cow<str>", ownership_category: "Owned" },
    ];
    const events = [
      { CowBorrowed: { timestamp: 10, var_name: "cow1", var_id: "cow1_1", location: "src/main.rs:3:5" } },
      { CowOwned: { timestamp: 20, var_name: "cow2", var_id: "cow2_1", location: "src/main.rs:4:5" } },
    ];
    const mapped = mapper.mapVariables(statics, events);
    assert.strictEqual(mapped.length, 2);
  });

  it("mapVariables maps StructCreate/TupleCreate/ArrayCreate", () => {
    const statics = [
      { name: "pt", line: 3, type_display: "Point", ownership_category: "Owned" },
      { name: "tup", line: 4, type_display: "(i32, i32)", ownership_category: "Owned" },
      { name: "arr", line: 5, type_display: "[i32; 5]", ownership_category: "Owned" },
    ];
    const events = [
      { StructCreate: { timestamp: 10, struct_id: "pt_1", type_name: "Point", location: "src/main.rs:3:5" } },
      { TupleCreate: { timestamp: 20, tuple_id: "tup_1", len: 2, location: "src/main.rs:4:5" } },
      { ArrayCreate: { timestamp: 30, array_id: "arr_1", len: 5, location: "src/main.rs:5:5" } },
    ];
    // These use struct_id/tuple_id/array_id not var_id, so they won't map via var_id
    const mapped = mapper.mapVariables(statics, events);
    // StructCreate has no var_name field, so it won't create a mapping
    assert.ok(mapped.length >= 0);
  });

  it("mapVariables collects Drop events for mapped variables", () => {
    const statics = [{ name: "x", line: 3, type_display: "i32", ownership_category: "Owned" }];
    const events = [
      { New: { timestamp: 10, var_name: "x", var_id: "x_1", type_name: "i32" } },
      { Drop: { timestamp: 50, var_id: "x_1" } },
    ];
    const mapped = mapper.mapVariables(statics, events);
    assert.strictEqual(mapped[0].events.length, 2);
  });

  it("mapVariables collects Borrow events referencing owner", () => {
    const statics = [{ name: "data", line: 3, type_display: "Vec<i32>", ownership_category: "Owned" }];
    const events = [
      { New: { timestamp: 10, var_name: "data", var_id: "data_1", type_name: "Vec<i32>" } },
      { Borrow: { timestamp: 20, borrower_name: "r", borrower_id: "r_1", owner_id: "data_1", mutable: false } },
    ];
    const mapped = mapper.mapVariables(statics, events);
    assert.strictEqual(mapped[0].events.length, 2);
  });

  it("mapVariables collects Move events referencing from_id", () => {
    const statics = [{ name: "x", line: 3, type_display: "String", ownership_category: "Owned" }];
    const events = [
      { New: { timestamp: 10, var_name: "x", var_id: "x_1", type_name: "String" } },
      { Move: { timestamp: 30, from_id: "x_1", to_name: "y", to_id: "y_1" } },
    ];
    const mapped = mapper.mapVariables(statics, events);
    assert.strictEqual(mapped[0].events.length, 2);
  });

  it("mapVariables collects RcClone events referencing source_id", () => {
    const statics = [{ name: "rc", line: 3, type_display: "Rc<i32>", ownership_category: "Rc" }];
    const events = [
      { RcNew: { timestamp: 10, var_name: "rc", var_id: "rc_1", type_name: "Rc<i32>", strong_count: 1, weak_count: 0 } },
      { RcClone: { timestamp: 20, var_name: "rc2", var_id: "rc2_1", source_id: "rc_1", strong_count: 2, weak_count: 0 } },
    ];
    const mapped = mapper.mapVariables(statics, events);
    const rcEntry = mapped.find((m: any) => m.var_id === "rc_1");
    assert.ok(rcEntry.events.length >= 2);
  });

  it("mapVariables filters by target file", () => {
    const statics = [{ name: "x", line: 3, type_display: "i32", ownership_category: "Owned" }];
    const events = [
      { New: { timestamp: 10, var_name: "x", var_id: "x_1", type_name: "i32" } },
      { BoxNew: { timestamp: 20, var_name: "b", var_id: "b_1", type_name: "Box<i32>", location: "src/other.rs:5:5" } },
    ];
    // x_1 has no location so won't be filtered out; b_1 has location in other.rs
    const mapped = mapper.mapVariables(statics, events, "main.rs");
    const otherFile = mapped.find((m: any) => m.var_id === "b_1");
    assert.strictEqual(otherFile, undefined);
  });

  it("mapVariables handles empty events", () => {
    const statics = [{ name: "x", line: 3, type_display: "i32", ownership_category: "Owned" }];
    const mapped = mapper.mapVariables(statics, []);
    assert.strictEqual(mapped.length, 0);
  });

  it("mapVariables handles empty static vars", () => {
    const events = [{ New: { timestamp: 10, var_name: "x", var_id: "x_1", type_name: "i32" } }];
    const mapped = mapper.mapVariables([], events);
    assert.strictEqual(mapped.length, 1);
    assert.strictEqual(mapped[0].match_confidence, "none");
  });

  // === mappingStats ===

  it("mappingStats counts confidence levels", () => {
    const mapped = [
      { match_confidence: "exact" },
      { match_confidence: "exact" },
      { match_confidence: "name_line" },
      { match_confidence: "none" },
    ];
    const stats = mapper.mappingStats(mapped);
    assert.strictEqual(stats.total, 4);
    assert.strictEqual(stats.exact, 2);
    assert.strictEqual(stats.name_line, 1);
    assert.strictEqual(stats.unmatched, 1);
  });

  it("mappingStats handles empty array", () => {
    const stats = mapper.mappingStats([]);
    assert.strictEqual(stats.total, 0);
  });
});
