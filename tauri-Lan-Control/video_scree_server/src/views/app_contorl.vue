<template>
    <div class="titlebar" data-tauri-drag-region>
        <div class="title-left">
            <img src="../assets/icon.png" class="title-icon" />
            <span class="title-text">scree_server</span>
        </div>
        <div class="title-right conten_center">
            <el-icon class="title-icon-btn" @click="minimize">
                <Minus />
            </el-icon>&nbsp;
            <span class="conten_center">
                <el-icon v-if="!AppConfig?.isMaxWindow" class="title-icon-btn"
                    @click="maximize(), AppConfig.isMaxWindow = true">
                    <FullScreen />
                </el-icon>

                <el-icon v-else class="title-icon-btn" @click="unmaximize(), AppConfig.isMaxWindow = false"
                    style="font-weight: 800;">
                    <CopyDocument />
                </el-icon>
            </span>&nbsp;
            <el-icon class="title-icon-btn" @click="close" style="font-size: 20px;">
                <Close />
            </el-icon>
        </div>
    </div>
</template>
<script setup>
import { getCurrentWindow } from '@tauri-apps/api/window'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { Close, Minus, FullScreen, CopyDocument } from '@element-plus/icons-vue'
import { useAppConfig } from '@/store/config';

const AppConfig = useAppConfig()
const minimize = async () => await getCurrentWindow().minimize()
const maximize = async () => await getCurrentWindow().maximize()
const unmaximize = async () => await getCurrentWindow().unmaximize()
const close = async () => {
    const video_window = await WebviewWindow.getByLabel("video");
    if (video_window) {
        await new Promise((resolve) => {
            video_window.once("tauri://destroyed", () => resolve());
            video_window.close();
        })
    }
    await getCurrentWindow().close()
}
</script>

<style lang="scss" scoped>
.titlebar {
    user-select: none;
    height: 32px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 12px;
    background-color: var(--titlebar-bg);
    color: var(--titlebar-text);
    -webkit-app-region: drag;
    border-bottom: 1px solid rgba(0, 0, 0, .05);

    .title-left {
        display: flex;
        align-items: center;

        .title-icon {
            margin-right: 4px;
            width: 20px;
            height: 20px;
        }

        .title-text {
            font-size: 13px;
            user-select: none;
        }
    }

    .title-right {
        & > * {
            color: var(--titlebar-icon-color);
            margin-left: 8px;
            cursor: pointer;
            -webkit-app-region: no-drag;
        }

        .title-icon-btn:hover {
            color: var(--titlebar-icon-hover);
        }
    }
}
</style>
