type __VLS_Props = {
    visible: boolean;
    /** 编辑目标的记忆 id；null 表示新建 */
    memoryId: string | null;
    tagOptions: string[];
    /** 弹层宽度；窄容器由面板传入收窄值 */
    width?: string;
};
declare const _default: import('vue').DefineComponent<__VLS_Props, {}, {}, {}, {}, import('vue').ComponentOptionsMixin, import('vue').ComponentOptionsMixin, {
    saved: () => any;
    "update:visible": (value: boolean) => any;
}, string, import('vue').PublicProps, Readonly<__VLS_Props> & Readonly<{
    onSaved?: (() => any) | undefined;
    "onUpdate:visible"?: ((value: boolean) => any) | undefined;
}>, {}, {}, {}, {}, string, import('vue').ComponentProvideOptions, false, {}, any>;
export default _default;
