import { Ref } from 'vue';
export declare function useContainerWidth(target: Ref<HTMLElement | null>, compactBelow?: number): {
    width: Ref<number, number>;
    compact: import('vue').ComputedRef<boolean>;
};
