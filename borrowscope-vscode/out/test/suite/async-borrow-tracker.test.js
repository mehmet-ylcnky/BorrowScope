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
describe("12.10 Async Borrow Tracking", () => {
    let tracker;
    before(() => {
        tracker = require(path.join(ROOT, "out", "async-borrow-tracker"));
    });
    // === Basic tracking ===
    it("returns empty analysis for no events", () => {
        const result = tracker.trackAsyncBorrows([]);
        assert.strictEqual(result.total_borrows, 0);
        assert.strictEqual(result.borrows_across_await, 0);
        assert.strictEqual(result.held_borrows.length, 0);
    });
    it("counts total borrows", () => {
        const events = [
            { Borrow: { timestamp: 10, borrower_name: "r1", borrower_id: "r1_1", owner_id: "x_1", mutable: false } },
            { Borrow: { timestamp: 20, borrower_name: "r2", borrower_id: "r2_1", owner_id: "x_1", mutable: true } },
        ];
        const result = tracker.trackAsyncBorrows(events);
        assert.strictEqual(result.total_borrows, 2);
    });
    it("detects borrow held across single await", () => {
        const events = [
            { Borrow: { timestamp: 10, borrower_name: "r", borrower_id: "r_1", owner_id: "data_1", mutable: false } },
            { AwaitStart: { timestamp: 20, await_id: "aw_1", future_name: "fetch", location: "src/main.rs:5:5", live_variables: ["r"] } },
            { AwaitEnd: { timestamp: 500, await_id: "aw_1", location: "src/main.rs:5:5" } },
            { Drop: { timestamp: 600, var_id: "r_1" } },
        ];
        const result = tracker.trackAsyncBorrows(events);
        assert.strictEqual(result.borrows_across_await, 1);
        assert.strictEqual(result.held_borrows[0].borrower_name, "r");
        assert.strictEqual(result.held_borrows[0].await_points.length, 1);
        assert.strictEqual(result.held_borrows[0].await_points[0].future_name, "fetch");
        assert.strictEqual(result.held_borrows[0].await_points[0].duration_ns, 480);
    });
    it("detects borrow held across multiple awaits", () => {
        const events = [
            { Borrow: { timestamp: 10, borrower_name: "r", borrower_id: "r_1", owner_id: "data_1", mutable: false } },
            { AwaitStart: { timestamp: 20, await_id: "aw_1", future_name: "fetch1", location: "src/main.rs:5:5", live_variables: [] } },
            { AwaitEnd: { timestamp: 100, await_id: "aw_1", location: "src/main.rs:5:5" } },
            { AwaitStart: { timestamp: 110, await_id: "aw_2", future_name: "fetch2", location: "src/main.rs:6:5", live_variables: [] } },
            { AwaitEnd: { timestamp: 300, await_id: "aw_2", location: "src/main.rs:6:5" } },
            { Drop: { timestamp: 400, var_id: "r_1" } },
        ];
        const result = tracker.trackAsyncBorrows(events);
        assert.strictEqual(result.borrows_across_await, 1);
        assert.strictEqual(result.held_borrows[0].await_points.length, 2);
    });
    it("does not flag borrow released before await", () => {
        const events = [
            { Borrow: { timestamp: 10, borrower_name: "r", borrower_id: "r_1", owner_id: "data_1", mutable: false } },
            { Drop: { timestamp: 15, var_id: "r_1" } },
            { AwaitStart: { timestamp: 20, await_id: "aw_1", future_name: "fetch", location: "src/main.rs:5:5", live_variables: [] } },
            { AwaitEnd: { timestamp: 100, await_id: "aw_1", location: "src/main.rs:5:5" } },
        ];
        const result = tracker.trackAsyncBorrows(events);
        assert.strictEqual(result.borrows_across_await, 0);
    });
    it("tracks mutable borrow across await", () => {
        const events = [
            { Borrow: { timestamp: 10, borrower_name: "m", borrower_id: "m_1", owner_id: "data_1", mutable: true } },
            { AwaitStart: { timestamp: 20, await_id: "aw_1", future_name: "save", location: "src/main.rs:5:5", live_variables: [] } },
            { AwaitEnd: { timestamp: 200, await_id: "aw_1", location: "src/main.rs:5:5" } },
            { Drop: { timestamp: 300, var_id: "m_1" } },
        ];
        const result = tracker.trackAsyncBorrows(events);
        assert.strictEqual(result.held_borrows[0].mutable, true);
    });
    it("computes total_held_ns", () => {
        const events = [
            { Borrow: { timestamp: 100, borrower_name: "r", borrower_id: "r_1", owner_id: "x_1", mutable: false } },
            { AwaitStart: { timestamp: 200, await_id: "aw_1", future_name: "f", location: "src/main.rs:5:5", live_variables: [] } },
            { AwaitEnd: { timestamp: 500, await_id: "aw_1", location: "src/main.rs:5:5" } },
            { Drop: { timestamp: 800, var_id: "r_1" } },
        ];
        const result = tracker.trackAsyncBorrows(events);
        assert.strictEqual(result.held_borrows[0].total_held_ns, 700); // 800 - 100
    });
    it("tracks longest held variable", () => {
        const events = [
            { Borrow: { timestamp: 10, borrower_name: "short", borrower_id: "s_1", owner_id: "x_1", mutable: false } },
            { Borrow: { timestamp: 20, borrower_name: "long", borrower_id: "l_1", owner_id: "y_1", mutable: false } },
            { AwaitStart: { timestamp: 30, await_id: "aw_1", future_name: "f", location: "src/main.rs:5:5", live_variables: [] } },
            { AwaitEnd: { timestamp: 100, await_id: "aw_1", location: "src/main.rs:5:5" } },
            { Drop: { timestamp: 50, var_id: "s_1" } },
            { Drop: { timestamp: 500, var_id: "l_1" } },
        ];
        const result = tracker.trackAsyncBorrows(events);
        assert.strictEqual(result.longest_held_var, "long");
    });
    // === RefCellBorrow tracking ===
    it("tracks RefCellBorrow across await", () => {
        const events = [
            { RefCellBorrow: { timestamp: 10, borrow_id: "rb_1", refcell_id: "cell_1", is_mutable: true, location: "src/main.rs:3:5" } },
            { AwaitStart: { timestamp: 20, await_id: "aw_1", future_name: "process", location: "src/main.rs:4:5", live_variables: [] } },
            { AwaitEnd: { timestamp: 200, await_id: "aw_1", location: "src/main.rs:4:5" } },
            { RefCellDrop: { timestamp: 300, borrow_id: "rb_1", location: "src/main.rs:6:5" } },
        ];
        const result = tracker.trackAsyncBorrows(events);
        assert.strictEqual(result.borrows_across_await, 1);
        assert.strictEqual(result.held_borrows[0].mutable, true);
        assert.strictEqual(result.held_borrows[0].owner_id, "cell_1");
    });
    // === Function filtering ===
    it("filters to specific function", () => {
        const events = [
            { FnEnter: { timestamp: 5, fn_id: "f1", fn_name: "async_fn", location: "src/main.rs:1:1" } },
            { Borrow: { timestamp: 10, borrower_name: "r", borrower_id: "r_1", owner_id: "x_1", mutable: false } },
            { AwaitStart: { timestamp: 20, await_id: "aw_1", future_name: "f", location: "src/main.rs:3:5", live_variables: [] } },
            { AwaitEnd: { timestamp: 100, await_id: "aw_1", location: "src/main.rs:3:5" } },
            { Drop: { timestamp: 200, var_id: "r_1" } },
            { FnExit: { timestamp: 210, fn_id: "f1", fn_name: "async_fn", location: "src/main.rs:10:1" } },
            // Events outside the function
            { Borrow: { timestamp: 300, borrower_name: "other", borrower_id: "o_1", owner_id: "y_1", mutable: false } },
        ];
        const result = tracker.trackAsyncBorrows(events, "async_fn");
        assert.strictEqual(result.function_name, "async_fn");
        assert.strictEqual(result.total_borrows, 1);
        assert.strictEqual(result.borrows_across_await, 1);
    });
    // === Issue detection ===
    it("reports borrow_across_await issue", () => {
        const events = [
            { Borrow: { timestamp: 10, borrower_name: "r", borrower_id: "r_1", owner_id: "x_1", mutable: false } },
            { AwaitStart: { timestamp: 20, await_id: "aw_1", future_name: "f", location: "src/main.rs:5:5", live_variables: [] } },
            { AwaitEnd: { timestamp: 100, await_id: "aw_1", location: "src/main.rs:5:5" } },
            { Drop: { timestamp: 200, var_id: "r_1" } },
        ];
        const result = tracker.trackAsyncBorrows(events);
        assert.ok(result.issues.some((i) => i.kind === "borrow_across_await"));
        assert.ok(result.issues.find((i) => i.kind === "borrow_across_await").severity === "warning");
    });
    it("reports mut_borrow_across_await as error", () => {
        const events = [
            { Borrow: { timestamp: 10, borrower_name: "m", borrower_id: "m_1", owner_id: "x_1", mutable: true } },
            { AwaitStart: { timestamp: 20, await_id: "aw_1", future_name: "f", location: "src/main.rs:5:5", live_variables: [] } },
            { AwaitEnd: { timestamp: 100, await_id: "aw_1", location: "src/main.rs:5:5" } },
            { Drop: { timestamp: 200, var_id: "m_1" } },
        ];
        const result = tracker.trackAsyncBorrows(events);
        assert.ok(result.issues.some((i) => i.kind === "mut_borrow_across_await"));
        assert.ok(result.issues.find((i) => i.kind === "mut_borrow_across_await").severity === "error");
    });
    it("reports multiple_awaits_crossed", () => {
        const events = [
            { Borrow: { timestamp: 10, borrower_name: "r", borrower_id: "r_1", owner_id: "x_1", mutable: false } },
            { AwaitStart: { timestamp: 20, await_id: "aw_1", future_name: "f1", location: "src/main.rs:5:5", live_variables: [] } },
            { AwaitEnd: { timestamp: 50, await_id: "aw_1", location: "src/main.rs:5:5" } },
            { AwaitStart: { timestamp: 60, await_id: "aw_2", future_name: "f2", location: "src/main.rs:6:5", live_variables: [] } },
            { AwaitEnd: { timestamp: 100, await_id: "aw_2", location: "src/main.rs:6:5" } },
            { Drop: { timestamp: 200, var_id: "r_1" } },
        ];
        const result = tracker.trackAsyncBorrows(events);
        assert.ok(result.issues.some((i) => i.kind === "multiple_awaits_crossed"));
    });
    it("reports long_held_borrow (>100ms)", () => {
        const events = [
            { Borrow: { timestamp: 10, borrower_name: "r", borrower_id: "r_1", owner_id: "x_1", mutable: false } },
            { AwaitStart: { timestamp: 20, await_id: "aw_1", future_name: "slow", location: "src/main.rs:5:5", live_variables: [] } },
            { AwaitEnd: { timestamp: 200_000_000, await_id: "aw_1", location: "src/main.rs:5:5" } },
            { Drop: { timestamp: 200_000_010, var_id: "r_1" } },
        ];
        const result = tracker.trackAsyncBorrows(events);
        assert.ok(result.issues.some((i) => i.kind === "long_held_borrow"));
    });
    it("no issues when borrow released before await", () => {
        const events = [
            { Borrow: { timestamp: 10, borrower_name: "r", borrower_id: "r_1", owner_id: "x_1", mutable: false } },
            { Drop: { timestamp: 15, var_id: "r_1" } },
            { AwaitStart: { timestamp: 20, await_id: "aw_1", future_name: "f", location: "src/main.rs:5:5", live_variables: [] } },
            { AwaitEnd: { timestamp: 100, await_id: "aw_1", location: "src/main.rs:5:5" } },
        ];
        const result = tracker.trackAsyncBorrows(events);
        assert.strictEqual(result.issues.length, 0);
    });
    // === All issues have suggestions ===
    it("all issues have suggestions", () => {
        const events = [
            { Borrow: { timestamp: 10, borrower_name: "r", borrower_id: "r_1", owner_id: "x_1", mutable: true } },
            { AwaitStart: { timestamp: 20, await_id: "aw_1", future_name: "f1", location: "src/main.rs:5:5", live_variables: [] } },
            { AwaitEnd: { timestamp: 50, await_id: "aw_1", location: "src/main.rs:5:5" } },
            { AwaitStart: { timestamp: 60, await_id: "aw_2", future_name: "f2", location: "src/main.rs:6:5", live_variables: [] } },
            { AwaitEnd: { timestamp: 200_000_000, await_id: "aw_2", location: "src/main.rs:6:5" } },
            { Drop: { timestamp: 200_000_010, var_id: "r_1" } },
        ];
        const result = tracker.trackAsyncBorrows(events);
        for (const issue of result.issues) {
            assert.ok(issue.suggestion, `Issue "${issue.kind}" should have suggestion`);
            assert.ok(issue.description, `Issue "${issue.kind}" should have description`);
        }
    });
    // === Borrow still active at end ===
    it("tracks borrow never released that crossed await", () => {
        const events = [
            { Borrow: { timestamp: 10, borrower_name: "leaked", borrower_id: "lk_1", owner_id: "x_1", mutable: false } },
            { AwaitStart: { timestamp: 20, await_id: "aw_1", future_name: "f", location: "src/main.rs:5:5", live_variables: [] } },
            { AwaitEnd: { timestamp: 100, await_id: "aw_1", location: "src/main.rs:5:5" } },
            // No Drop for lk_1
        ];
        const result = tracker.trackAsyncBorrows(events);
        assert.strictEqual(result.borrows_across_await, 1);
        assert.strictEqual(result.held_borrows[0].released_timestamp, -1);
        assert.strictEqual(result.held_borrows[0].total_held_ns, -1);
    });
});
//# sourceMappingURL=async-borrow-tracker.test.js.map