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
describe("12.5 Runtime Timing Decorations", () => {
    let decs;
    before(() => {
        decs = require(path.join(ROOT, "out", "runtime-decorations"));
    });
    // === formatDuration ===
    it("formatDuration: nanoseconds", () => {
        assert.strictEqual(decs.formatDuration(500), "500ns");
    });
    it("formatDuration: microseconds", () => {
        assert.strictEqual(decs.formatDuration(1500), "1.5μs");
    });
    it("formatDuration: milliseconds", () => {
        assert.strictEqual(decs.formatDuration(2_500_000), "2.5ms");
    });
    it("formatDuration: seconds", () => {
        assert.strictEqual(decs.formatDuration(1_500_000_000), "1.50s");
    });
    it("formatDuration: negative returns ∞", () => {
        assert.strictEqual(decs.formatDuration(-1), "∞");
    });
    it("formatDuration: zero", () => {
        assert.strictEqual(decs.formatDuration(0), "0ns");
    });
    // === buildTimingDecorations ===
    it("buildTimingDecorations returns array", () => {
        const merged = [{
                name: "x", var_id: "x_1", line: 3, file: "src/main.rs",
                static_info: { type_display: "i32", ownership_category: "Owned", is_copy: true },
                runtime_info: { actual_lifetime_ns: 5000, actual_borrow_count: 0, actual_mut_borrow_count: 0, was_actually_moved: false, move_destination: null, drop_order: 0, drop_timestamp: 5010, ref_count_peak: 0, ref_count_final: 0, weak_count_peak: 0, clone_count: 0, thread_id: null, await_crossings: [], unsafe_accesses: 0, event_count: 2 },
                agreement: "match", divergences: [],
            }];
        const result = decs.buildTimingDecorations(merged);
        assert.ok(Array.isArray(result));
        assert.strictEqual(result.length, 1);
    });
    it("buildTimingDecorations includes borrow count in text", () => {
        const merged = [{
                name: "data", var_id: "d_1", line: 5, file: "src/main.rs",
                static_info: { type_display: "Vec<i32>", ownership_category: "Owned", is_copy: false },
                runtime_info: { actual_lifetime_ns: 10000, actual_borrow_count: 3, actual_mut_borrow_count: 1, was_actually_moved: false, move_destination: null, drop_order: 0, drop_timestamp: 10010, ref_count_peak: 0, ref_count_final: 0, weak_count_peak: 0, clone_count: 0, thread_id: null, await_crossings: [], unsafe_accesses: 0, event_count: 5 },
                agreement: "match", divergences: [],
            }];
        const result = decs.buildTimingDecorations(merged);
        assert.ok(result[0].renderOptions.after.contentText.includes("3×&"));
    });
    it("buildTimingDecorations includes ref count peak for Rc", () => {
        const merged = [{
                name: "rc", var_id: "rc_1", line: 7, file: "src/main.rs",
                static_info: { type_display: "Rc<i32>", ownership_category: "Rc", is_copy: false },
                runtime_info: { actual_lifetime_ns: 8000, actual_borrow_count: 0, actual_mut_borrow_count: 0, was_actually_moved: false, move_destination: null, drop_order: 1, drop_timestamp: 8010, ref_count_peak: 4, ref_count_final: 0, weak_count_peak: 0, clone_count: 3, thread_id: null, await_crossings: [], unsafe_accesses: 0, event_count: 5 },
                agreement: "match", divergences: [],
            }];
        const result = decs.buildTimingDecorations(merged);
        assert.ok(result[0].renderOptions.after.contentText.includes("refs↑4"));
    });
    it("buildTimingDecorations skips static_only vars", () => {
        const merged = [{
                name: "unused", var_id: null, line: 20, file: "src/main.rs",
                static_info: { type_display: "i32", ownership_category: "Owned", is_copy: true },
                runtime_info: null,
                agreement: "static_only", divergences: [],
            }];
        const result = decs.buildTimingDecorations(merged);
        assert.strictEqual(result.length, 0);
    });
    it("buildTimingDecorations skips vars with line <= 0", () => {
        const merged = [{
                name: "x", var_id: "x_1", line: 0, file: null,
                static_info: null,
                runtime_info: { actual_lifetime_ns: 100, actual_borrow_count: 0, actual_mut_borrow_count: 0, was_actually_moved: false, move_destination: null, drop_order: -1, drop_timestamp: -1, ref_count_peak: 0, ref_count_final: 0, weak_count_peak: 0, clone_count: 0, thread_id: null, await_crossings: [], unsafe_accesses: 0, event_count: 1 },
                agreement: "runtime_only", divergences: [],
            }];
        const result = decs.buildTimingDecorations(merged);
        assert.strictEqual(result.length, 0);
    });
    it("buildTimingDecorations has green color", () => {
        const merged = [{
                name: "x", var_id: "x_1", line: 3, file: "src/main.rs",
                static_info: { type_display: "i32", ownership_category: "Owned", is_copy: true },
                runtime_info: { actual_lifetime_ns: 100, actual_borrow_count: 0, actual_mut_borrow_count: 0, was_actually_moved: false, move_destination: null, drop_order: 0, drop_timestamp: 110, ref_count_peak: 0, ref_count_final: 0, weak_count_peak: 0, clone_count: 0, thread_id: null, await_crossings: [], unsafe_accesses: 0, event_count: 2 },
                agreement: "match", divergences: [],
            }];
        const result = decs.buildTimingDecorations(merged);
        assert.ok(result[0].renderOptions.after.color.includes("46, 204, 113"));
    });
    // === buildDivergenceDecorations ===
    it("buildDivergenceDecorations returns decorations for diverging vars", () => {
        const merged = [{
                name: "rc", var_id: "rc_1", line: 7, file: "src/main.rs",
                static_info: { type_display: "Rc<i32>", ownership_category: "Rc", is_copy: false },
                runtime_info: { actual_lifetime_ns: -1, actual_borrow_count: 0, actual_mut_borrow_count: 0, was_actually_moved: false, move_destination: null, drop_order: -1, drop_timestamp: -1, ref_count_peak: 2, ref_count_final: 2, weak_count_peak: 0, clone_count: 1, thread_id: null, await_crossings: [], unsafe_accesses: 0, event_count: 2 },
                agreement: "diverge", divergences: [{ kind: "rc_leak", description: "Rc never dropped (final ref count: 2)" }],
            }];
        const result = decs.buildDivergenceDecorations(merged);
        assert.strictEqual(result.length, 1);
        assert.ok(result[0].renderOptions.after.contentText.includes("⚡"));
        assert.ok(result[0].renderOptions.after.color.includes("e74c3c"));
    });
    it("buildDivergenceDecorations skips non-diverging vars", () => {
        const merged = [{
                name: "x", var_id: "x_1", line: 3, file: "src/main.rs",
                static_info: { type_display: "i32", ownership_category: "Owned", is_copy: true },
                runtime_info: { actual_lifetime_ns: 100, actual_borrow_count: 0, actual_mut_borrow_count: 0, was_actually_moved: false, move_destination: null, drop_order: 0, drop_timestamp: 110, ref_count_peak: 0, ref_count_final: 0, weak_count_peak: 0, clone_count: 0, thread_id: null, await_crossings: [], unsafe_accesses: 0, event_count: 2 },
                agreement: "match", divergences: [],
            }];
        const result = decs.buildDivergenceDecorations(merged);
        assert.strictEqual(result.length, 0);
    });
    // === buildDropOrderDecorations ===
    it("buildDropOrderDecorations shows drop order numbers", () => {
        const merged = [
            { name: "a", var_id: "a_1", line: 3, file: "src/main.rs", static_info: null, runtime_info: { actual_lifetime_ns: 100, actual_borrow_count: 0, actual_mut_borrow_count: 0, was_actually_moved: false, move_destination: null, drop_order: 2, drop_timestamp: 300, ref_count_peak: 0, ref_count_final: 0, weak_count_peak: 0, clone_count: 0, thread_id: null, await_crossings: [], unsafe_accesses: 0, event_count: 2 }, agreement: "match", divergences: [] },
            { name: "b", var_id: "b_1", line: 4, file: "src/main.rs", static_info: null, runtime_info: { actual_lifetime_ns: 100, actual_borrow_count: 0, actual_mut_borrow_count: 0, was_actually_moved: false, move_destination: null, drop_order: 1, drop_timestamp: 200, ref_count_peak: 0, ref_count_final: 0, weak_count_peak: 0, clone_count: 0, thread_id: null, await_crossings: [], unsafe_accesses: 0, event_count: 2 }, agreement: "match", divergences: [] },
        ];
        // showDropOrder defaults to false, so should return empty
        const result = decs.buildDropOrderDecorations(merged);
        assert.strictEqual(result.length, 0); // disabled by default
    });
    // === buildRefCountDecorations ===
    it("buildRefCountDecorations shows peak for Rc/Arc", () => {
        const merged = [{
                name: "rc", var_id: "rc_1", line: 7, file: "src/main.rs",
                static_info: { type_display: "Rc<i32>", ownership_category: "Rc", is_copy: false },
                runtime_info: { actual_lifetime_ns: 8000, actual_borrow_count: 0, actual_mut_borrow_count: 0, was_actually_moved: false, move_destination: null, drop_order: 0, drop_timestamp: 8010, ref_count_peak: 5, ref_count_final: 0, weak_count_peak: 2, clone_count: 4, thread_id: null, await_crossings: [], unsafe_accesses: 0, event_count: 6 },
                agreement: "match", divergences: [],
            }];
        const result = decs.buildRefCountDecorations(merged);
        assert.strictEqual(result.length, 1);
        assert.ok(result[0].renderOptions.after.contentText.includes("peak:5"));
        assert.ok(result[0].renderOptions.after.contentText.includes("4 clones"));
        assert.ok(result[0].renderOptions.after.contentText.includes("weak:2"));
    });
    it("buildRefCountDecorations skips non-Rc/Arc vars", () => {
        const merged = [{
                name: "x", var_id: "x_1", line: 3, file: "src/main.rs",
                static_info: { type_display: "i32", ownership_category: "Owned", is_copy: true },
                runtime_info: { actual_lifetime_ns: 100, actual_borrow_count: 0, actual_mut_borrow_count: 0, was_actually_moved: false, move_destination: null, drop_order: 0, drop_timestamp: 110, ref_count_peak: 0, ref_count_final: 0, weak_count_peak: 0, clone_count: 0, thread_id: null, await_crossings: [], unsafe_accesses: 0, event_count: 2 },
                agreement: "match", divergences: [],
            }];
        const result = decs.buildRefCountDecorations(merged);
        assert.strictEqual(result.length, 0);
    });
    // === createRuntimeDecorationTypes ===
    it("createRuntimeDecorationTypes returns 4 types", () => {
        const types = decs.createRuntimeDecorationTypes();
        assert.ok(types.timing);
        assert.ok(types.divergence);
        assert.ok(types.dropOrder);
        assert.ok(types.refCount);
    });
    // === applyRuntimeDecorations / clearRuntimeDecorations ===
    it("applyRuntimeDecorations is a function", () => {
        assert.strictEqual(typeof decs.applyRuntimeDecorations, "function");
    });
    it("clearRuntimeDecorations is a function", () => {
        assert.strictEqual(typeof decs.clearRuntimeDecorations, "function");
    });
    // === Hover content ===
    it("timing decoration has hoverMessage", () => {
        const merged = [{
                name: "data", var_id: "d_1", line: 5, file: "src/main.rs",
                static_info: { type_display: "Vec<i32>", ownership_category: "Owned", is_copy: false },
                runtime_info: { actual_lifetime_ns: 50000, actual_borrow_count: 2, actual_mut_borrow_count: 1, was_actually_moved: true, move_destination: "other", drop_order: 0, drop_timestamp: 50010, ref_count_peak: 0, ref_count_final: 0, weak_count_peak: 0, clone_count: 0, thread_id: null, await_crossings: [{ await_line: 8, duration_ns: 1000, future_name: "fetch" }], unsafe_accesses: 0, event_count: 5 },
                agreement: "match", divergences: [],
            }];
        const result = decs.buildTimingDecorations(merged);
        assert.ok(result[0].hoverMessage);
        assert.ok(result[0].hoverMessage.value.includes("data"));
        assert.ok(result[0].hoverMessage.value.includes("Lifetime"));
    });
    it("divergence decoration has hoverMessage with reason", () => {
        const merged = [{
                name: "rc", var_id: "rc_1", line: 7, file: "src/main.rs",
                static_info: { type_display: "Rc<i32>", ownership_category: "Rc", is_copy: false },
                runtime_info: { actual_lifetime_ns: -1, actual_borrow_count: 0, actual_mut_borrow_count: 0, was_actually_moved: false, move_destination: null, drop_order: -1, drop_timestamp: -1, ref_count_peak: 2, ref_count_final: 2, weak_count_peak: 0, clone_count: 1, thread_id: null, await_crossings: [], unsafe_accesses: 0, event_count: 2 },
                agreement: "diverge", divergences: [{ kind: "rc_leak", description: "Rc never dropped" }],
            }];
        const result = decs.buildDivergenceDecorations(merged);
        assert.ok(result[0].hoverMessage);
        assert.ok(result[0].hoverMessage.value.includes("Divergence"));
    });
});
//# sourceMappingURL=runtime-decorations.test.js.map