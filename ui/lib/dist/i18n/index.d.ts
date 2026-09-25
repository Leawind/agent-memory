export type MemoryUILocale = 'zh' | 'en';
/** locale 配置取值：固定语言或跟随浏览器 */
export type MemoryUILocaleOption = MemoryUILocale | 'auto';
export declare const memoryUIi18n: import('vue-i18n').I18n<{
    zh: {
        nav: {
            memories: string;
            tags: string;
            ops: string;
        };
        common: {
            detail: string;
            edit: string;
            delete: string;
            cancel: string;
            save: string;
        };
        admin: {
            hint: string;
        };
        memories: {
            title: string;
            subtitle: string;
            create: string;
            searchPlaceholder: string;
            tagFilter: string;
            sortUpdated: string;
            sortCreated: string;
            orderDesc: string;
            orderAsc: string;
            colId: string;
            colSummary: string;
            colTags: string;
            colScore: string;
            colCreatedAt: string;
            colUpdatedAt: string;
            colActions: string;
            deleteTitle: string;
            deleteConfirm: string;
            deleted: string;
        };
        tags: {
            title: string;
            subtitle: string;
            create: string;
            colName: string;
            colDescription: string;
            colMemoryCount: string;
            colLastUsed: string;
            editTitle: string;
            createTitle: string;
            nameLabel: string;
            namePlaceholder: string;
            renameLabel: string;
            renamePlaceholder: string;
            descLabel: string;
            descPlaceholder: string;
            deleteTitle: string;
            deleteBefore: string;
            deleteMiddle: string;
            deleteAfter: string;
            detach: string;
            purge: string;
            purgeConfirmTitle: string;
            purgeConfirm: string;
            purgeButton: string;
            saved: string;
            created: string;
            deleted: string;
        };
        ops: {
            title: string;
            subtitle: string;
            refresh: string;
            statMemories: string;
            statTags: string;
            statSize: string;
            statNextId: string;
            dbCard: string;
            path: string;
            lastUpdate: string;
            lastUpdateValue: string;
            crossPlatform: string;
            crossPlatformNote: string;
            version: string;
            export: string;
            import: string;
            importHint: string;
            doctorCard: string;
            runDoctor: string;
            doctorOk: string;
            doctorFail: string;
            doctorEmpty: string;
            imported: string;
        };
        drawer: {
            title: string;
            rendered: string;
            source: string;
        };
        editor: {
            editTitle: string;
            createTitle: string;
            summaryLabel: string;
            summaryPlaceholder: string;
            contentLabel: string;
            tabEdit: string;
            tabPreview: string;
            contentPlaceholder: string;
            tagsLabel: string;
            tagsPlaceholder: string;
            updated: string;
            created: string;
        };
        errors: {
            http: string;
            invalidBackup: string;
        };
        shell: {
            theme: string;
            themeLight: string;
            themeDark: string;
            themeSystem: string;
            language: string;
        };
    };
    en: {
        nav: {
            memories: string;
            tags: string;
            ops: string;
        };
        common: {
            detail: string;
            edit: string;
            delete: string;
            cancel: string;
            save: string;
        };
        admin: {
            hint: string;
        };
        memories: {
            title: string;
            subtitle: string;
            create: string;
            searchPlaceholder: string;
            tagFilter: string;
            sortUpdated: string;
            sortCreated: string;
            orderDesc: string;
            orderAsc: string;
            colId: string;
            colSummary: string;
            colTags: string;
            colScore: string;
            colCreatedAt: string;
            colUpdatedAt: string;
            colActions: string;
            deleteTitle: string;
            deleteConfirm: string;
            deleted: string;
        };
        tags: {
            title: string;
            subtitle: string;
            create: string;
            colName: string;
            colDescription: string;
            colMemoryCount: string;
            colLastUsed: string;
            editTitle: string;
            createTitle: string;
            nameLabel: string;
            namePlaceholder: string;
            renameLabel: string;
            renamePlaceholder: string;
            descLabel: string;
            descPlaceholder: string;
            deleteTitle: string;
            deleteBefore: string;
            deleteMiddle: string;
            deleteAfter: string;
            detach: string;
            purge: string;
            purgeConfirmTitle: string;
            purgeConfirm: string;
            purgeButton: string;
            saved: string;
            created: string;
            deleted: string;
        };
        ops: {
            title: string;
            subtitle: string;
            refresh: string;
            statMemories: string;
            statTags: string;
            statSize: string;
            statNextId: string;
            dbCard: string;
            path: string;
            lastUpdate: string;
            lastUpdateValue: string;
            crossPlatform: string;
            crossPlatformNote: string;
            version: string;
            export: string;
            import: string;
            importHint: string;
            doctorCard: string;
            runDoctor: string;
            doctorOk: string;
            doctorFail: string;
            doctorEmpty: string;
            imported: string;
        };
        drawer: {
            title: string;
            rendered: string;
            source: string;
        };
        editor: {
            editTitle: string;
            createTitle: string;
            summaryLabel: string;
            summaryPlaceholder: string;
            contentLabel: string;
            tabEdit: string;
            tabPreview: string;
            contentPlaceholder: string;
            tagsLabel: string;
            tagsPlaceholder: string;
            updated: string;
            created: string;
        };
        errors: {
            http: string;
            invalidBackup: string;
        };
        shell: {
            theme: string;
            themeLight: string;
            themeDark: string;
            themeSystem: string;
            language: string;
        };
    };
}, {}, {}, MemoryUILocale, false>;
/** 库内组件统一从这里取 t（composer.t 为响应式） */
export declare const t: import('vue-i18n').ComposerTranslation<{
    zh: {
        nav: {
            memories: string;
            tags: string;
            ops: string;
        };
        common: {
            detail: string;
            edit: string;
            delete: string;
            cancel: string;
            save: string;
        };
        admin: {
            hint: string;
        };
        memories: {
            title: string;
            subtitle: string;
            create: string;
            searchPlaceholder: string;
            tagFilter: string;
            sortUpdated: string;
            sortCreated: string;
            orderDesc: string;
            orderAsc: string;
            colId: string;
            colSummary: string;
            colTags: string;
            colScore: string;
            colCreatedAt: string;
            colUpdatedAt: string;
            colActions: string;
            deleteTitle: string;
            deleteConfirm: string;
            deleted: string;
        };
        tags: {
            title: string;
            subtitle: string;
            create: string;
            colName: string;
            colDescription: string;
            colMemoryCount: string;
            colLastUsed: string;
            editTitle: string;
            createTitle: string;
            nameLabel: string;
            namePlaceholder: string;
            renameLabel: string;
            renamePlaceholder: string;
            descLabel: string;
            descPlaceholder: string;
            deleteTitle: string;
            deleteBefore: string;
            deleteMiddle: string;
            deleteAfter: string;
            detach: string;
            purge: string;
            purgeConfirmTitle: string;
            purgeConfirm: string;
            purgeButton: string;
            saved: string;
            created: string;
            deleted: string;
        };
        ops: {
            title: string;
            subtitle: string;
            refresh: string;
            statMemories: string;
            statTags: string;
            statSize: string;
            statNextId: string;
            dbCard: string;
            path: string;
            lastUpdate: string;
            lastUpdateValue: string;
            crossPlatform: string;
            crossPlatformNote: string;
            version: string;
            export: string;
            import: string;
            importHint: string;
            doctorCard: string;
            runDoctor: string;
            doctorOk: string;
            doctorFail: string;
            doctorEmpty: string;
            imported: string;
        };
        drawer: {
            title: string;
            rendered: string;
            source: string;
        };
        editor: {
            editTitle: string;
            createTitle: string;
            summaryLabel: string;
            summaryPlaceholder: string;
            contentLabel: string;
            tabEdit: string;
            tabPreview: string;
            contentPlaceholder: string;
            tagsLabel: string;
            tagsPlaceholder: string;
            updated: string;
            created: string;
        };
        errors: {
            http: string;
            invalidBackup: string;
        };
        shell: {
            theme: string;
            themeLight: string;
            themeDark: string;
            themeSystem: string;
            language: string;
        };
    };
    en: {
        nav: {
            memories: string;
            tags: string;
            ops: string;
        };
        common: {
            detail: string;
            edit: string;
            delete: string;
            cancel: string;
            save: string;
        };
        admin: {
            hint: string;
        };
        memories: {
            title: string;
            subtitle: string;
            create: string;
            searchPlaceholder: string;
            tagFilter: string;
            sortUpdated: string;
            sortCreated: string;
            orderDesc: string;
            orderAsc: string;
            colId: string;
            colSummary: string;
            colTags: string;
            colScore: string;
            colCreatedAt: string;
            colUpdatedAt: string;
            colActions: string;
            deleteTitle: string;
            deleteConfirm: string;
            deleted: string;
        };
        tags: {
            title: string;
            subtitle: string;
            create: string;
            colName: string;
            colDescription: string;
            colMemoryCount: string;
            colLastUsed: string;
            editTitle: string;
            createTitle: string;
            nameLabel: string;
            namePlaceholder: string;
            renameLabel: string;
            renamePlaceholder: string;
            descLabel: string;
            descPlaceholder: string;
            deleteTitle: string;
            deleteBefore: string;
            deleteMiddle: string;
            deleteAfter: string;
            detach: string;
            purge: string;
            purgeConfirmTitle: string;
            purgeConfirm: string;
            purgeButton: string;
            saved: string;
            created: string;
            deleted: string;
        };
        ops: {
            title: string;
            subtitle: string;
            refresh: string;
            statMemories: string;
            statTags: string;
            statSize: string;
            statNextId: string;
            dbCard: string;
            path: string;
            lastUpdate: string;
            lastUpdateValue: string;
            crossPlatform: string;
            crossPlatformNote: string;
            version: string;
            export: string;
            import: string;
            importHint: string;
            doctorCard: string;
            runDoctor: string;
            doctorOk: string;
            doctorFail: string;
            doctorEmpty: string;
            imported: string;
        };
        drawer: {
            title: string;
            rendered: string;
            source: string;
        };
        editor: {
            editTitle: string;
            createTitle: string;
            summaryLabel: string;
            summaryPlaceholder: string;
            contentLabel: string;
            tabEdit: string;
            tabPreview: string;
            contentPlaceholder: string;
            tagsLabel: string;
            tagsPlaceholder: string;
            updated: string;
            created: string;
        };
        errors: {
            http: string;
            invalidBackup: string;
        };
        shell: {
            theme: string;
            themeLight: string;
            themeDark: string;
            themeSystem: string;
            language: string;
        };
    };
}, "zh" | "en", import('@intlify/core-base').RemoveIndexSignature<{
    [x: string]: import('vue-i18n').LocaleMessageValue<import('vue-i18n').VueMessageType>;
}>, never, "nav.memories" | "nav.tags" | "nav.ops" | "common.detail" | "common.edit" | "common.delete" | "common.cancel" | "common.save" | "admin.hint" | "memories.title" | "memories.subtitle" | "memories.create" | "memories.searchPlaceholder" | "memories.tagFilter" | "memories.sortUpdated" | "memories.sortCreated" | "memories.orderDesc" | "memories.orderAsc" | "memories.colId" | "memories.colSummary" | "memories.colTags" | "memories.colScore" | "memories.colCreatedAt" | "memories.colUpdatedAt" | "memories.colActions" | "memories.deleteTitle" | "memories.deleteConfirm" | "memories.deleted" | "tags.title" | "tags.subtitle" | "tags.create" | "tags.deleteTitle" | "tags.deleted" | "tags.colName" | "tags.colDescription" | "tags.colMemoryCount" | "tags.colLastUsed" | "tags.editTitle" | "tags.createTitle" | "tags.nameLabel" | "tags.namePlaceholder" | "tags.renameLabel" | "tags.renamePlaceholder" | "tags.descLabel" | "tags.descPlaceholder" | "tags.deleteBefore" | "tags.deleteMiddle" | "tags.deleteAfter" | "tags.detach" | "tags.purge" | "tags.purgeConfirmTitle" | "tags.purgeConfirm" | "tags.purgeButton" | "tags.saved" | "tags.created" | "ops.title" | "ops.subtitle" | "ops.refresh" | "ops.statMemories" | "ops.statTags" | "ops.statSize" | "ops.statNextId" | "ops.dbCard" | "ops.path" | "ops.lastUpdate" | "ops.lastUpdateValue" | "ops.crossPlatform" | "ops.crossPlatformNote" | "ops.version" | "ops.export" | "ops.import" | "ops.importHint" | "ops.doctorCard" | "ops.runDoctor" | "ops.doctorOk" | "ops.doctorFail" | "ops.doctorEmpty" | "ops.imported" | "drawer.title" | "drawer.rendered" | "drawer.source" | "editor.editTitle" | "editor.createTitle" | "editor.created" | "editor.summaryLabel" | "editor.summaryPlaceholder" | "editor.contentLabel" | "editor.tabEdit" | "editor.tabPreview" | "editor.contentPlaceholder" | "editor.tagsLabel" | "editor.tagsPlaceholder" | "editor.updated" | "errors.http" | "errors.invalidBackup" | "shell.theme" | "shell.themeLight" | "shell.themeDark" | "shell.themeSystem" | "shell.language", "nav.memories" | "nav.tags" | "nav.ops" | "common.detail" | "common.edit" | "common.delete" | "common.cancel" | "common.save" | "admin.hint" | "memories.title" | "memories.subtitle" | "memories.create" | "memories.searchPlaceholder" | "memories.tagFilter" | "memories.sortUpdated" | "memories.sortCreated" | "memories.orderDesc" | "memories.orderAsc" | "memories.colId" | "memories.colSummary" | "memories.colTags" | "memories.colScore" | "memories.colCreatedAt" | "memories.colUpdatedAt" | "memories.colActions" | "memories.deleteTitle" | "memories.deleteConfirm" | "memories.deleted" | "tags.title" | "tags.subtitle" | "tags.create" | "tags.deleteTitle" | "tags.deleted" | "tags.colName" | "tags.colDescription" | "tags.colMemoryCount" | "tags.colLastUsed" | "tags.editTitle" | "tags.createTitle" | "tags.nameLabel" | "tags.namePlaceholder" | "tags.renameLabel" | "tags.renamePlaceholder" | "tags.descLabel" | "tags.descPlaceholder" | "tags.deleteBefore" | "tags.deleteMiddle" | "tags.deleteAfter" | "tags.detach" | "tags.purge" | "tags.purgeConfirmTitle" | "tags.purgeConfirm" | "tags.purgeButton" | "tags.saved" | "tags.created" | "ops.title" | "ops.subtitle" | "ops.refresh" | "ops.statMemories" | "ops.statTags" | "ops.statSize" | "ops.statNextId" | "ops.dbCard" | "ops.path" | "ops.lastUpdate" | "ops.lastUpdateValue" | "ops.crossPlatform" | "ops.crossPlatformNote" | "ops.version" | "ops.export" | "ops.import" | "ops.importHint" | "ops.doctorCard" | "ops.runDoctor" | "ops.doctorOk" | "ops.doctorFail" | "ops.doctorEmpty" | "ops.imported" | "drawer.title" | "drawer.rendered" | "drawer.source" | "editor.editTitle" | "editor.createTitle" | "editor.created" | "editor.summaryLabel" | "editor.summaryPlaceholder" | "editor.contentLabel" | "editor.tabEdit" | "editor.tabPreview" | "editor.contentPlaceholder" | "editor.tagsLabel" | "editor.tagsPlaceholder" | "editor.updated" | "errors.http" | "errors.invalidBackup" | "shell.theme" | "shell.themeLight" | "shell.themeDark" | "shell.themeSystem" | "shell.language">;
export declare function setMemoryUILocale(locale: MemoryUILocale): void;
/** 应用 locale 偏好；'auto' 表示跟随浏览器语言 */
export declare function applyMemoryUILocalePreference(preference: MemoryUILocaleOption): void;
export declare function currentMemoryUILocale(): MemoryUILocale;
