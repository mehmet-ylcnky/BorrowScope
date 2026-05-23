export interface OwnershipDiff {
    addedVariables: string[];
    removedVariables: string[];
    addedBorrows: string[];
    removedBorrows: string[];
    addedMoves: string[];
    removedMoves: string[];
    summary: string;
    hasChanges: boolean;
}
export declare function computeOwnershipDiff(before: any, after: any): OwnershipDiff;
