<template>
    <dialog ref="dialogRef" class="modal" @click.self="loading ? '' : modelShow = false" v-loading="loading"
        :style="!showModelPage ? (!isFull ? SetWindowSize.max : SetWindowSize.min) : ''">
        <div class="modal-content" v-if="showModelPage">
            <h2>自定义执行命令</h2>
            <p>如需更多执行命令，请在被控端软件设置中添加....</p>
            <div style="height: 20px;"></div>
            <div class="scriptBtn">
                <div v-if="Object.keys(scriptListData).length > 0">
                    <button v-for="item in scriptListData" :key="item.name" v-show="item.name"
                        @click="checkDeviceStatus(ActiveIp, item.name)">
                        {{ item.name }}
                    </button>

                    <button @click="syncScript" style="color: #B0D5FB;" v-if="!ActiveSyncSta">开始同步脚本</button>
                    <button title="已完成同步" disabled="false" v-else>已完成同步</button>
                </div>

                <div v-else>
                    <el-text class="mx-1" type="danger" size="large">没有添加执行命令, 请前往被控端软件添加</el-text>

                </div>
            </div>
            <el-button class="closeBtn" type="info" @click="modelShow = false" size="large" plain>关闭</el-button>
        </div>

        <div v-else class="settings-card">
            <div class="left-panel">
                <el-form :model="form" :rules="rules" ref="formRef" class="left-panel" label-width="80px">
                    <h3 class="section-title">连接设置</h3>
                    <div style="height: 25px;"></div>
                    <el-form-item label="广播端口" prop="AdPart" class="form-group">
                        <el-input class="form-input" :autofocus="false" type="number" v-model.number="form.AdPart"
                            placeholder="2000~65534" />
                    </el-form-item>

                    <el-form-item label="WS端口" prop="wspart" class="form-group">
                        <el-input class="form-input" :autofocus="false" type="number" v-model.number="form.wspart"
                            placeholder="2000~65534" />
                    </el-form-item>

                    <el-form-item label="连接密码" prop="LinkPwd" class="form-group">
                        <el-input class="form-input" :autofocus="false" type="password" v-model="form.LinkPwd"
                            placeholder="密码会被加密" show-password />
                    </el-form-item>

                    <el-form-item label="心跳间隔" prop="BROADCAST_INTERVAL" class="form-group">
                        <el-input class="form-input" :autofocus="false" type="number"
                            v-model.number="form.BROADCAST_INTERVAL" placeholder="500" />
                    </el-form-item>

                    <el-form-item label="广播间隔" prop="ADRunTime" class="form-group">
                        <el-input class="form-input" :autofocus="false" type="number" v-model.number="form.ADRunTime"
                            placeholder="500" />
                    </el-form-item>

                    <el-form-item label="Http端口" prop="HttpPort" class="form-group switch-group">
                        <el-input class="form-input" :autofocus="false" type="number" v-model.number="form.HttpPort"
                            placeholder="500" />
                    </el-form-item>


                </el-form>

                <div style="height: 10px;"></div>
                <el-button style="margin-left: 10px;" disabled class="form-switch">检查更新</el-button>

            </div>

            <div class="right-panel" v-loading="!form?.HttpSta">
                <div style="height: 4px;"></div>
                <p style="font-size: 14px;color: #17a2b8;">向所有连接设备追加自定义执行脚本</p>
                <div style="height: 12px;"></div>
                <div class="sendFileBox">
                    <el-form-item label="脚本名称" prop="AdPart" class="form-group" style="margin-bottom: 8px;">
                        <el-input class="form-input" :autofocus="true" type="text" v-model="fileObj.filename"
                            placeholder="运行Aida64" />
                    </el-form-item>
                    <el-form-item label="脚本文件" prop="AdPart" class="form-group" style="margin-bottom: 8px;">
                        <el-button class="selectBtn" @click="selectFile([true, 'select'])"
                            :disabled="fileObj.fielOrDoc === 1">
                            <div class="selectBtnText">选择文件</div>
                        </el-button>
                        <el-button class="selectBtn" @click="selectFile([false, 'select'])"
                            :disabled="fileObj.fielOrDoc === 2">
                            <div class="selectBtnText">选择文件夹</div>
                        </el-button>
                    </el-form-item>
                    <div style="font-size: 14px;color: red;margin-bottom: 5px;">选择文件夹时需带上应用名称/选择文件则不用</div>

                    <el-form-item label="执行参数" prop="AdPart" class="form-group">
                        <el-input class="form-input" :autofocus="false" type="text" v-model="fileObj.key"
                            placeholder="Adia64.exe /run(可选)" />
                    </el-form-item>
                </div>
                <el-button class="action-btn" @click="sendScript" :disabled="!sendBtn">提交脚本</el-button>
                <div v-show="!sendBtn" style="margin-left: 12px;font-size: 12px;display: inline-block;color: red;">
                    请选择文件/设置文件名称</div>
                <div style="height: 8px;"></div>
                <el-button class="syncBtn-btn" :disabled="!form?.HttpSta" @click="syncScript">同步脚本</el-button>

                <div style="height: 12px;"></div>
                <div class="link-group">
                    <h5>相关链接</h5>
                </div>

                <div class="linkbox">
                    <el-link type="primary" @click="text">被控端下载</el-link>&nbsp;&nbsp;|&nbsp;&nbsp;
                    <el-link type="primary" @click="text">项目地址</el-link>
                </div>

            </div>
        </div>
        <div class="saveBtn" v-if="saveConfig && !showModelPage">
            <el-button class="closeBtn" type="info" @click="saveSettings" size="large" plain>保存设置</el-button>
        </div>

    </dialog>
