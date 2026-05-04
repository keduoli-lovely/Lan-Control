<template>
    <div class="setting" :class="{ 'setting_atv': !tap_show }">
        <div class="setting_page">
            <router-view></router-view>
        </div>
        <div class="setting_box">
            <div class="setting_title">设置</div>

            <el-menu default-active="1" class="el-menu-vertical-demo">
                <el-menu-item index="1" @click="router.push('/')">
                    <el-icon>
                        <Setting />
                    </el-icon>
                    <span>设备端口</span>
                </el-menu-item>
                <el-menu-item index="2" @click="router.push('/select')">
                    <el-icon>
                        <setting />
                    </el-icon>
                    <span>快捷指令</span>
                </el-menu-item>
                <el-menu-item index="3" @click="router.push('/add')">
                    <el-icon>
                        <setting />
                    </el-icon>
                    <span>自定义指令</span>
                </el-menu-item>
            </el-menu>
        </div>


        <div class="show_icon">
            <el-icon class="icon" :class="{ 'icon_atv': tap_show }" @click="tap_show = !tap_show">
                <Histogram />
            </el-icon>
        </div>
    </div>
</template>

<script setup>
import { Histogram, Setting } from '@element-plus/icons-vue'
import { useRouter } from 'vue-router';
import { ref } from 'vue';
const tap_show = ref(false)
const router = useRouter()

</script>

<style lang="scss" scoped>
.setting_atv {
    right: -100% !important;
}

.setting {
    display: flex;
    position: fixed;
    top: 32px;
    right: 0;
    height: 100%;
    transition: right .3s ease;
    background-color: #fff;

    .setting_box {
        user-select: none;
        box-sizing: border-box;
        min-width: 14vw;
        height: 100%;

        .setting_title {
            padding-left: 2px;
            font-size: 22px;
            height: 50px;
            line-height: 50px;
            font-weight: bold;
        }

    }

    .setting_page {
        top: 0;
        left: 0;
        width: calc(100vw - 14vw);
        height: 100%;
    }

    .show_icon {
        position: fixed;
        top: 42px;
        right: 10px;
        display: flex;
        font-size: 28px;
        color: rgba(135, 206, 235, .9);

        .icon {
            transform: rotate(-90deg);
            transition: color .3s ease;
            cursor: pointer;
            transition: transform .3s ease;

            &:hover {
                color: skyblue;
            }

            &::before {
                content: "";
                position: absolute;
                top: 50%;
                left: 50%;
                width: 200%;
                height: 200%;
                background: radial-gradient(circle, rgba(0, 0, 0, 0.4) 0%, transparent 60%);
                transform: translate(-50%, -50%);
                opacity: 0;
                transition: opacity 0.4s ease;
                pointer-events: none;
            }

            &:hover::before {
                opacity: 1;
            }
        }

        .icon_atv {
            transform: rotate(90deg);
            color: skyblue;
        }
    }
}


@media (max-width: 1000px) {
    .setting {
        .setting_box {
            min-width: 24vw;
        }

        .setting_page {
            width: calc(100vw - 24vw);
        }
    }
}
</style>