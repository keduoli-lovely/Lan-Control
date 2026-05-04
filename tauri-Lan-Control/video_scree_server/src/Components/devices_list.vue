<template>
    <div class="devices_list">
        <div class="device" v-for="(item, key) in AppConfig.device_list_all" :key="key">
            <div class="SyncSta" v-if="item?.device_sync" style="color: #B0D5FB;">
                <el-icon>
                    <Refresh />
                </el-icon>

                <span class="sel_none">脚本已同步</span>
            </div>
            <div class="SyncSta" v-else style="color: #FFA239;">
                <el-icon>
                    <Refresh />
                </el-icon>

                <span class="sel_none">脚本待同步</span>
            </div>
            <img :src="getDeviceImage(key)" alt="获取桌面中..." class="device-image sel_none" />
            <div class="control-overlay sel_none">
                <div>
                    <button @click="SendHandle(key, 'see')">远程查看</button>
                    <button
                        @click="SendHandle(key, 'get_script'), centerDialogVisible = !centerDialogVisible">查看更多</button>
                </div>
                <div>
                    <button @click="SendHandle(key, 'shutdown')">关闭设备</button>
                    <button @click="SendHandle(key, 'reboot')">重启设备</button>
                </div>
            </div>

            <div class="device_info">
                <div>
                    <span class="sel_none">名称<span style="font-weight: bold;">:</span> </span>
                    <div style="width: 10px;"></div>
                    <div>
                        <div class="name" v-if="!setEdit[item.device_ip]?.state">{{ item.device_name }}</div>
                        <el-input :ref="setItemRef(key)" v-else autofocus v-model="setEdit[item.device_ip].newName"
                            style="width: 100px;height: 22px;" :placeholder="item.device_name"
                            @blur="checkNameChange(item)" @keydown.enter="checkNameChange(item)" />
                    </div>
                    <div style="width: 20px;"></div>
                    <div class="icon" style="width: 15px;cursor: pointer;display: flex;white-space: nowrap;"
                        v-if="!setEdit[item.device_ip]?.state" @click="enableEdit(item, key)">
                        <el-icon>
                            <Edit />
                        </el-icon>
                    </div>
                    <div class="icon" v-else
                        style="width: 15px;cursor: pointer;display: flex;white-space: nowrap;align-items: center;"
                        @click="setEdit[item.device_ip] = { state: false }">
                        <el-icon>
                            <Close />
                        </el-icon>
                    </div>
                </div>
                <div><span class="sel_none">I P &nbsp;<span style="font-weight: bold;">:</span> </span>
                    <div style="width: 10px;"></div>
                    <div class="ip">{{ item.device_ip }}</div>
                </div>
            </div>
        </div>
        <!-- 脚本弹窗 -->
        <el-dialog v-model="centerDialogVisible" :show-close="false" title="单独执行自定义指定命令(双击执行)" width="60%" align-center
            class="sel_none">
            <div style="display: flex;flex-wrap: wrap;margin-top: 20px;">
                <el-button plain v-show="Object.keys(AppConfig.run_script_list || {}).length > 0" 
                    v-for="(scriptItem, scriptKey) in AppConfig.run_script_list"
                    :key="scriptKey" style="margin: 0 14px 14px 0;" @dblclick="executeScript(currentDeviceIp, scriptItem)">
                    {{ scriptItem.name }}
                </el-button>
            </div>
            <template #footer>
                <div class="dialog-footer">
                    <el-button @click="centerDialogVisible = false">关闭页面</el-button>
                </div>
            </template>
        </el-dialog>
    </div>
</template>


<script setup>
import { Edit, Check, Close, Refresh } from '@element-plus/icons-vue'
import { ref, onMounted, onUnmounted, computed } from 'vue';
import { ElNotification } from 'element-plus'
import { invoke } from "@tauri-apps/api/core";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useMask } from '@/store/mask';
import { useAppConfig } from '@/store/config';
import { listen } from '@tauri-apps/api/event';
import { change_deive_name } from '@/composables/change_device_name';

const Mask_data = useMask()
const AppConfig = useAppConfig()
const centerDialogVisible = ref(false)
const currentDeviceIp = ref('')
const { setEdit,
    inputRefs,
    checkNameChange,
    enableEdit, setItemRef } = change_deive_name()

// 存储每个设备的预览图片URL和最后更新时间
const deviceImages = ref({})
const deviceLastUpdate = ref({})

// 默认黑色图片base64
const blackImageBase64 = 'data:image/jpeg;base64,/9j/4AAQSkZJRgABAQEASABIAAD/2wBDAAYEBQYFBAYGBQYHBwYIChAKCgkJChQODwwQFxQYGBcUFhYaHSUfGhsjHBYWICwgIyYnKSopGR8tMC0oMCUoKSj/2wBDAQcHBwoIChMKChMoGhYaKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCj/wAARCAABAAEDASIAAhEBAxEB/8QAFQABAQAAAAAAAAAAAAAAAAAAAAv/xAAUEAEAAAAAAAAAAAAAAAAAAAAA/8QAFQEBAQAAAAAAAAAAAAAAAAAAAAX/xAAUEQEAAAAAAAAAAAAAAAAAAAAA/9oADAMBAAIRAxEAPwCdABmX/9k='

