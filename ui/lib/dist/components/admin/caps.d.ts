export declare const CAPS: readonly [{
    readonly key: "read";
    readonly labelKey: "capRead";
}, {
    readonly key: "create";
    readonly labelKey: "capCreate";
}, {
    readonly key: "update";
    readonly labelKey: "capUpdate";
}, {
    readonly key: "delete";
    readonly labelKey: "capDelete";
}, {
    readonly key: "tag_manage";
    readonly labelKey: "capTagManage";
}, {
    readonly key: "admin";
    readonly labelKey: "capAdmin";
}];
export interface IdentityRow {
    name: string;
    token_hint: string;
    permissions: Record<string, boolean>;
    created_at: number;
}
export declare function emptyCaps(): Record<string, boolean>;
export declare const PRESETS: Record<string, string[]>;
export declare function capLabel(key: string): string;
export declare function enabledCaps(row: IdentityRow): string[];
export declare function maskToken(hint: string): string;
/** 面板层统一的错误包装：动作失败 toast 并返回 undefined，成功可选 toast。 */
export declare function run<T>(action: () => Promise<T>, successMsg?: string): Promise<T | undefined>;
