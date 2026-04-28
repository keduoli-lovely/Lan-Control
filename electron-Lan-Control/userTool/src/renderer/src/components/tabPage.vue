<template>
    <transition name="slide">
        <div v-if="visible" class="drawer">
            <div class="setName">
                <div class="title" style="margin-bottom: 4px;">
                    设置设备名称:
                </div>
                <el-input v-model="deviceName" style="width: 260px" :placeholder="ConfigAll.nikename || '未获取到默认设备名称'" />

                <div class="setNameBtn" style="margin-top: 8px;">
                    <el-button type="primary" :disabled="!deviceName && deviceName != ConfigAll.nikename" plain
                        @click="sendCodeObj('setname', { name: deviceName })">设置名称</el-button>
                </div>
            </div>

            <div style="height: 10px;"></div>

            <div class="autorun">
                <div>开机自启</div>
                <div style="width: 8px;"></div>
                <el-switch v-model="ConfigAll.autorun" class="ml-2" @change="sendCodeObj('autorun', { RunValue: ConfigAll.autorun })"
                    style="--el-switch-on-color: #A0CFFF; --el-switch-off-color: #F4F4F5" />
            </div>

            <div class="setPart">
                <div class="title" style="margin-bottom: 4px;">
                    设置设备端口(未获取端口才会使用):
                </div>

                <div class="partRow">
                    <div class="text" style="font-size: 14px;color: rgba(0, 0, 0, 0.6);margin-bottom: 4px;">广播端口(默认即可):
                    </div>
                    <el-input v-model="AdPart" style="width: 160px"
                        :placeholder="numberToString(ConfigAll.AdPart, '未获取广播端口')" />
                </div>
                <div style="height: 5px;"></div>
                <div class="partRow">
                    <div class="text" style="font-size: 14px;color: rgba(0, 0, 0, 0.6);margin-bottom: 4px;">
                        WebSocket端口(默认即可):</div>
                    <el-input v-model="wspart" style="width: 160px"
                        :placeholder="numberToString(ConfigAll.wspart, '未获取Ws端口')" />
                </div>

                <div class="setNameBtn" style="margin-top: 8px;display: flex;">
                    <el-button type="primary" plain :disabled="portisDisabled"
                        @click="sendCodeObj('update', { AdPart, wspart })">设置端口</el-button>
                    <div style="width: 12px;"></div>
                    <el-button type="info" plain
                        :disabled="ConfigAll.AdPart === ConfigAll.defAdPart && ConfigAll.wspart === ConfigAll.defwspart"
                        @click="sendCodeObj('Reset', 0)">重置端口</el-button>
                </div>
            </div>

            <div style="height: 10px;"></div>

            <div class="setRun">
                <div class="title" style="margin-bottom: 4px;">
                    设置额外运行脚本:
                </div>

                <el-button plain @click="openConfigFile">打开配置文件</el-button>

                <div style="height: 6px;"></div>

                <div style="font-size: 14px;color: red;">通过页面配置需将(bat /
                    exe)等可运行文件放入指定文件夹中/如需使用绝对路径则请手动修改配置文件</div>

                <div style="height: 6px;"></div>

                <div class="runRow">
                    <div>命令名称: <el-input v-model="CodeObj.name" style="width: 200px" placeholder="运行Aida64" /></div>
                    <div style="height: 6px;"></div>
                    <div>
                        文件名称: <el-input v-model="CodeObj.file" style="width: 200px"
                            placeholder="RunAdia64.(bat / exe)" />

                    </div>

                    <div style="margin-top: 14px;display: flex;justify-content: right;padding-right: 40px;">
                        <el-button type="primary" plain @click="sendCodeObj('Add', CodeObj)"
                            :disabled="!CodeObj.name || !CodeObj.file">添加</el-button>
                    </div>
                </div>
            </div>
        </div>
    </transition>
</template>

<script setup>
import { ref, computed } from 'vue'
import { ElNotification } from 'element-plus'


const Runvalue2 = ref(false);
const deviceName = ref('')
const AdPart = ref('')
const wspart = ref('')
const ConfigAll = ref({})
// 命令对象
const CodeObj = ref({
});
// 映射对象
const mapObj = {
    'Add': {
        success: '脚本添加成功',
        error: '脚本添加失败'
    },
    'update': {
        success: '配置更新成功',
        error: '配置更新失败'
    },
    'reboot': {
        success: '端口重置成功',
        error: '端口重置失败'
    },
    'setname': {
        success: '名称修改成功',
        error: '名称修改失败'
    },
    'autorun': {
        success: '更新成功',
        error: '更新失败'
    }
}
const props = defineProps({
    visible: Boolean,
});

const portisDisabled = computed(() => {
    console.log(ConfigAll.value)
    if (!ConfigAll.value) return true
    const noInput = !AdPart.value || AdPart.value.trim() === '';
    const inputEqualsDef = AdPart.value === ConfigAll.value.AdPart;

    const noInput1 = !wspart.value || wspart.value.trim() === '';
    const inputEqualsDef1 = wspart.value === ConfigAll.value.wspart;
    return (noInput || inputEqualsDef) && (noInput1 || inputEqualsDef1);
})


// 打开目录
const openConfigFile = () => {
    window.electron.ipcRenderer.send('openPath')
}

// 数字转字符串-如果传入为空返回提示
const numberToString = (num, text) => {
    if (!num) return text;
    return num.toString();
}


async function sendCodeObj(key, value) {
    const res = await window.api.command([key, JSON.stringify(value)])
    const mapItem = mapObj[key]

    // 公共通知函数
    const notify = (success, text) => {
        ElNotification({
            title: success ? mapItem.success : mapItem.error,
            message: text,
            type: success ? 'success' : 'error',
        })
    }

    if (!res.state) {
        notify(false, res.text)
        return
    }

    // 成功时的分支逻辑
    switch (key) {
        case 'update':
            if (AdPart.value) {
                ConfigAll.value.AdPart = AdPart.value.toString()
                AdPart.value = ''
            }
            if (wspart.value) {
                ConfigAll.value.wspart = wspart.value.toString()
                wspart.value = ''
            }
            break

        case 'Reset':
            if (ConfigAll.value.AdPart !== ConfigAll.value.defAdPart) {
                ConfigAll.value.AdPart = ConfigAll.value.defAdPart
            }
            if (ConfigAll.value.wspart !== ConfigAll.value.defwspart) {
                ConfigAll.value.wspart = ConfigAll.value.defwspart
            }
            break

        case 'Add':
            CodeObj.value = {}
            break

        case 'setname':
            ConfigAll.value.nikename = deviceName.value
            deviceName.value = ''
            break

        // 其他 key 可以在这里扩展
        default:
            console.warn(`未处理的 key: ${key}`)
    }

    notify(true, res.text)
}

window.api.defaultConfigFn('defaultConfig', (data) => {
    ConfigAll.value = data
})
</script>

<style scoped lang="scss">
.drawer {
    padding: 20px;
    position: fixed;
    top: 0;
    right: 0;
    width: 350px;
    height: 100%;
    background: #fff;
    box-shadow: -2px 0 8px rgba(0, 0, 0, 0.1);
    z-index: 1000;
    display: flex;
    flex-direction: column;

    .setName {
        .setNameBtn {
            display: flex;
        }
    }
}

.drawer-content {
    padding: 16px;
    flex: 1;
    overflow-y: auto;
}

.slide-enter-active,
.slide-leave-active {
    transition: transform 0.3s ease;
}

.slide-enter-from,
.slide-leave-to {
    transform: translateX(100%);
}
</style>
