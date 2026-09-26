import { IdentityRow } from './caps';
type __VLS_Props = {
    identities: IdentityRow[];
    /** compact（<960px）时创建时间列收起 */
    compact: boolean;
};
declare const _default: import('vue').DefineComponent<__VLS_Props, {}, {}, {}, {}, import('vue').ComponentOptionsMixin, import('vue').ComponentOptionsMixin, {
    edit: (row: IdentityRow) => any;
    delete: (row: IdentityRow) => any;
    "reset-token": (row: IdentityRow) => any;
}, string, import('vue').PublicProps, Readonly<__VLS_Props> & Readonly<{
    onEdit?: ((row: IdentityRow) => any) | undefined;
    onDelete?: ((row: IdentityRow) => any) | undefined;
    "onReset-token"?: ((row: IdentityRow) => any) | undefined;
}>, {}, {}, {}, {}, string, import('vue').ComponentProvideOptions, false, {}, any>;
export default _default;
