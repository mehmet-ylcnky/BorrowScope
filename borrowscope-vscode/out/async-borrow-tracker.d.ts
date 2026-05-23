import { RuntimeEvent } from "./runtime-types";
import { SourceLocation } from "./runtime-mapper";
/** A borrow that was active during an await */
export interface AsyncBorrowHeld {
    borrower_id: string;
    borrower_name: string;
    owner_id: string;
    mutable: boolean;
    borrow_timestamp: number;
    borrow_location: SourceLocation | null;
    await_points: AwaitPoint[];
    total_held_ns: number;
    released_timestamp: number;
}
/** An await point crossed while a borrow was active */
export interface AwaitPoint {
    await_id: string;
    future_name: string;
    location: SourceLocation | null;
    start_timestamp: number;
    end_timestamp: number;
    duration_ns: number;
    poll_count: number;
    live_variables: string[];
}
/** Summary of async borrow issues in a function */
export interface AsyncBorrowAnalysis {
    function_name: string | null;
    total_borrows: number;
    borrows_across_await: number;
    longest_held_ns: number;
    longest_held_var: string | null;
    held_borrows: AsyncBorrowHeld[];
    issues: AsyncBorrowIssue[];
}
/** An issue detected with async borrows */
export interface AsyncBorrowIssue {
    kind: "borrow_across_await" | "mut_borrow_across_await" | "long_held_borrow" | "multiple_awaits_crossed";
    severity: "info" | "warning" | "error";
    borrower: string;
    owner: string;
    description: string;
    suggestion: string;
}
/** Track borrows held across await points */
export declare function trackAsyncBorrows(events: RuntimeEvent[], functionName?: string): AsyncBorrowAnalysis;
