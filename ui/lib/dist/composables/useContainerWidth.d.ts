import { Ref } from 'vue';
export declare function useContainerWidth(target: Ref<HTMLElement | null>): {
    width: Ref<number, number>;
    compact: import('vue').ComputedRef<boolean>;
    narrow: import('vue').ComputedRef<boolean>;
};
