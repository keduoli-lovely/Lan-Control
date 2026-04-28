<template>
  <div class="box">
    <tbarVue />
    <div style="height: 40px;"></div>
    <div class="device-grid">
      <!-- 每个设备方块 -->
      <div class="device-card" v-for="(item, index) in device_list" :key="item.ip">
        <div class="SyncSta" v-if="item?.syncSta" style="color: #B0D5FB;">
          <el-icon>
            <Refresh />
          </el-icon>

          <span>脚本已同步</span>
        </div>
        <div class="SyncSta" v-else style="color: #FFA239;">
          <el-icon>
            <Refresh />
          </el-icon>

          <span>脚本待同步</span>
        </div>
        <img :src="device_pic[item.ip]" alt="获取桌面中..." class="device-image" />
        <div class="control-overlay">
          <button @click="remoteView(item.ip)">远程查看</button>
          <button @click="SendRunScript([item.ip, 'furmark'])">运行Furmark</button>
          <button @click="SendRunScript([item.ip, 'showdown'])">关机</button>
          <button @click="SendRunScript([item.ip, 'restart'])">重启</button>
          <button
            @click="SendRunScript([item.ip, 'getInfo']), modelShow = true, showModelPage = true, ActiveIp = item.ip, ActiveSyncSta = item.syncSta">查看更多...</button>
        </div>

        <div class="device_info">
          <div>
            <span>名称: </span>
            <div style="width: 10px;"></div>
            <div>
              <div class="name" v-if="!setEdit[item.ip]?.state">{{ item.name }}</div>
              <el-input :ref="setItemRef(index)" v-else autofocus v-model="setEdit[item.ip].newName"
                style="width: 100px;height: 22px;" :placeholder="item.name" @blur="checkNameChange(item)"
                @keydown.enter="checkNameChange(item)" />
            </div>
            <div style="width: 20px;"></div>
            <div class="icon" style="width: 15px;cursor: pointer;display: flex;white-space: nowrap;"
              v-if="!setEdit[item.ip]?.state" @click="enableEdit(item, index)">
              <el-icon>
                <Edit />
              </el-icon>
            </div>
            <div class="icon" v-else
              style="width: 15px;cursor: pointer;display: flex;white-space: nowrap;align-items: center;"
              @click="setEdit[item.ip] = { state: false }">
              <el-icon>
                <Close />
              </el-icon>
            </div>
          </div>
          <div><span>I&nbsp; P &nbsp;: </span>
            <div style="width: 10px;"></div>
            <div class="ip">{{ item.ip }}</div>
          </div>
        </div>
      </div>

      <!-- 更多设备... -->
      <div class="runAllCode" @click="handleClick" title="批量操作功能"
        :style="{ right: `${moveToOutShowFlag ? '40px' : '-24px'}` }">
        <div class="runAllCodeMask" data-keduoli="showDom" @mouseenter="moveTOShowMenu('move')"
          @mouseleave="moveTOShowMenu('out')"></div>
        <el-icon style="z-index: -1;">
          <Menu />
        </el-icon>
        <div class="towMenu" :style="{ right: `${menuShow ? '0' : '-1000px'}` }">
          <div class="row1"
            @click="modelShow = true, showModelPage = false, menuShow = !menuShow, moveToOutShowFlag = false">
            <el-icon>
              <Setting />
            </el-icon>
            <div style="width: 8px;"></div>
            <div>软件设置</div>
          </div>
          <div class="row1 menu2">
            <el-icon>
              <Fold />
            </el-icon>
            <div style="width: 8px;"></div>
            <div>运行脚本</div>

            <div class="sanMenu">
              <div class="row2" @click="SendToAll('furmark')">
                运行Furmark
              </div>
              <div class="row2" @click="syncScript('furmark')">同步脚本</div>
              <div class="row2" @click="SendToAll('showdown')">关机</div>
              <div class="row2" @click="SendToAll('restart')">重启</div>
            </div>
          </div>
        </div>
      </div>
    </div>


    <ModalDialog />
  </div>


