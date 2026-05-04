<template>
    <div class="setting_page">
        <el-card class="config-card" shadow="never" body-style="padding: 0">
            <template #header>
                <div class="card-header">
                    <span>系统设置</span>
                    <el-button type="primary" @click="submitForm">保存设置</el-button>
                </div>
            </template>

            <el-form :model="AppConfig.config" label-width="160px" class="setting-form">
                <el-form-item label="客户端开机自启">
                    <el-switch v-model="AppConfig.config.autoStart" />
                </el-form-item>
                <el-form-item label="WS端口" :rules="[{ validator: validateWsPort, trigger: 'blur' }]">
                    <el-input-number v-model="AppConfig.config.wsPort" :min="2000" :max="50000" />
                </el-form-item>
                <el-form-item label="WS指令路径">
                    <el-input v-model="AppConfig.config.wsCommandPath" placeholder="/ws/command" clearable />
                </el-form-item>
                <el-form-item label="WS截图路径">
                    <el-input v-model="AppConfig.config.wsScreenshotPath" placeholder="/ws/pic" clearable />
                </el-form-item>
                <el-form-item label="广播端口" :rules="[{ validator: validateBroadcastPort, trigger: 'blur' }]">
                    <el-input-number v-model="AppConfig.config.broadcastPort" :min="2000" :max="50000" />
                </el-form-item>
                <el-form-item label="客户端重连次数">
                    <el-input-number v-model="AppConfig.config.reconnectTimes" :min="5" :max="999" />
                </el-form-item>
                <el-form-item label="截图频率 (毫秒)">
                    <el-input-number v-model="AppConfig.config.screenshotInterval" :min="100" :max="5000" />
                </el-form-item>
                <el-form-item label="客户端预览">
                    <el-switch v-model="AppConfig.config.preview" />
                </el-form-item>
                <el-form-item label="远程时暂停预览">
                    <el-switch v-model="AppConfig.config.pausePreview" />
                </el-form-item>
                <el-form-item label="客户端异常退出WS">
                    <el-switch v-model="AppConfig.config.reconnectWs" />
                </el-form-item>
                <el-form-item label="客户端重连间隔 (秒)">
                    <el-input-number v-model="AppConfig.config.reconnectInterval" :min="1" :max="40"
                        :disabled="!AppConfig.config.reconnectWs" />
                </el-form-item>
                <div style="height: 40px;"></div>
            </el-form>
        </el-card>
    </div>
</template>

<script setup>
import { ElMessage } from 'element-plus'
import { useAppConfig } from '@/store/config'
import { Store } from '@tauri-apps/plugin-store';
import { invoke } from '@tauri-apps/api/core';
import { onUnmounted, watch, ref, onMounted } from 'vue';

const AppConfig = useAppConfig()
const autoSaveTimeout = ref(null)
const default_run_start = ref(null)

onMounted(() => {
    default_run_start.value = AppConfig.config.autoStart
})

// 设置默认值
if (!AppConfig.config.wsPort) AppConfig.config.wsPort = 9000
if (!AppConfig.config.wsCommandPath) AppConfig.config.wsCommandPath = "/ws/command"
if (!AppConfig.config.wsScreenshotPath) AppConfig.config.wsScreenshotPath = "/ws/pic"
if (!AppConfig.config.broadcastPort) AppConfig.config.broadcastPort = 13140
if (!AppConfig.config.reconnectTimes) AppConfig.config.reconnectTimes = 5
if (!AppConfig.config.screenshotInterval) AppConfig.config.screenshotInterval = 500

// 校验 WS 端口
function validateWsPort(_, value, callback) {
    if (value < 2000 || value > 50000) {
        callback(new Error('WS端口必须在2000-50000之间'))
    } else if (value === AppConfig.config.broadcastPort) {
        callback(new Error('WS端口不能与广播端口相同'))
    } else {
        callback()
    }
}

