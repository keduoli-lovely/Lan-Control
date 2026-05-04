<template>
    <div class="command-config">
        <el-card class="config-card" shadow="never" body-style="padding-left: 0">
            <template #header>
                <div class="card-header">
                    <span>指令配置</span>
                    <el-button type="primary" @click="handleSave">保存配置</el-button>
                </div>
            </template>

            <el-form ref="formRef" :model="formData" :rules="rules" label-width="120px" class="config-form">
                <!-- 指令ID -->
                <el-form-item label="指令ID" prop="commandId">
                    <el-input v-model="formData.commandId" placeholder="请输入指令ID" clearable />
                </el-form-item>

                <!-- 指令名称 -->
                <el-form-item label="指令名称" prop="commandName">
                    <el-input v-model="formData.commandName" placeholder="请输入指令名称" clearable />
                </el-form-item>

                <!-- 路径选择类型 -->
                <el-form-item label="路径类型" prop="pathType">
                    <el-radio-group v-model="formData.pathType">
                        <el-radio-button value="folder">选择文件夹</el-radio-button>
                        <el-radio-button value="file">选择文件</el-radio-button>
                    </el-radio-group>
                </el-form-item>

                <!-- 选择文件夹时的强制执行文件 -->
                <el-form-item v-if="formData.pathType === 'folder'" label="执行文件" prop="executable">
                    <div class="executable-input">
                        <el-input v-model="formData.executable" placeholder="请选择执行文件" readonly class="path-input">
                            <template #append>
                                <el-button @click="handleSelectExecutable">
                                    <el-icon>
                                        <Document />
                                    </el-icon>
                                    选择文件
                                </el-button>
                            </template>
                        </el-input>
                    </div>
                </el-form-item>

                <!-- 路径选择 -->
                <el-form-item :label="formData.pathType === 'folder' ? '文件夹路径' : '文件路径'" prop="path">
                    <div class="path-selector">
                        <el-input v-model="formData.path"
                            :placeholder="formData.pathType === 'folder' ? '请选择文件夹' : '请选择文件'" readonly
                            class="path-input">
                            <template #append>
                                <el-button @click="handleSelectPath">
                                    <el-icon>
                                        <Folder />
                                    </el-icon>
                                    浏览
                                </el-button>
                            </template>
                        </el-input>
                    </div>
                </el-form-item>

                <!-- 执行参数 -->
                <el-form-item label="执行参数">
                    <div class="args-container">
                        <div v-for="(arg, index) in formData.arguments" :key="index" class="arg-item">
                            <el-input v-model="formData.arguments[index]" :placeholder="`参数 ${index + 1}`" clearable
                                class="arg-input">
                            </el-input>
                            <el-button v-if="formData.arguments.length > 1" type="danger" circle plain
                                @click="removeArgument(index)" class="remove-btn">
                                <el-icon>
                                    <Minus />
                                </el-icon>
                            </el-button>
                        </div>
                        <el-button type="primary" plain @click="addArgument" class="add-btn">
                            <el-icon>
                                <Plus />
                            </el-icon>
                            添加参数
                        </el-button>
                    </div>
                </el-form-item>

                <!-- 预览 -->
                <el-form-item label="命令预览">
                    <div class="preview-box">
                        <code>{{ commandPreview }}</code>
                    </div>
                </el-form-item>
            </el-form>
        </el-card>
    </div>
</template>

