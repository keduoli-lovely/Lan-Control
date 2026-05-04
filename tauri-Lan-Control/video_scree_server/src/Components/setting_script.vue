<template>
    <div class="container">
        <!-- 上方框 -->
        <el-card class="box" shadow="never">
            <h3>常驻命令</h3>
            <transition-group name="fade" tag="div" class="list">
                <div v-for="item in AppConfig.option_script[0]" :key="item.key" class="item"
                    @click="moveToBottom(item)">
                    <span>{{ item.name }}</span>
                    <el-button type="danger" size="small" class="action-btn"
                        @click.stop="moveToBottom(item)">x</el-button>
                </div>
            </transition-group>
        </el-card>

        <!-- 下方框 -->
        <el-card class="box" shadow="never">
            <h3>可选命令</h3>
            <transition-group name="fade" tag="div" class="list">
                <div v-for="item in AppConfig.option_script[1]" :key="item.key" class="item" @click="moveToTop(item)">
                    <span>{{ item.name }}</span>
                    <el-button type="success" size="small" class="action-btn"
                        @click.stop="moveToTop(item)">+</el-button>
                </div>
            </transition-group>
        </el-card>
    </div>
</template>

<script setup>
import { onUnmounted, ref } from "vue";
import { Store } from '@tauri-apps/plugin-store';
import { useAppConfig } from "@/store/config";
import { ElMessage } from 'element-plus';

const AppConfig = useAppConfig()
const saveTimeout = ref(null)


// 保存配置到store
async function saveConfigToStore() {
    try {
        const store = await Store.load('.server_settings.json');
        const store_script = await store.get("script");
        if (!store_script) {
            await store.set("script", AppConfig.option_script);
            await store.save();
            return true;
        }

        const flag = change_is_update(store_script);
        if (flag) {
            await store.set("script", AppConfig.option_script);
            await store.save();
            return true;
        }
        return false;
    } catch (error) {
        console.error('保存配置失败:', error);
        return false;
    }
}

// 防抖保存函数
function debouncedSave() {
    if (saveTimeout.value) {
        clearTimeout(saveTimeout.value);
    }
    saveTimeout.value = setTimeout(async () => {
        const saved = await saveConfigToStore();
        if (saved) {
            console.log('脚本配置已自动保存');
        }
    }, 1000); // 1秒后保存
}

function moveToBottom(item) {
    // 从常驻命令移除，添加到可选命令
    if (AppConfig.option_script[0][item.key]) {
        delete AppConfig.option_script[0][item.key];
        AppConfig.option_script[1][item.key] = item;
        debouncedSave();
    }
}

function moveToTop(item) {
    // 从可选命令移除，添加到常驻命令
    if (AppConfig.option_script[1][item.key]) {
        delete AppConfig.option_script[1][item.key];
        AppConfig.option_script[0][item.key] = item;
        debouncedSave();
    }
}

const change_is_update = (state) => {
    // 检查常驻命令是否有变化
    for (let key in AppConfig.option_script[0]) {
        if (!state[0] || !state[0][key]) {
            return true;
        }
    }
    for (let key in state[0]) {
        if (!AppConfig.option_script[0][key]) {
            return true;
        }
    }
    
    // 检查可选命令是否有变化
    for (let key in AppConfig.option_script[1]) {
        if (!state[1] || !state[1][key]) {
            return true;
        }
    }
    for (let key in state[1]) {
        if (!AppConfig.option_script[1][key]) {
            return true;
        }
    }

    return false;
}

onUnmounted(async () => {
    // 清理定时器
    if (saveTimeout.value) {
        clearTimeout(saveTimeout.value);
    }
    // 确保配置已保存
    await saveConfigToStore();
})
</script>

<style lang="scss" scoped>
.container {
    display: flex;
    flex-direction: column;
    gap: 20px;
    height: 100vh;
    padding: 10px;
    background: #f9fafb;

    .box {
        flex: 1;
        display: flex;
        flex-direction: column;
        border: 1px solid #ddd;

        .list {
            flex: 1;
            padding: 10px;
            display: flex;
            flex-wrap: wrap;
            gap: 10px;
            align-content: flex-start;
            overflow-y: auto;

            .item {
                display: inline-flex;
                align-items: center;
                padding: 6px 10px;
                background: #ffffff;
                border: 1px solid #ddd;
                border-radius: 6px;
                cursor: pointer;
                transition: all 0.3s ease;

                &:hover {
                    background: #e6f7ff;
                    border-color: #91d5ff;
                }

                .action-btn {
                    margin-left: 8px;
                    padding: 2px 6px;
                }
            }
        }
    }
}

/* 动画效果 */
.fade-enter-active,
.fade-leave-active {
    transition: all 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
    opacity: 0;
    transform: translateY(10px);
}
</style>