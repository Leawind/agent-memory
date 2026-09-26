type __VLS_Props = {
    /** 服务器当前的鉴权开关状态（随父层 load 刷新） */
    authRequired: boolean;
    /** 库里是否存在 admin 身份（开启的前置条件） */
    hasAdminIdentity: boolean;
};
declare const _default: import('vue').DefineComponent<__VLS_Props, {}, {}, {}, {}, import('vue').ComponentOptionsMixin, import('vue').ComponentOptionsMixin, {}, string, import('vue').PublicProps, Readonly<__VLS_Props> & Readonly<{}>, {}, {}, {}, {}, string, import('vue').ComponentProvideOptions, false, {}, any>;
export default _default;
