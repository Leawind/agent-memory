import { IdentityRow } from './caps';
type __VLS_Props = {
    /** compact（<960px）时对话框加宽到 96% */
    compact: boolean;
};
declare function openCreate(): void;
declare function openEdit(row: IdentityRow): void;
declare const _default: import('vue').DefineComponent<__VLS_Props, {
    openCreate: typeof openCreate;
    openEdit: typeof openEdit;
}, {}, {}, {}, import('vue').ComponentOptionsMixin, import('vue').ComponentOptionsMixin, {
    saved: () => any;
    created: (payload: {
        name: string;
        token: string;
    }) => any;
}, string, import('vue').PublicProps, Readonly<__VLS_Props> & Readonly<{
    onSaved?: (() => any) | undefined;
    onCreated?: ((payload: {
        name: string;
        token: string;
    }) => any) | undefined;
}>, {}, {}, {}, {}, string, import('vue').ComponentProvideOptions, false, {}, any>;
export default _default;