<script setup>
import { ref, reactive, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { Folder, Plus, Minus, Document } from '@element-plus/icons-vue'
import { open } from '@tauri-apps/plugin-dialog'
import { Store } from '@tauri-apps/plugin-store'
import { invoke } from '@tauri-apps/api/core'

const formRef = ref(null)

// 表单数据
const formData = reactive({
    commandId: '',
    commandName: '',
    pathType: 'folder', // 'folder' 或 'file'
    path: '',
    executable: '', // 仅在 pathType 为 folder 时有效，现在存储完整路径
    arguments: [''] // 默认一个空参数输入框
})

// 表单验证规则
const rules = {
    commandId: [
        { required: true, message: '请输入指令ID', trigger: 'blur' },
        { pattern: /^[a-zA-Z0-9_-]+$/, message: '只能包含字母、数字、下划线和横线', trigger: 'blur' }
    ],
    commandName: [
        { required: true, message: '请输入指令名称', trigger: 'blur' },
        { min: 2, max: 50, message: '长度在 2 到 50 个字符', trigger: 'blur' }
    ],
    pathType: [
        { required: true, message: '请选择路径类型', trigger: 'change' }
    ],
    path: [
        { required: true, message: '请选择路径', trigger: 'change' }
    ],
    executable: [
        { required: true, message: '请选择执行文件', trigger: 'change' }
    ]
}

// 命令预览
const commandPreview = computed(() => {
    const parts = []

    if (formData.pathType === 'folder' && formData.executable) {
        // 文件夹模式下使用选择的执行文件完整路径
        parts.push(`"${formData.executable}"`)
    } else if (formData.path) {
        parts.push(`"${formData.path}"`)
    } else {
        parts.push('[未选择路径]')
    }

    // 过滤空参数
    const validArgs = formData.arguments.filter(arg => arg.trim() !== '')
    if (validArgs.length > 0) {
        parts.push(...validArgs.map(arg => `${arg}`))
    }

    return parts.join(' ') || '暂无命令'
})

// 选择主路径（文件夹或文件）
const handleSelectPath = async () => {
    try {
        const selected = await open({
            directory: formData.pathType === 'folder',
            multiple: false,
            title: formData.pathType === 'folder' ? '选择文件夹' : '选择文件'
        })

        if (selected) {
            formData.path = selected
            // 如果切换了文件夹，清空之前选择的执行文件
            if (formData.pathType === 'folder') {
                formData.executable = ''
            }
            ElMessage.success('已选择：' + selected)
        }
    } catch (error) {
        ElMessage.error('选择失败：' + error.message)
    }
}

// 选择执行文件（仅在文件夹模式下可用）
const handleSelectExecutable = async () => {
    // 必须先选择文件夹
    if (!formData.path) {
        ElMessage.warning('请先选择文件夹')
        return
    }

    try {
        const selected = await open({
            directory: false,
            multiple: false,
            defaultPath: formData.path, // 默认打开已选择的文件夹
            title: '选择执行文件'
        })

        if (selected) {
            // 验证选择的文件是否在指定的文件夹内
            if (!selected.startsWith(formData.path)) {
                ElMessage.warning('请选择该文件夹内的文件')
                return
            }
            formData.executable = selected
            ElMessage.success('已选择执行文件：' + selected.split('/').pop())
        }
    } catch (error) {
        ElMessage.error('选择失败：' + error.message)
    }
}

// 添加参数
const addArgument = () => {
    formData.arguments.push('')
}

// 移除参数
const removeArgument = (index) => {
    formData.arguments.splice(index, 1)
}

// 保存配置
const handleSave = async () => {
    if (!formRef.value) return

    try {
        await formRef.value.validate((valid, fields) => {
            if (valid) {
                saveScriptToStore()
            } else {
                const firstError = Object.values(fields)[0][0].message
                ElMessage.error(firstError)
            }
        })
    } catch (error) {
        ElMessage.error('保存失败：' + error.message)
    }
}

// 保存脚本到store
const saveScriptToStore = async () => {
    try {
        const store = await Store.load('.server_settings.json')
        const scripts = await store.get("scripts") || {}
        
        // 创建脚本对象
        const script = {
            id: formData.commandId,
            name: formData.commandName,
            pathType: formData.pathType,
            path: formData.path,
            executable: formData.executable,
            arguments: formData.arguments.filter(arg => arg.trim() !== '')
        }
        
        // 保存到store
        scripts[formData.commandId] = script
        await store.set("scripts", scripts)
        await store.save()
        
        // 发送给所有已连接的设备
        await invoke("send_script_to_all", { script: JSON.stringify(script) })
        
        ElMessage.success('脚本保存成功并已发送给所有设备！')
        
        // 清空表单
        Object.assign(formData, {
            commandId: '',
            commandName: '',
            pathType: 'folder',
            path: '',
            executable: '',
            arguments: ['']
        })
    } catch (error) {
        ElMessage.error('保存失败：' + error.message)
    }
}
</script>

<style lang="scss" scoped>
$bg-color: #f5f7fa;
$border-color: #e4e7ed;
$primary-color: #409eff;

.command-config {
    margin-top: 10px;
    padding-bottom: 20px;
    height: 100vh;
    overflow-y: auto;
    box-sizing: border-box;
    background-color: $bg-color;

    .config-card {
        max-width: none;
        border: 1px solid $border-color;
        margin-right: 10px;

        .card-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            font-weight: bold;
            font-size: 16px;
        }

        .config-form {
            margin-top: 20px;

            .path-selector,
            .executable-input,
            .path-input {
                width: 100%;
            }

            .args-container {
                display: flex;
                flex-direction: column;
                gap: 10px;
                width: 100%;

                .arg-item {
                    display: flex;
                    align-items: center;
                    gap: 10px;

                    .arg-input {
                        flex: 1;
                    }

                    .remove-btn {
                        flex-shrink: 0;
                    }
                }

                .add-btn {
                    align-self: flex-start;
                    margin-top: 5px;
                }
            }

            .preview-box {
                background-color: $bg-color;
                border: 1px solid $border-color;
                border-radius: 4px;
                padding: 12px 16px;
                font-family: 'Courier New', Consolas, Monaco, monospace;
                font-size: 13px;
                color: $primary-color;
                word-break: break-all;
                line-height: 1.6;
            }
        }
    }
}

/* 响应式调整 */
@media (max-width: 768px) {
    .command-config {
        padding: 10px;
    }

    :deep(.el-form-item__label) {
        float: none;
        display: block;
        text-align: left;
        margin-bottom: 8px;
    }

    :deep(.el-form-item__content) {
        margin-left: 0 !important;
    }
}

::-webkit-scrollbar {
    width: 2px;
    height: 2px;

    &-thumb {
        background-color: skyblue;
        border-radius: 2px;
    }
}
</style>