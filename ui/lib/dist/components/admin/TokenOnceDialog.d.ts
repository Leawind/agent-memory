type __VLS_Props = {
    /** compact（<960px）时对话框加宽到 96% */
    compact: boolean;
};
/** 展示一次性 token（供父层在创建/重置成功后调用）。 */
declare function show(newTitle: string, name: string, newToken: string): void;
declare const _default: import('vue').DefineComponent<__VLS_Props, {
    show: typeof show;
}, {}, {}, {}, import('vue').ComponentOptionsMixin, import('vue').ComponentOptionsMixin, {}, string, import('vue').PublicProps, Readonly<__VLS_Props> & Readonly<{}>, {}, {}, {}, {}, string, import('vue').ComponentProvideOptions, false, {}, any>;
export default _default;
