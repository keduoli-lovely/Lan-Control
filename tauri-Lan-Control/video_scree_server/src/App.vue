<script setup>
import { onMounted, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import app_contorl from './views/app_contorl.vue';
import device_view from '@/views/device_view.vue';
import setting_tap from '@/views/setting_tap.vue';
import header_script_tap from '@/Components/header_script_tap.vue';
import { useMask } from '@/store/mask';
import { useAppConfig } from '@/store/config';
import { init_listen } from '@/composables/init_listen';
import { init_store } from '@/composables/init_store';

const Mask_data = useMask();
const AppConfig = useAppConfig()

// 初始化广播 / ws管理
onMounted(async () => {
  // 加载json配置
  await init_store()

  // 监听ws的移除 / 删除
  await init_listen()

  // 广播
  await invoke("initialize_runtime")
  // 获取全部设备
  let res_device_list = await invoke("sync_devices")
  if (!AppConfig.device_list_all) AppConfig.device_list_all = res_device_list
})

// 是否显示顶部批量控制
const header_script_show = computed(() => {
  return AppConfig?.option_script && Object.keys(AppConfig.option_script[0]).length > 0
})
</script>



<template>
  <div class="main" v-loading.fullscreen.lock="Mask_data.main_mask" style="overflow-y: hidden;">
    <app_contorl />
    <header_script_tap v-if="header_script_show" />
    <device_view :header_script_show="header_script_show" />
    <setting_tap />
  </div>
</template>