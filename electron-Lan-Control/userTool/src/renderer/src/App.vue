<script setup>
import LogViewer from './components/LogViewer.vue'
import HeaderInfo from './components/HeaderInfo.vue'
import SystemCharts from './components/SystemCharts.vue'
import tbar from './components/tbar.vue'
import tabPage from './components/tabPage.vue'
import { ref } from 'vue';

const showDrawer = ref(false);

</script>

<template>
    <div>
        <tbar @show="showDrawer = true" />
        <div class="app-container">
            <div class="top-section">
                <LogViewer class="log-viewer" />
                <div style="width: 4px;"></div>
                <HeaderInfo class="header-info" />
            </div>
            <SystemCharts class="chart-section" />
        </div>

        <tabPage :visible="showDrawer" />
        <div v-if="showDrawer" class="drawer-footer" @click="showDrawer = false"></div>

    </div>
</template>


<style lang="scss">
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

.app-container {
    margin-top: 40px;
    display: flex;
    flex-direction: column;
    height: calc(100vh - 40px);
    background-color: rgba(0, 0, 0, 0.01);
    backdrop-filter: blur(10px);
    color: #fff;
    overflow: hidden;
}

.top-section {
    display: flex;
    flex: .5;
    min-height: 0;
    padding: 10px;
    gap: 10px;
    margin-bottom: 10px;
}

.log-viewer {
    flex: 1.5;
    overflow: hidden;
}

.header-info {
    flex: 1;
    overflow: auto;
}

.chart-section {
    flex: 1;
    padding: 10px;
    min-height: 0;
}

::-webkit-scrollbar {
    width: 0px;
    background: transparent;
}

.drawer-footer {
    position: fixed;
    top: 0;
    left: 0;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.1);
}
</style>