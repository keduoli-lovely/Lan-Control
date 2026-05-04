<template>
    <div class="header">
        <div class="rowS" @wheel.prevent="handleWheel">
            <el-button @dblclick="run_all_script(item)" type="primary" plain v-for="item in AppConfig.option_script[0]"
                :key="item.name">{{ item.name
                }}</el-button>
        </div>
    </div>
</template>

<script setup>
import { useAppConfig } from '@/store/config';
import { invoke } from '@tauri-apps/api/core';
import { ElNotification } from 'element-plus';

const AppConfig = useAppConfig()
function handleWheel(e) {
    const box = e.currentTarget;
    box.scrollLeft += e.deltaY;
}

// 所有设备执行
const run_all_script = async (script) => {
    console.log("执行脚本:", script, AppConfig?.device_list_all)
    try {
        if ("key" in script) {
            // 内部指令
            if (!AppConfig?.device_list_all || !Object.keys(AppConfig?.device_list_all).length) return;
            
            for (let item in AppConfig.device_list_all) {
                let cmd = script.key;
                
                // 处理内部命令：shutdown, reboot, sync_files
                if (script.key === 'shutdown' || script.key === 'reboot' || script.key === 'sync_files') {
                    // 直接使用脚本key作为命令
                    cmd = script.key;
                } else if (script?.fullData?.id) {
                    // 外部脚本，使用script_execute格式
                    cmd = `script_execute|${script.fullData.id}`;
                }
                
                console.log("发送命令到设备:", item, "命令:", cmd);
                await invoke("sned_fn", {
                    ip: item, key: cmd
                })

                ElNotification({
                    type: 'success',
                    title: '执行成功',
                    message: `命令 "${script.name}" 已发送到设备 ${item}`,
                    duration: 3000
                })
            }
        }
    } catch (error) {
        ElNotification({
            type: 'error',
            title: '执行失败',
            message: `命令执行失败: ${error.message}`,
            duration: 3000
        })
    }
}

</script>

<style lang="scss" scoped>
.header {
    padding: 0 40px 0 20px;
    height: 45px;
    line-height: 45px;
    box-shadow: 0 4px 6px -2px rgba(0, 0, 0, 0.15);

    .rowS {
        overflow-x: auto;
        white-space: nowrap;
        width: 90vw;
    }
}

::-webkit-scrollbar {
    width: 2px;
    height: 2px;
}

::-webkit-scrollbar-thumb {
    background-color: skyblue;
    border-radius: 2px;
}
</style>