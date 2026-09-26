import { InjectionKey } from 'vue';
import { MemoryUILocaleOption } from './i18n';
export interface MemoryUIConfig {
    /** API 前缀。默认 ''（与服务器同源部署）；跨系统集成时填服务器地址，如 'http://127.0.0.1:8899' */
    baseUrl?: string;
    /** 自定义 fetch（注入鉴权头、错误拦截等）。默认 globalThis.fetch */
    fetch?: typeof fetch;
    /** 列表默认分页大小。默认 20 */
    defaultPageSize?: number;
    /** 界面语言。'auto' 跟随浏览器（默认）；也可固定 'zh' / 'en'，运行时可用 setMemoryUILocale 切换 */
    locale?: MemoryUILocaleOption;
    /** 管理面板创建/重置身份后的 token 一次性展示弹窗里，「保存到本浏览器」按钮的落点。
     *  注入后按钮才渲染；独立站点壳用它把 token 存入自己的多身份令牌表。 */
    onIdentityToken?: (name: string, token: string) => void;
    /** 鉴权开关切换成功后回调（独立站点壳借此重新解析 whoami，右上角身份区即时反映 token 模式）。 */
    onAuthChanged?: (enabled: boolean) => void;
}
export type ResolvedMemoryUIConfig = Omit<Required<MemoryUIConfig>, 'onIdentityToken' | 'onAuthChanged'> & Pick<MemoryUIConfig, 'onIdentityToken' | 'onAuthChanged'>;
export declare const MemoryUIConfigKey: InjectionKey<MemoryUIConfig>;
/** 在组件 setup 中注入配置（也可直接用 app.provide(MemoryUIConfigKey, config)）。 */
export declare function provideMemoryUI(config: MemoryUIConfig): void;
/** 读取解析后的完整配置；必须在组件 setup 中调用。 */
export declare function useMemoryConfig(): ResolvedMemoryUIConfig;