// 校验广播端口
function validateBroadcastPort(_, value, callback) {
    if (value < 2000 || value > 50000) {
        callback(new Error('广播端口必须在2000-50000之间'))
    } else if (value === AppConfig.config.wsPort) {
        callback(new Error('广播端口不能与WS端口相同'))
    } else {
        callback()
    }
}

// 自动保存配置到store
async function autoSaveConfig() {
    try {
        const store = await Store.load('.server_settings.json');
        await store.set("config", AppConfig.config)
        await store.save()
        if (default_run_start.value !== AppConfig.config.autoStart) {
            await set_auto_run_state()
            default_run_start.value = AppConfig.config.autoStart
        }
        console.log('配置已自动保存')
    } catch (error) {
        console.error('自动保存配置失败:', error)
    }
}

// 防抖自动保存函数
function debouncedAutoSave() {
    if (autoSaveTimeout.value) {
        clearTimeout(autoSaveTimeout.value);
    }
    autoSaveTimeout.value = setTimeout(async () => {
        await autoSaveConfig();
        console.log('配置已通过防抖自动保存');
    }, 2000); // 2秒后自动保存
}

async function submitForm() {
    try {
        await autoSaveConfig()

        // 触发客户端重新发现服务器配置
        try {
            await invoke('trigger_client_rediscovery')
            console.log('已触发客户端重新发现服务器配置')
        } catch (error) {
            console.error('触发客户端重新发现失败:', error)
            // 不阻止保存成功，只是记录错误
        }

        ElMessage.success('设置已保存！')
    } catch (error) {
        ElMessage.error('保存失败:', error)
    }
}

// 设置开机自启动
const set_auto_run_state = async () => {
    console.log(`正在设置开机自启状态为 ${Object.keys(AppConfig.device_list_all)} 个设备，请稍候...`)
    // 发送给全部设备
    let device_list = Object.keys(AppConfig.device_list_all) || []
    device_list.forEach(async (ip) => {
        try {
            await invoke('sned_fn', { ip, key: `auto_run_state|${AppConfig.config.autoStart}` })
            console.log(`已设置设备 ${ip} 的开机自启状态为 ${AppConfig.config.autoStart}`)
        } catch (error) {
            console.error(`设置设备 ${ip} 的开机自启状态失败:`, error)
        }
    })
}

// 监听配置变化
watch(() => AppConfig.config, (newVal, oldVal) => {
    if (oldVal && JSON.stringify(newVal) !== JSON.stringify(oldVal)) {
        console.log('配置发生变化，触发防抖保存');
        debouncedAutoSave();
    }
}, { deep: true, immediate: false });

// 组件卸载时清理定时器
onUnmounted(() => {
    if (autoSaveTimeout.value) {
        clearTimeout(autoSaveTimeout.value);
    }
    // 确保配置已保存
    autoSaveConfig().catch(console.error);
})
</script>

<style lang="scss" scoped>
$bg-color: #f5f7fa;
$border-color: #e4e7ed;

.setting_page {
    margin: 10px 20px 0 0;
    padding-bottom: 20px;
    height: 100vh;
    overflow-y: auto;
    box-sizing: border-box;
    background-color: $bg-color;

    .config-card {
        max-width: none;
        border: 1px solid $border-color;
        margin-right: 10px;

        .card-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            font-weight: bold;
            font-size: 16px;
        }

        .setting-form {
            margin-top: 20px;

            .el-form-item {
                margin: 14px !important;
            }
        }
    }
}

/* 响应式调整 */
@media (max-width: 768px) {
    .setting_page {
        padding: 10px;
    }

    :deep(.el-form-item__label) {
        float: none;
        display: block;
        text-align: left;
        margin-bottom: 8px;
    }

    :deep(.el-form-item__content) {
        margin-left: 0 !important;
    }
}

::-webkit-scrollbar {
    width: 2px;
    height: 2px;

    &-thumb {
        background-color: skyblue;
        border-radius: 2px;
    }
}
</style>