</template>

<script setup>
import {
    form,
    saveConfig,
    dialogRef,
    scriptListData,
    checkDeviceStatus,
    saveSettings,
    setupWatchers,
    SetWindowSize,
    rules,
    fileObj,
    selectFile,
    sendScript,
    sendBtn
} from './ModalDialog.script.js'
import {
    modelShow,
    ActiveIp,
    showModelPage,
    isFull,
    loading,
    syncScript,
    ActiveSyncSta
} from '../utils/popupStore.js'

const text = async (i) => {
    await window.api.command(["https://keduoli.com", 'openLink'])
}

setupWatchers()
</script>

<style lang="scss" scoped>
.modal {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -60%);
    width: 80vw;
    height: 60vh;
    background: #fff;
    border: none;
    border-radius: 8px;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
    z-index: 1001;
    transition: opacity 0.3s ease, transform 0.3s ease;
    backdrop-filter: blur(10px);

    &::backdrop {
        background: rgba(0, 0, 0, 0.5);
    }

    &.open {
        opacity: 1;
        transform: translate(-50%, -50%);
        pointer-events: auto;
    }

    &::backdrop {
        background: rgba(0, 0, 0, 0.5);
    }

    .modal-content {
        padding: 20px;
        width: 100%;
        height: 100%;
        box-sizing: border-box;

        .closeBtn {
            position: absolute;
            right: 50px;
            bottom: 30px;
        }

        .scriptBtn {
            display: flex;
            flex-wrap: wrap;
            padding: 18px 20px;
            max-height: 60%;
            backdrop-filter: blur(10px);
            border-radius: 6px;

            button {
                margin: 0 18px 16px 0;
                padding: 8px 12px;
                background-color: #ffffffcc;
                border: none;
                border-radius: 4px;
                cursor: pointer;
                font-weight: bold;
                box-shadow: rgba(0, 0, 0, 0.24) 0px 3px 8px;
                transition: all 0.2s ease;

                &:hover {
                    transform: scale(1.05);
                }
            }
        }

    }

    .settings-card {
        width: 100%;
        height: 100%;
        display: flex;
        flex-direction: row;
        gap: 20px;
        padding: 20px;
        padding-top: 10px;
        box-sizing: border-box;
        background: #fff;
        border-radius: 8px;
        overflow: hidden;

        .left-panel,
        .right-panel {
            flex: 1;
            overflow-y: auto;
        }

        .left-panel {
            .form-group {
                display: flex;
                justify-content: left;
                margin-bottom: 10px;

                &:nth-child(8) {
                    margin-bottom: 5px;
                }
            }
        }

        .right-panel {
            background-color: #f9f9f9;
            border-radius: 6px;
            padding: 16px;

            .selectBtn {
                width: 80px;

                .selectBtnText {
                    // width: 200px;
                    margin-right: 4px;
                    white-space: nowrap;
                    text-overflow: ellipsis;
                    overflow: hidden;
                }

                .link-group {
                    display: flex;
                    justify-content: center;
                }

                .linkbox {
                    margin-top: 2px;
                    display: flex;
                    justify-content: center;
                }
            }
        }
    }
}

.saveBtn {
    position: fixed;
    bottom: 22px;
    left: 235px;
}
</style>
