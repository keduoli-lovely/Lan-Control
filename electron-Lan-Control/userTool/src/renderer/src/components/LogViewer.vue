<template>
  <div class="log-viewer">
    <div class="log-header">运行日志</div>
    <div class="log-content" ref="logContainer">
      <div v-for="(log, index) in logs" :key="index" class="log-line">
        {{ log }}
      </div>
      <div style="height: 12px;"></div>

    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, watch, nextTick } from 'vue'

const logs = ref([])
const logContainer = ref(null)

function scrollToBottom() {
  if (logContainer.value) {
    logContainer.value.scrollTop = logContainer.value.scrollHeight
  }
}

onMounted(() => {
  scrollToBottom()
})

// 每次 logs 更新后滚动到底部
watch(logs, () => {
  nextTick(() => {
    scrollToBottom()
  })
}, {
  deep: true
})

// 接收日志数据
window.api.Sendtips('tips', (data) => {
  logs.value.push(data)
})
</script>


<style scoped>
.log-viewer {
  background-color: #fff;
  color: #333333;
  font-family: monospace;
  display: flex;
  flex-direction: column;
  box-shadow: rgba(0, 0, 0, 0.24) 0px 3px 8px;
  border-radius: 8px;
  height: 100%;
  border-radius: 6px;
  padding: 16px;
}

.log-header {
  height: 30px;
  font-size: 16px;
  font-weight: bold;
  color: #222222;
  margin-bottom: 8px;
  border-bottom: 1px solid #F0F0F0;
  padding-bottom: 4px;
  line-height: 30px;
  margin-left: 5px;
}


.log-content {
  flex: 1;
  overflow-y: auto;
  padding: 5px;
  scrollbar-width: thin;
  color: #444444;
  font-size: 14px;
  line-height: 1.6;
  scrollbar-width: thin;
  scrollbar-gutter: stable; /* 关键 */
}

.log-content::-webkit-scrollbar {
    width: 4px;
}

.log-content::-webkit-scrollbar-track {
    background: transparent;
}

.log-content::-webkit-scrollbar-thumb {
    background: #ccc;
    border-radius: 2px;
}

.log-content::-webkit-scrollbar-button {
    display: none;
}
</style>
