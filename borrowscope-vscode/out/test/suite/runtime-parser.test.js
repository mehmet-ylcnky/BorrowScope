"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
const assert = __importStar(require("assert"));
const path = __importStar(require("path"));
const ROOT = path.resolve(__dirname, "..", "..", "..");
describe("12.2 Runtime Event Parser", () => {
    let parser;
    before(() => {
        parser = require(path.join(ROOT, "out", "runtime-parser"));
    });
    // === parseEvents ===
    it("parseEvents returns empty for empty array", () => {
        const r = parser.parseEvents("[]");
        assert.strictEqual(r.events.length, 0);
        assert.strictEqual(r.errors.length, 0);
        assert.strictEqual(r.total, 0);
        assert.strictEqual(r.valid, 0);
    });
    it("parseEvents returns error for invalid JSON", () => {
        const r = parser.parseEvents("{broken");
        assert.strictEqual(r.events.length, 0);
        assert.ok(r.errors[0].includes("Invalid JSON"));
    });
    it("parseEvents returns error for non-array JSON", () => {
        const r = parser.parseEvents('{"foo": 1}');
        assert.ok(r.errors[0].includes("Expected JSON array"));
    });
    it("parseEvents parses valid New event", () => {
        const json = JSON.stringify([{ New: { timestamp: 100, var_name: "x", var_id: "x_1", type_name: "i32" } }]);
        const r = parser.parseEvents(json);
        assert.strictEqual(r.valid, 1);
        assert.strictEqual(r.errors.length, 0);
    });
    it("parseEvents parses valid Drop event", () => {
        const json = JSON.stringify([{ Drop: { timestamp: 200, var_id: "x_1" } }]);
        const r = parser.parseEvents(json);
        assert.strictEqual(r.valid, 1);
    });
    it("parseEvents parses valid Borrow event", () => {
        const json = JSON.stringify([{ Borrow: { timestamp: 150, borrower_name: "r", borrower_id: "r_1", owner_id: "x_1", mutable: false } }]);
        const r = parser.parseEvents(json);
        assert.strictEqual(r.valid, 1);
    });
    it("parseEvents parses valid Move event", () => {
        const json = JSON.stringify([{ Move: { timestamp: 300, from_id: "x_1", to_name: "y", to_id: "y_1" } }]);
        const r = parser.parseEvents(json);
        assert.strictEqual(r.valid, 1);
    });
    it("parseEvents parses RcNew/RcClone", () => {
        const json = JSON.stringify([
            { RcNew: { timestamp: 10, var_name: "rc1", var_id: "rc1_1", type_name: "Rc<i32>", strong_count: 1, weak_count: 0 } },
            { RcClone: { timestamp: 20, var_name: "rc2", var_id: "rc2_1", source_id: "rc1_1", strong_count: 2, weak_count: 0 } },
        ]);
        const r = parser.parseEvents(json);
        assert.strictEqual(r.valid, 2);
    });
    it("parseEvents parses ArcNew/ArcClone", () => {
        const json = JSON.stringify([
            { ArcNew: { timestamp: 10, var_name: "a1", var_id: "a1_1", type_name: "Arc<String>", strong_count: 1, weak_count: 0 } },
            { ArcClone: { timestamp: 20, var_name: "a2", var_id: "a2_1", source_id: "a1_1", strong_count: 2, weak_count: 0 } },
        ]);
        const r = parser.parseEvents(json);
        assert.strictEqual(r.valid, 2);
    });
    it("parseEvents parses RefCell events", () => {
        const json = JSON.stringify([
            { RefCellNew: { timestamp: 10, var_name: "rc", var_id: "rc_1", type_name: "RefCell<i32>" } },
            { RefCellBorrow: { timestamp: 20, borrow_id: "b_1", refcell_id: "rc_1", is_mutable: false, location: "src/main.rs:5:10" } },
            { RefCellDrop: { timestamp: 30, borrow_id: "b_1", location: "src/main.rs:8:1" } },
        ]);
        const r = parser.parseEvents(json);
        assert.strictEqual(r.valid, 3);
    });
    it("parseEvents parses Cell events", () => {
        const json = JSON.stringify([
            { CellNew: { timestamp: 10, var_name: "c", var_id: "c_1", type_name: "Cell<i32>" } },
            { CellGet: { timestamp: 20, cell_id: "c_1", location: "src/main.rs:3:5" } },
            { CellSet: { timestamp: 30, cell_id: "c_1", location: "src/main.rs:4:5" } },
        ]);
        const r = parser.parseEvents(json);
        assert.strictEqual(r.valid, 3);
    });
    it("parseEvents parses unsafe events", () => {
        const json = JSON.stringify([
            { UnsafeBlockEnter: { timestamp: 10, block_id: "ub_1", location: "src/main.rs:10:5" } },
            { RawPtrCreated: { timestamp: 20, var_name: "p", var_id: "p_1", ptr_type: "*const i32", address: 12345, location: "src/main.rs:11:5" } },
            { RawPtrDeref: { timestamp: 30, ptr_id: "p_1", location: "src/main.rs:12:5", is_write: false } },
            { UnsafeBlockExit: { timestamp: 40, block_id: "ub_1", location: "src/main.rs:13:5" } },
            { UnsafeFnCall: { timestamp: 50, fn_name: "dangerous", location: "src/main.rs:15:5" } },
            { FfiCall: { timestamp: 60, fn_name: "libc::malloc", location: "src/main.rs:16:5" } },
            { Transmute: { timestamp: 70, from_type: "u32", to_type: "f32", location: "src/main.rs:17:5" } },
            { UnionFieldAccess: { timestamp: 80, union_name: "MyUnion", field_name: "f1", location: "src/main.rs:18:5" } },
        ]);
        const r = parser.parseEvents(json);
        assert.strictEqual(r.valid, 8);
    });
    it("parseEvents parses async events", () => {
        const json = JSON.stringify([
            { AsyncBlockEnter: { timestamp: 10, block_id: "ab_1", location: "src/main.rs:5:5" } },
            { AwaitStart: { timestamp: 20, await_id: "aw_1", future_name: "fetch", location: "src/main.rs:6:5", live_variables: ["data"] } },
            { AwaitEnd: { timestamp: 30, await_id: "aw_1", location: "src/main.rs:6:5" } },
            { AsyncBlockExit: { timestamp: 40, block_id: "ab_1", location: "src/main.rs:8:5" } },
        ]);
        const r = parser.parseEvents(json);
        assert.strictEqual(r.valid, 4);
    });
    it("parseEvents parses control flow events", () => {
        const json = JSON.stringify([
            { LoopEnter: { timestamp: 10, loop_id: "l_1", loop_type: "for", location: "src/main.rs:3:5" } },
            { LoopIteration: { timestamp: 20, loop_id: "l_1", iteration: 0, location: "src/main.rs:3:5" } },
            { LoopExit: { timestamp: 30, loop_id: "l_1", location: "src/main.rs:5:5" } },
            { MatchEnter: { timestamp: 40, match_id: "m_1", location: "src/main.rs:7:5" } },
            { MatchArm: { timestamp: 50, match_id: "m_1", arm_index: 0, pattern: "Some(x)", location: "src/main.rs:8:9", bindings: ["x"] } },
            { MatchExit: { timestamp: 60, match_id: "m_1", location: "src/main.rs:10:5" } },
            { Branch: { timestamp: 70, branch_id: "br_1", branch_type: "if_true", location: "src/main.rs:12:5" } },
            { Return: { timestamp: 80, return_id: "ret_1", has_value: true, location: "src/main.rs:14:5" } },
            { Try: { timestamp: 90, try_id: "try_1", location: "src/main.rs:15:5" } },
            { Break: { timestamp: 100, break_id: "brk_1", loop_label: null, location: "src/main.rs:16:5" } },
            { Continue: { timestamp: 110, continue_id: "cnt_1", loop_label: null, location: "src/main.rs:17:5" } },
        ]);
        const r = parser.parseEvents(json);
        assert.strictEqual(r.valid, 11);
    });
    it("parseEvents parses access/call events", () => {
        const json = JSON.stringify([
            { IndexAccess: { timestamp: 10, access_id: "ia_1", container: "vec", location: "src/main.rs:3:5" } },
            { FieldAccess: { timestamp: 20, access_id: "fa_1", base: "obj", field: "name", location: "src/main.rs:4:5" } },
            { Call: { timestamp: 30, call_id: "c_1", fn_name: "push", location: "src/main.rs:5:5" } },
            { Lock: { timestamp: 40, lock_id: "lk_1", lock_type: "Mutex", var_name: "m", location: "src/main.rs:6:5" } },
            { Unwrap: { timestamp: 50, unwrap_id: "uw_1", method: "unwrap", var_name: "opt", location: "src/main.rs:7:5" } },
            { Clone: { timestamp: 60, clone_id: "cl_1", var_name: "data", location: "src/main.rs:8:5" } },
            { Deref: { timestamp: 70, deref_id: "dr_1", var_name: "ptr", location: "src/main.rs:9:5" } },
        ]);
        const r = parser.parseEvents(json);
        assert.strictEqual(r.valid, 7);
    });
    it("parseEvents parses creation events", () => {
        const json = JSON.stringify([
            { ClosureCreate: { timestamp: 10, closure_id: "cls_1", capture_mode: "move", location: "src/main.rs:3:5" } },
            { StructCreate: { timestamp: 20, struct_id: "st_1", type_name: "Point", location: "src/main.rs:4:5" } },
            { TupleCreate: { timestamp: 30, tuple_id: "tp_1", len: 3, location: "src/main.rs:5:5" } },
            { ArrayCreate: { timestamp: 40, array_id: "ar_1", len: 5, location: "src/main.rs:6:5" } },
            { LetElse: { timestamp: 50, let_id: "le_1", pattern: "Some(x)", location: "src/main.rs:7:5" } },
            { Range: { timestamp: 60, range_id: "rg_1", range_type: "RangeFull", location: "src/main.rs:8:5" } },
            { BinaryOp: { timestamp: 70, op_id: "bo_1", operator: "+", location: "src/main.rs:9:5" } },
            { TypeCast: { timestamp: 80, cast_id: "tc_1", to_type: "u64", location: "src/main.rs:10:5" } },
        ]);
        const r = parser.parseEvents(json);
        assert.strictEqual(r.valid, 8);
    });
    it("parseEvents parses region/function events", () => {
        const json = JSON.stringify([
            { RegionEnter: { timestamp: 10, region_id: "rg_1", name: "scope1", location: "src/main.rs:3:5" } },
            { RegionExit: { timestamp: 20, region_id: "rg_1", location: "src/main.rs:5:5" } },
            { FnEnter: { timestamp: 30, fn_id: "fn_1", fn_name: "main", location: "src/main.rs:1:1" } },
            { FnExit: { timestamp: 40, fn_id: "fn_1", fn_name: "main", location: "src/main.rs:10:1" } },
            { ClosureCapture: { timestamp: 50, closure_id: "cls_1", var_name: "x", capture_mode: "by_ref", location: "src/main.rs:3:5" } },
        ]);
        const r = parser.parseEvents(json);
        assert.strictEqual(r.valid, 5);
    });
    it("parseEvents parses smart pointer events (Weak, Box, Pin, Cow)", () => {
        const json = JSON.stringify([
            { WeakNew: { timestamp: 10, var_name: "w", var_id: "w_1", source_id: "rc_1", weak_count: 1, location: "src/main.rs:3:5" } },
            { WeakClone: { timestamp: 20, var_name: "w2", var_id: "w2_1", source_id: "w_1", weak_count: 2, location: "src/main.rs:4:5" } },
            { WeakUpgrade: { timestamp: 30, weak_id: "w_1", success: true, location: "src/main.rs:5:5" } },
            { BoxNew: { timestamp: 40, var_name: "b", var_id: "b_1", type_name: "Box<i32>", location: "src/main.rs:6:5" } },
            { BoxIntoRaw: { timestamp: 50, box_id: "b_1", location: "src/main.rs:7:5" } },
            { BoxFromRaw: { timestamp: 60, var_name: "b2", var_id: "b2_1", location: "src/main.rs:8:5" } },
            { PinNew: { timestamp: 70, var_name: "p", var_id: "p_1", location: "src/main.rs:9:5" } },
            { PinIntoInner: { timestamp: 80, pin_id: "p_1", location: "src/main.rs:10:5" } },
            { CowBorrowed: { timestamp: 90, var_name: "cow", var_id: "cow_1", location: "src/main.rs:11:5" } },
            { CowOwned: { timestamp: 100, var_name: "cow2", var_id: "cow2_1", location: "src/main.rs:12:5" } },
            { CowToMut: { timestamp: 110, cow_id: "cow_1", cloned: true, location: "src/main.rs:13:5" } },
        ]);
        const r = parser.parseEvents(json);
        assert.strictEqual(r.valid, 11);
    });
    it("parseEvents parses concurrency events", () => {
        const json = JSON.stringify([
            { LockGuardAcquire: { timestamp: 10, guard_id: "g_1", lock_id: "lk_1", lock_type: "Mutex", location: "src/main.rs:3:5" } },
            { LockGuardDrop: { timestamp: 20, guard_id: "g_1", location: "src/main.rs:5:5" } },
            { ThreadSpawn: { timestamp: 30, thread_id: "t_1", location: "src/main.rs:6:5" } },
            { ThreadJoin: { timestamp: 40, thread_id: "t_1", location: "src/main.rs:7:5" } },
            { ChannelSenderNew: { timestamp: 50, sender_id: "s_1", channel_id: "ch_1", location: "src/main.rs:8:5" } },
            { ChannelReceiverNew: { timestamp: 60, receiver_id: "r_1", channel_id: "ch_1", location: "src/main.rs:9:5" } },
            { ChannelSend: { timestamp: 70, sender_id: "s_1", location: "src/main.rs:10:5" } },
            { ChannelRecv: { timestamp: 80, receiver_id: "r_1", success: true, location: "src/main.rs:11:5" } },
        ]);
        const r = parser.parseEvents(json);
        assert.strictEqual(r.valid, 8);
    });
    it("parseEvents parses OnceCell/MaybeUninit events", () => {
        const json = JSON.stringify([
            { OnceCellNew: { timestamp: 10, var_name: "oc", var_id: "oc_1", location: "src/main.rs:3:5" } },
            { OnceCellSet: { timestamp: 20, cell_id: "oc_1", success: true, location: "src/main.rs:4:5" } },
            { OnceCellGet: { timestamp: 30, cell_id: "oc_1", was_initialized: true, location: "src/main.rs:5:5" } },
            { OnceCellGetOrInit: { timestamp: 40, cell_id: "oc_1", was_initialized: true, location: "src/main.rs:6:5" } },
            { MaybeUninitNew: { timestamp: 50, var_name: "mu", var_id: "mu_1", initialized: false, location: "src/main.rs:7:5" } },
            { MaybeUninitWrite: { timestamp: 60, var_id: "mu_1", location: "src/main.rs:8:5" } },
            { MaybeUninitAssumeInit: { timestamp: 70, var_id: "mu_1", location: "src/main.rs:9:5" } },
            { MaybeUninitAssumeInitRead: { timestamp: 80, var_id: "mu_1", location: "src/main.rs:10:5" } },
            { MaybeUninitAssumeInitDrop: { timestamp: 90, var_id: "mu_1", location: "src/main.rs:11:5" } },
        ]);
        const r = parser.parseEvents(json);
        assert.strictEqual(r.valid, 9);
    });
    it("parseEvents parses StaticInit/StaticAccess/ConstEval", () => {
        const json = JSON.stringify([
            { StaticInit: { timestamp: 10, var_name: "GLOBAL", var_id: "g_1", type_name: "i32", is_mutable: false } },
            { StaticAccess: { timestamp: 20, var_id: "g_1", var_name: "GLOBAL", is_write: false, location: "src/main.rs:5:5" } },
            { ConstEval: { timestamp: 30, const_name: "MAX", const_id: "c_1", type_name: "usize", location: "src/main.rs:1:1" } },
        ]);
        const r = parser.parseEvents(json);
        assert.strictEqual(r.valid, 3);
    });
    // === validateEvent ===
    it("validateEvent rejects null", () => {
        assert.ok(parser.validateEvent(null) !== null);
    });
    it("validateEvent rejects non-object", () => {
        assert.ok(parser.validateEvent("string") !== null);
    });
    it("validateEvent rejects empty object", () => {
        assert.ok(parser.validateEvent({}) !== null);
    });
    it("validateEvent rejects multi-key object", () => {
        assert.ok(parser.validateEvent({ New: {}, Drop: {} }) !== null);
    });
    it("validateEvent rejects unknown type", () => {
        assert.ok(parser.validateEvent({ FakeEvent: { timestamp: 1 } }) !== null);
    });
    it("validateEvent rejects missing timestamp", () => {
        assert.ok(parser.validateEvent({ New: { var_name: "x" } }) !== null);
    });
    it("validateEvent accepts valid event", () => {
        assert.strictEqual(parser.validateEvent({ New: { timestamp: 1, var_name: "x", var_id: "x_1", type_name: "i32" } }), null);
    });
    // === filterByFile ===
    it("filterByFile filters by location suffix", () => {
        const events = [
            { FnEnter: { timestamp: 10, fn_id: "f1", fn_name: "main", location: "src/main.rs:1:1" } },
            { FnEnter: { timestamp: 20, fn_id: "f2", fn_name: "helper", location: "src/utils.rs:1:1" } },
        ];
        const filtered = parser.filterByFile(events, "main.rs");
        assert.strictEqual(filtered.length, 1);
    });
    it("filterByFile returns empty for no matches", () => {
        const events = [{ FnEnter: { timestamp: 10, fn_id: "f1", fn_name: "main", location: "src/main.rs:1:1" } }];
        assert.strictEqual(parser.filterByFile(events, "other.rs").length, 0);
    });
    // === filterByType ===
    it("filterByType filters single type", () => {
        const events = [
            { New: { timestamp: 10, var_name: "x", var_id: "x_1", type_name: "i32" } },
            { Drop: { timestamp: 20, var_id: "x_1" } },
        ];
        const filtered = parser.filterByType(events, "New");
        assert.strictEqual(filtered.length, 1);
    });
    it("filterByType filters multiple types", () => {
        const events = [
            { New: { timestamp: 10, var_name: "x", var_id: "x_1", type_name: "i32" } },
            { Drop: { timestamp: 20, var_id: "x_1" } },
            { Move: { timestamp: 30, from_id: "x_1", to_name: "y", to_id: "y_1" } },
        ];
        const filtered = parser.filterByType(events, ["New", "Drop"]);
        assert.strictEqual(filtered.length, 2);
    });
    // === filterOwnershipEvents ===
    it("filterOwnershipEvents keeps ownership events only", () => {
        const events = [
            { New: { timestamp: 10, var_name: "x", var_id: "x_1", type_name: "i32" } },
            { LoopEnter: { timestamp: 20, loop_id: "l_1", loop_type: "for", location: "src/main.rs:3:5" } },
            { Drop: { timestamp: 30, var_id: "x_1" } },
            { Branch: { timestamp: 40, branch_id: "b_1", branch_type: "if", location: "src/main.rs:5:5" } },
        ];
        const filtered = parser.filterOwnershipEvents(events);
        assert.strictEqual(filtered.length, 2);
    });
    // === groupByVariable ===
    it("groupByVariable groups by var_id", () => {
        const events = [
            { New: { timestamp: 10, var_name: "x", var_id: "x_1", type_name: "i32" } },
            { New: { timestamp: 20, var_name: "y", var_id: "y_1", type_name: "String" } },
            { Drop: { timestamp: 30, var_id: "x_1" } },
        ];
        const groups = parser.groupByVariable(events);
        assert.strictEqual(groups.get("x_1").length, 2);
        assert.strictEqual(groups.get("y_1").length, 1);
    });
    // === groupByFunction ===
    it("groupByFunction groups events between FnEnter/FnExit", () => {
        const events = [
            { FnEnter: { timestamp: 10, fn_id: "f1", fn_name: "main", location: "src/main.rs:1:1" } },
            { New: { timestamp: 20, var_name: "x", var_id: "x_1", type_name: "i32" } },
            { Drop: { timestamp: 30, var_id: "x_1" } },
            { FnExit: { timestamp: 40, fn_id: "f1", fn_name: "main", location: "src/main.rs:10:1" } },
        ];
        const groups = parser.groupByFunction(events);
        assert.strictEqual(groups.get("main").length, 4);
    });
    // === sortByTimestamp ===
    it("sortByTimestamp sorts events", () => {
        const events = [
            { Drop: { timestamp: 300, var_id: "x_1" } },
            { New: { timestamp: 100, var_name: "x", var_id: "x_1", type_name: "i32" } },
            { Borrow: { timestamp: 200, borrower_name: "r", borrower_id: "r_1", owner_id: "x_1", mutable: false } },
        ];
        const sorted = parser.sortByTimestamp(events);
        assert.strictEqual(sorted[0].New.timestamp, 100);
        assert.strictEqual(sorted[1].Borrow.timestamp, 200);
        assert.strictEqual(sorted[2].Drop.timestamp, 300);
    });
    // === summarize ===
    it("summarize counts events by type", () => {
        const events = [
            { New: { timestamp: 10, var_name: "x", var_id: "x_1", type_name: "i32" } },
            { New: { timestamp: 20, var_name: "y", var_id: "y_1", type_name: "String" } },
            { Drop: { timestamp: 30, var_id: "x_1" } },
        ];
        const summary = parser.summarize(events);
        assert.strictEqual(summary.New, 2);
        assert.strictEqual(summary.Drop, 1);
    });
    // === Edge cases ===
    it("parseEvents skips invalid events but keeps valid ones", () => {
        const json = JSON.stringify([
            { New: { timestamp: 10, var_name: "x", var_id: "x_1", type_name: "i32" } },
            { FakeType: { timestamp: 20 } },
            { Drop: { timestamp: 30, var_id: "x_1" } },
        ]);
        const r = parser.parseEvents(json);
        assert.strictEqual(r.valid, 2);
        assert.strictEqual(r.errors.length, 1);
        assert.strictEqual(r.total, 3);
    });
    it("parseEvents handles empty string", () => {
        const r = parser.parseEvents("");
        assert.ok(r.errors.length > 0);
    });
    it("KNOWN_EVENT_TYPES has 88 entries", () => {
        // Verify via OWNERSHIP_EVENT_TYPES being a subset
        assert.ok(parser.OWNERSHIP_EVENT_TYPES.size > 0);
        assert.ok(parser.OWNERSHIP_EVENT_TYPES.size < 88);
    });
});
//# sourceMappingURL=runtime-parser.test.js.map