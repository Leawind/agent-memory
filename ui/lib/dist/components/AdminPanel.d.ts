import { WhoAmI } from '../types';
type __VLS_Props = {
    /** 调用者身份摘要：决定面板是否可用（admin 能力）与开放模式提示；嵌入宿主未接入时传 null */
    who?: WhoAmI | null;
    /** 隐藏标题/副标题区（嵌入宿主已有页面标题时只要内容卡） */
    showHeader?: boolean;
    /** 覆盖默认标题 */
    title?: string;
    /** 覆盖默认副标题 */
    subtitle?: string;
};
declare const _default: import('vue').DefineComponent<__VLS_Props, {
    refresh: () => void;
}, {}, {}, {}, import('vue').ComponentOptionsMixin, import('vue').ComponentOptionsMixin, {}, string, import('vue').PublicProps, Readonly<__VLS_Props> & Readonly<{}>, {
    showHeader: boolean;
    who: WhoAmI | null;
}, {}, {}, {}, string, import('vue').ComponentProvideOptions, false, {
    rootRef: HTMLDivElement;
    importInput: HTMLInputElement;
}, HTMLDivElement>;
export default _default;
