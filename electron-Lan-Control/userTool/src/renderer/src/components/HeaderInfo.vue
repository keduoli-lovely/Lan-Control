<template>
  <div class="header-info">
    <div><strong>名称：</strong>{{ info?.name }}</div>
    <div><strong>IP: </strong>{{ info?.ip }}</div>
    <div><strong>平台：</strong>{{ info?.platform }}</div>
    <div><strong>内存：</strong>{{ info?.mem }}</div>
    <div><strong>CPU: </strong>{{ info?.cpu }}</div>
    <div><strong>GPU: </strong>{{ info?.gpu }}</div>
  </div>
</template>

<script setup>
import { ref } from 'vue';

const info = ref({
  name: ' ',
  ip: ' ',
  platform: ' ',
  mem: ' ',
  cpu: ' ',
  gpu: ' '
})
// 接收系统信息
window.api.Sendinfo('Systeminfo', (data) => {
  if (data.state) {
    info.value.name = data.nikename;
  } else {
    info.value.mem = data.memory;
    info.value.cpu = data.cpu;
    try {
      info.value.gpu = data.gpu.length > 0 ? data.gpu[0] : data.gpu;
    } catch (error) {
      info.value.gpu = '无GPU信息';
    }
    info.value.platform = data.platform;
    info.value.name = data.nikename;
    info.value.ip = data.ip;
  }
});
</script>

<style scoped>
.header-info {
  /* background-color: rgba(0, 0, 0, 0.01); */
  background-color: #F7F9FA;
  padding: 10px;
  box-shadow: rgba(0, 0, 0, 0.24) 0px 3px 8px;
  font-size: 14px;
  line-height: 1.6;
  height: 100%;
  overflow-y: auto;
  /* color: skyblue; */
  color: #1A1A1A;
  border-radius: 8px;
}
.header-info div {
  color: #555555;
}
.header-info div>strong {
  color: #333333;
}
</style>
