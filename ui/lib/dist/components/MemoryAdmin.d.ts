/**
 * 管理台根组件。layout:
 * - 'sidebar'：左侧固定导航，适合独立全屏站点（默认）
 * - 'tabs'：顶部页签导航，适合嵌入宿主系统页面
 */
type __VLS_Props = {
    layout?: 'sidebar' | 'tabs';
    title?: string;
};
declare function __VLS_template(): {
    attrs: Partial<{}>;
    slots: {
        footer?(_: {}): any;
    };
    refs: {};
    rootEl: HTMLDivElement;
};
type __VLS_TemplateResult = ReturnType<typeof __VLS_template>;
declare const __VLS_component: import('vue').DefineComponent<__VLS_Props, {}, {}, {}, {}, import('vue').ComponentOptionsMixin, import('vue').ComponentOptionsMixin, {}, string, import('vue').PublicProps, Readonly<__VLS_Props> & Readonly<{}>, {
    title: string;
    layout: "sidebar" | "tabs";
}, {}, {}, {}, string, import('vue').ComponentProvideOptions, false, {}, HTMLDivElement>;
declare const _default: __VLS_WithTemplateSlots<typeof __VLS_component, __VLS_TemplateResult["slots"]>;
export default _default;
type __VLS_WithTemplateSlots<T, S> = T & {
    new (): {
        $slots: S;
    };
};
