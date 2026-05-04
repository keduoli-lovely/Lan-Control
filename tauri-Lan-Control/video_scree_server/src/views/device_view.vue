<template>
    <div class="device_view" :class="{ 'show_header_height': header_script_show }">
        <div class="device_list" style="height: 100%;overflow-y: auto;">
            <devices_list />
        </div>
    </div>
</template>

<script setup>
import { Setting } from '@element-plus/icons-vue'
import devices_list from '@/Components/devices_list.vue';

defineProps({
    header_script_show: {
        type: Boolean
    }
})
</script>

<style lang="scss" scoped>
.device_view {
    box-sizing: border-box;
    width: 100%;
    height: calc(100vh - 32px);
    overflow-y: hidden;
}

.show_header_height {
    height: calc(100vh - 77px) !important;
}

::-webkit-scrollbar {
    width: 4px;
    height: 2px;
}

::-webkit-scrollbar-thumb {
    background-color: skyblue;
    border-radius: 2px;
}
</style>