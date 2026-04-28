<template>
    <div class="menu">
        <div class="min" @click="min">
            <el-icon>
                <Minus />
            </el-icon>
        </div>
        <div style="width: 18px;"></div>
        <div class="min" @click="max">
            <el-icon v-if="!isFull">
                <FullScreen />
            </el-icon>
            <el-icon v-else>
                <Crop />
            </el-icon>
        </div>
        <div style="width: 18px;"></div>
        <div class="close" @click="close">
            <el-icon>
                <CloseBold />
            </el-icon>
        </div>
    </div>
</template>

<script setup>
import { Minus, CloseBold, FullScreen, Crop } from '@element-plus/icons-vue'
import { isFull } from '../utils/popupStore'

const min = () => window.electron.ipcRenderer.send('min')
const max = () => {
    if (!isFull.value) {
        window.electron.ipcRenderer.send('max')
        isFull.value = true
    } else {
        window.electron.ipcRenderer.send('unmaximize')
        isFull.value = false
    }
}
const close = () => window.electron.ipcRenderer.send('close')
</script>

<style lang="scss">
.menu {
    position: fixed;
    top: 0;
    z-index: 999;
    -webkit-app-region: drag;
    width: 100%;
    height: 40px;
    display: flex;
    padding: 0 20px;
    justify-content: flex-end;
    align-items: center;
    font-size: 22px;
    background-color: rgba(135, 206, 235, .03);
    backdrop-filter: blur(20px);
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.03);

    .min,
    .close {
        font-size: 18px;
        -webkit-app-region: none;
        cursor: pointer;
        transition: all .4s;

        &:hover {
            color: skyblue;
        }
    }

    .min {
        font-size: 16px;
    }
}
</style>