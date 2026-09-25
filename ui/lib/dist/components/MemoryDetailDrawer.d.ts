type __VLS_Props = {
    visible: boolean;
    memoryId: string | null;
    /** 抽屉宽度；窄容器由面板传入 '100%' */
    size?: string;
};
declare const _default: import('vue').DefineComponent<__VLS_Props, {}, {}, {}, {}, import('vue').ComponentOptionsMixin, import('vue').ComponentOptionsMixin, {
    "update:visible": (value: boolean) => any;
}, string, import('vue').PublicProps, Readonly<__VLS_Props> & Readonly<{
    "onUpdate:visible"?: ((value: boolean) => any) | undefined;
}>, {}, {}, {}, {}, string, import('vue').ComponentProvideOptions, false, {}, any>;
export default _default;