// 获取设备图片URL，如果超时则返回黑色图片
const getDeviceImage = (deviceIp) => {
    const now = Date.now()
    const lastUpdate = deviceLastUpdate.value[deviceIp]
    const screenshotInterval = AppConfig.config.screenshotInterval || 500
    
    // 如果超过截图频率+1秒没有更新，显示黑色图片
    if (lastUpdate && (now - lastUpdate) > (screenshotInterval + 1000)) {
        return blackImageBase64
    }
    
    return deviceImages.value[deviceIp] || blackImageBase64
}

// 监听设备预览事件
let heartbeatInterval;

onMounted(async () => {
    await listen("device_preview", (e) => {
        const [deviceIp, imageUrl] = e.payload
        deviceImages.value[deviceIp] = imageUrl
        deviceLastUpdate.value[deviceIp] = Date.now()
    })
    
    await listen("device_preview_clear", (e) => {
        const deviceIp = e.payload
        delete deviceImages.value[deviceIp]
        delete deviceLastUpdate.value[deviceIp]
    })
    
    heartbeatInterval = setInterval(() => {
        // 触发重新计算
    }, 1000)
})

onUnmounted(() => {
    if (heartbeatInterval) {
        clearInterval(heartbeatInterval)
    }
})

const SendHandle = async (ip, key) => {
    Mask_data.main_mask = true
    currentDeviceIp.value = ip
    
    if (key === 'see') {
        await run_see_video(ip, key)
    } else if (key === 'get_script') {
        // 发送获取脚本命令
        await invoke("sned_fn", {
            ip, key
        })
    } else if (key === 'sync_files') {
        // 发送同步文件命令
        await invoke("sned_fn", {
            ip, key: 'sync_files'
        })
    } else {
        await invoke("sned_fn", {
            ip, key
        })
    }

    Mask_data.main_mask = false
}

// 执行脚本
const executeScript = async (deviceIp, script) => {
    try {
        console.log('执行脚本:', script, '到设备:', deviceIp)
        
        // 确保脚本对象有id字段
        if (!script.id) {
            console.error('脚本缺少id字段:', script)
            ElNotification({
                type: 'error',
                title: '执行失败',
                message: `脚本缺少ID字段，无法执行`,
                duration: 3000
            })
            return
        }
        
        // 发送脚本执行命令
        // 格式为 script_execute|{script_id}
        const cmd = `script_execute|${script.id}`
        console.log('发送执行命令:', cmd, '到设备:', deviceIp)
        
        await invoke("sned_fn", {
            ip: deviceIp,
            key: cmd
        })
        
        ElNotification({
            type: 'success',
            title: '执行成功',
            message: `脚本 "${script.name}" 执行命令已发送到设备 ${deviceIp}`,
            duration: 3000
        })
    } catch (error) {
        console.error('执行脚本失败:', error)
        ElNotification({
            type: 'error',
            title: '执行失败',
            message: `脚本执行失败, 等待同步后再执行: ${error.message}`,
            duration: 3000
        })
    }
}

const run_see_video = async (ip, key) => {
    let isRun_Viode = await WebviewWindow.getByLabel("vnc-window")
    if (isRun_Viode) {
        ElNotification({
            type: 'warning',
            title: 'tips',
            message: "远程窗口已经打开, 请先关闭",
            duration: 3000
        })

        Mask_data.main_mask = false
        return
    }

    let res = await invoke("create_video_data_flow", {
        ip, key
    })

    if (!res.state) return ElNotification({
        type: 'error',
        title: 'tips',
        message: res.mes,
        duration: 3000
    })

    await new Promise((resolve) => setTimeout(() => resolve(), 1500))
    isRun_Viode = await WebviewWindow.getByLabel("video")
    console.log(isRun_Viode)
    if (isRun_Viode) {
        await isRun_Viode.show()
    }
}
</script>

<style lang="scss" scoped>
.devices_list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 56px;
    padding: 16px;

    .device {
        position: relative;
        width: 100%;
        aspect-ratio: 16 / 9;
        border-radius: 8px;
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
        background-color: #f0f0f0;
        margin-bottom: 55px;

        .device_info {
            position: absolute;
            bottom: -60px;
            left: 50%;
            transform: translateX(-50%);
            height: 55px;

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

            button {
                padding: 8px 12px;
                background-color: #ffffffcc;
                border: none;
                border-radius: 4px;
                cursor: pointer;
                font-weight: bold;
                transition: background-color 0.2s ease;

                &:hover {
                    background-color: #fff;
                }
            }

            &>div {
                padding: 0 50px;
                width: 100%;
                display: flex;
                justify-content: space-evenly;
            }
        }

        &:hover .control-overlay {
            opacity: 1;
        }
    }
}
</style>