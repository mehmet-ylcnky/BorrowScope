export type RuntimeEvent = {
    New: {
        timestamp: number;
        var_name: string;
        var_id: string;
        type_name: string;
    };
} | {
    Borrow: {
        timestamp: number;
        borrower_name: string;
        borrower_id: string;
        owner_id: string;
        mutable: boolean;
    };
} | {
    Move: {
        timestamp: number;
        from_id: string;
        to_name: string;
        to_id: string;
    };
} | {
    Drop: {
        timestamp: number;
        var_id: string;
        location: string | undefined;
    };
} | {
    RcNew: {
        timestamp: number;
        var_name: string;
        var_id: string;
        type_name: string;
        strong_count: number;
        weak_count: number;
    };
} | {
    RcClone: {
        timestamp: number;
        var_name: string;
        var_id: string;
        source_id: string;
        strong_count: number;
        weak_count: number;
    };
} | {
    ArcNew: {
        timestamp: number;
        var_name: string;
        var_id: string;
        type_name: string;
        strong_count: number;
        weak_count: number;
    };
} | {
    ArcClone: {
        timestamp: number;
        var_name: string;
        var_id: string;
        source_id: string;
        strong_count: number;
        weak_count: number;
    };
} | {
    RefCellNew: {
        timestamp: number;
        var_name: string;
        var_id: string;
        type_name: string;
    };
} | {
    RefCellBorrow: {
        timestamp: number;
        borrow_id: string;
        refcell_id: string;
        is_mutable: boolean;
        location: string;
    };
} | {
    RefCellDrop: {
        timestamp: number;
        borrow_id: string;
        location: string;
    };
} | {
    CellNew: {
        timestamp: number;
        var_name: string;
        var_id: string;
        type_name: string;
    };
} | {
    CellGet: {
        timestamp: number;
        cell_id: string;
        location: string;
    };
} | {
    CellSet: {
        timestamp: number;
        cell_id: string;
        location: string;
    };
} | {
    StaticInit: {
        timestamp: number;
        var_name: string;
        var_id: string;
        type_name: string;
        is_mutable: boolean;
    };
} | {
    StaticAccess: {
        timestamp: number;
        var_id: string;
        var_name: string;
        is_write: boolean;
        location: string;
    };
} | {
    ConstEval: {
        timestamp: number;
        const_name: string;
        const_id: string;
        type_name: string;
        location: string;
    };
} | {
    RawPtrCreated: {
        timestamp: number;
        var_name: string;
        var_id: string;
        ptr_type: string;
        address: number;
        location: string;
    };
} | {
    RawPtrDeref: {
        timestamp: number;
        ptr_id: string;
        location: string;
        is_write: boolean;
    };
} | {
    UnsafeBlockEnter: {
        timestamp: number;
        block_id: string;
        location: string;
        operation_kind: string | undefined;
        operation_context: string | undefined;
    };
} | {
    UnsafeBlockExit: {
        timestamp: number;
        block_id: string;
        location: string;
    };
} | {
    UnsafeFnCall: {
        timestamp: number;
        fn_name: string;
        location: string;
    };
} | {
    FfiCall: {
        timestamp: number;
        fn_name: string;
        location: string;
    };
} | {
    Transmute: {
        timestamp: number;
        from_type: string;
        to_type: string;
        location: string;
    };
} | {
    UnionFieldAccess: {
        timestamp: number;
        union_name: string;
        field_name: string;
        location: string;
    };
} | {
    AsyncBlockEnter: {
        timestamp: number;
        block_id: string;
        location: string;
    };
} | {
    AsyncBlockExit: {
        timestamp: number;
        block_id: string;
        location: string;
    };
} | {
    AwaitStart: {
        timestamp: number;
        await_id: string;
        future_name: string;
        location: string;
        live_variables: string[];
    };
} | {
    AwaitEnd: {
        timestamp: number;
        await_id: string;
        location: string;
    };
} | {
    LoopEnter: {
        timestamp: number;
        loop_id: string;
        loop_type: string;
        location: string;
    };
} | {
    LoopIteration: {
        timestamp: number;
        loop_id: string;
        iteration: number;
        location: string;
    };
} | {
    LoopExit: {
        timestamp: number;
        loop_id: string;
        location: string;
    };
} | {
    MatchEnter: {
        timestamp: number;
        match_id: string;
        location: string;
    };
} | {
    MatchArm: {
        timestamp: number;
        match_id: string;
        arm_index: number;
        pattern: string;
        location: string;
        bindings: string[];
    };
} | {
    MatchExit: {
        timestamp: number;
        match_id: string;
        location: string;
    };
} | {
    Branch: {
        timestamp: number;
        branch_id: string;
        branch_type: string;
        location: string;
    };
} | {
    Return: {
        timestamp: number;
        return_id: string;
        has_value: boolean;
        location: string;
    };
} | {
    Try: {
        timestamp: number;
        try_id: string;
        location: string;
    };
} | {
    IndexAccess: {
        timestamp: number;
        access_id: string;
        container: string;
        location: string;
    };
} | {
    FieldAccess: {
        timestamp: number;
        access_id: string;
        base: string;
        field: string;
        location: string;
    };
} | {
    Call: {
        timestamp: number;
        call_id: string;
        fn_name: string;
        location: string;
        receiver_type: string | undefined;
        result_type: string | undefined;
    };
} | {
    Lock: {
        timestamp: number;
        lock_id: string;
        lock_type: string;
        var_name: string;
        location: string;
    };
} | {
    Unwrap: {
        timestamp: number;
        unwrap_id: string;
        method: string;
        var_name: string;
        location: string;
    };
} | {
    Clone: {
        timestamp: number;
        clone_id: string;
        var_name: string;
        location: string;
    };
} | {
    Deref: {
        timestamp: number;
        deref_id: string;
        var_name: string;
        location: string;
    };
} | {
    Break: {
        timestamp: number;
        break_id: string;
        loop_label: string | undefined;
        location: string;
    };
} | {
    Continue: {
        timestamp: number;
        continue_id: string;
        loop_label: string | undefined;
        location: string;
    };
} | {
    ClosureCreate: {
        timestamp: number;
        closure_id: string;
        capture_mode: string;
        location: string;
        fn_trait: string | undefined;
    };
} | {
    StructCreate: {
        timestamp: number;
        struct_id: string;
        type_name: string;
        location: string;
    };
} | {
    TupleCreate: {
        timestamp: number;
        tuple_id: string;
        len: number;
        location: string;
    };
} | {
    LetElse: {
        timestamp: number;
        let_id: string;
        pattern: string;
        location: string;
    };
} | {
    Range: {
        timestamp: number;
        range_id: string;
        range_type: string;
        location: string;
    };
} | {
    BinaryOp: {
        timestamp: number;
        op_id: string;
        operator: string;
        location: string;
    };
} | {
    ArrayCreate: {
        timestamp: number;
        array_id: string;
        len: number;
        location: string;
    };
} | {
    TypeCast: {
        timestamp: number;
        cast_id: string;
        to_type: string;
        location: string;
    };
} | {
    RegionEnter: {
        timestamp: number;
        region_id: string;
        name: string;
        location: string;
    };
} | {
    RegionExit: {
        timestamp: number;
        region_id: string;
        location: string;
    };
} | {
    FnEnter: {
        timestamp: number;
        fn_id: string;
        fn_name: string;
        location: string;
    };
} | {
    FnExit: {
        timestamp: number;
        fn_id: string;
        fn_name: string;
        location: string;
    };
} | {
    ClosureCapture: {
        timestamp: number;
        closure_id: string;
        var_name: string;
        capture_mode: string;
        location: string;
    };
} | {
    WeakNew: {
        timestamp: number;
        var_name: string;
        var_id: string;
        source_id: string;
        weak_count: number;
        location: string;
    };
} | {
    WeakClone: {
        timestamp: number;
        var_name: string;
        var_id: string;
        source_id: string;
        weak_count: number;
        location: string;
    };
} | {
    WeakUpgrade: {
        timestamp: number;
        weak_id: string;
        success: boolean;
        location: string;
    };
} | {
    BoxNew: {
        timestamp: number;
        var_name: string;
        var_id: string;
        type_name: string;
        location: string;
    };
} | {
    BoxIntoRaw: {
        timestamp: number;
        box_id: string;
        location: string;
    };
} | {
    BoxFromRaw: {
        timestamp: number;
        var_name: string;
        var_id: string;
        location: string;
    };
} | {
    LockGuardAcquire: {
        timestamp: number;
        guard_id: string;
        lock_id: string;
        lock_type: string;
        location: string;
    };
} | {
    LockGuardDrop: {
        timestamp: number;
        guard_id: string;
        location: string;
    };
} | {
    PinNew: {
        timestamp: number;
        var_name: string;
        var_id: string;
        location: string;
    };
} | {
    PinIntoInner: {
        timestamp: number;
        pin_id: string;
        location: string;
    };
} | {
    CowBorrowed: {
        timestamp: number;
        var_name: string;
        var_id: string;
        location: string;
    };
} | {
    CowOwned: {
        timestamp: number;
        var_name: string;
        var_id: string;
        location: string;
    };
} | {
    CowToMut: {
        timestamp: number;
        cow_id: string;
        cloned: boolean;
        location: string;
    };
} | {
    ThreadSpawn: {
        timestamp: number;
        thread_id: string;
        location: string;
    };
} | {
    ThreadJoin: {
        timestamp: number;
        thread_id: string;
        location: string;
    };
} | {
    ChannelSenderNew: {
        timestamp: number;
        sender_id: string;
        channel_id: string;
        location: string;
    };
} | {
    ChannelReceiverNew: {
        timestamp: number;
        receiver_id: string;
        channel_id: string;
        location: string;
    };
} | {
    ChannelSend: {
        timestamp: number;
        sender_id: string;
        location: string;
    };
} | {
    ChannelRecv: {
        timestamp: number;
        receiver_id: string;
        success: boolean;
        location: string;
    };
} | {
    OnceCellNew: {
        timestamp: number;
        var_name: string;
        var_id: string;
        cell_type: any;
        location: string;
    };
} | {
    OnceCellSet: {
        timestamp: number;
        cell_id: string;
        success: boolean;
        location: string;
    };
} | {
    OnceCellGet: {
        timestamp: number;
        cell_id: string;
        was_initialized: boolean;
        location: string;
    };
} | {
    OnceCellGetOrInit: {
        timestamp: number;
        cell_id: string;
        was_initialized: boolean;
        location: string;
    };
} | {
    MaybeUninitNew: {
        timestamp: number;
        var_name: string;
        var_id: string;
        initialized: boolean;
        location: string;
    };
} | {
    MaybeUninitWrite: {
        timestamp: number;
        var_id: string;
        location: string;
    };
} | {
    MaybeUninitAssumeInit: {
        timestamp: number;
        var_id: string;
        location: string;
    };
} | {
    MaybeUninitAssumeInitRead: {
        timestamp: number;
        var_id: string;
        location: string;
    };
} | {
    MaybeUninitAssumeInitDrop: {
        timestamp: number;
        var_id: string;
        location: string;
    };
};
/** Get the event type name (handles both internally and externally tagged) */
export declare function eventType(event: RuntimeEvent): string;
/** Get the event payload (handles both internally and externally tagged) */
export declare function eventData(event: RuntimeEvent): {
    timestamp: number;
    [key: string]: any;
};
