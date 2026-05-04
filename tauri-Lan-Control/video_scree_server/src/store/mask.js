import { defineStore } from "pinia";
import { ref } from "vue";

export const useMask = defineStore("mask", () => {
    const main_mask = ref(false)
    const main_mask_text = ref("加载中...")


    return {
        main_mask,
        main_mask_text
    }
})