</template>
<script setup>
import { Menu, Setting, Fold, Edit, Check, Close, Refresh, Stopwatch } from '@element-plus/icons-vue'
import tbarVue from './tbar.vue'
import ModalDialog from './ModalDialog.vue'
import { onMounted } from 'vue'
import {
  device_pic,
  menuShow,
  setEdit,
  SendToAll,
  handleClick,
  setItemRef,
  enableEdit,
  checkNameChange,
  setupListeners,
  moveTOShowMenu,
  moveToOutShowFlag
} from './Home.script.js'

import {
  remoteView,
  modelShow,
  SendRunScript,
  ActiveIp,
  device_list,
  showModelPage,
  syncScript,
  ActiveSyncSta
} from '../utils/popupStore'

setupListeners()

onMounted(() => {
  moveTOShowMenu('move')
})
</script>


<style scoped lang="scss">
.device-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 56px;
  padding: 16px;
}

.device-card {
  position: relative;
  width: 100%;
  aspect-ratio: 16 / 9;
  /* 保持截图比例 */
  // overflow: hidden;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  background-color: #f0f0f0;

  .device_info {
    position: absolute;
    bottom: -50px;
    left: 50%;
    transform: translateX(-50%);

    &>div {
      white-space: nowrap;
      display: flex;
      align-items: center;

      &>span {
        text-align: right;
        width: 40px;
      }
    }
  }

  .SyncSta {
    position: absolute;
    right: -2px;
    bottom: -5px;
    width: 90px;
    height: 30px;
    // background-color: #000;
    display: flex;
    align-items: center;
    line-height: 30px;
    justify-content: space-evenly;

    &>span {
      font-size: 12px;
    }


  }
}

.device-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.control-overlay {
  padding: 20px 0;
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  justify-content: center;
  align-items: center;
  flex-wrap: wrap;
  gap: 12px;
  background-color: rgba(0, 0, 0, 0.4);
  opacity: 0;
  transition: opacity 0.3s ease;
}

.device-card:hover .control-overlay {
  opacity: 1;
}

.control-overlay button {
  padding: 8px 12px;
  background-color: #ffffffcc;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-weight: bold;
  transition: background-color 0.2s ease;
}

.control-overlay button:hover {
  background-color: #fff;
}

.icon {
  &:hover {
    color: #409eff;
  }
}

.runAllCode {
  position: fixed;
  bottom: 40px;
  right: -24px;
  z-index: 9999;
  width: 60px;
  height: 60px;
  display: flex;
  justify-content: center;
  align-items: center;
  backdrop-filter: blur(10px);
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.03);
  border-radius: 50%;
  box-shadow: rgba(0, 0, 0, 0.24) 0px 3px 8px;
  cursor: pointer;
  transition: all 0.3s ease;

  &:hover {
    background-color: rgba(0, 0, 0, 0.1);
  }

  .towMenu {
    transition: all 0.3s ease;
    position: absolute;
    top: -85px;
    right: -1000px;
    background-color: #fff;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    border-radius: 4px;
    width: 120px;

    .row1 {
      display: flex;
      flex-wrap: nowrap;
      align-items: center;
      justify-content: center;
      height: 35px;

      &:hover {
        background-color: #f5f5f5;
      }
    }

    .row1:first-child {
      border-bottom: 1px solid #ddd;
    }

    .menu2 {
      position: relative;
      cursor: pointer;

      .sanMenu {
        transition: all 0.3s ease;
        text-align: center;
        position: absolute;
        bottom: 0;
        left: -100%;
        background-color: #fff;
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
        width: 120px;

        .row2 {
          display: flex;
          align-items: center;
          height: 40px;
          line-height: 40px;
          padding-left: 10px;

          &:hover {
            background-color: #f5f5f5;
          }
        }

        height: 0;
        overflow: hidden;
      }

      &:hover .sanMenu {
        height: 160px !important;
      }
    }
  }

  .runAllCodeMask {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    border-radius: 50%;
  }
}
</style>