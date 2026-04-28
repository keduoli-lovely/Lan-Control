<template>
    <div style="display: flex;">
        <div class="chart-container"> <canvas ref="chartRef"></canvas> </div>
        <div class="chart-container newCode">
            <div class="title">
                详细数据:
            </div>

            <div class="row">
                <div>CPU功耗:</div>
                <div style="width: 50px;display: flex;justify-content: space-between;color: #000000;">
                    <div>
                        {{ formatValue(infoData?.['CPU Package']) }}
                    </div>
                    <div>W</div>
                </div>

                <div>CPU温度:</div>
                <div style="width: 50px;display: flex;justify-content: space-between;color: #000000;">
                    <div>
                        {{ formatValue(infoData?.['Core Max']) }}
                    </div>
                    <div>℃</div>
                </div>

                <div>CPU使用率:</div>
                <div style="width: 50px;display: flex;justify-content: space-between;color: #000000;">
                    <div>
                        {{ formatValue(infoData?.['CPU Total']) }}
                    </div>
                    <div>%</div>
                </div>

                <div>内存使用率:</div>
                <div style="width: 50px;display: flex;justify-content: space-between;color: #000000;">
                    <div>
                        {{ formatValue(infoData?.['Memory']) }}
                    </div>
                    <div>%</div>
                </div>

                <div>显卡功耗:</div>
                <div style="width: 50px;display: flex;justify-content: space-between;color: #000000;">
                    <div>
                        {{ formatValue(infoData?.['GPU Package']) }}
                    </div>
                    <div>W</div>
                </div>

                <div>显卡温度:</div>
                <div style="width: 50px;display: flex;justify-content: space-between;color: #000000;">
                    <div>
                        {{ formatValue(infoData?.['GPU Hot Spot']) }}
                    </div>
                    <div>℃</div>
                </div>

                <div>显卡使用率:</div>
                <div style="width: 50px;display: flex;justify-content: space-between;color: #000000;">
                    <div>
                        {{ formatValue(infoData?.['D3D 3D']) }}
                    </div>
                    <div>%</div>
                </div>

            </div>
        </div>
    </div>
</template>

<script setup>
import { onMounted, ref } from 'vue'
import {
    Chart,
    LineController,
    LineElement,
    PointElement,
    LinearScale,
    Title,
    Tooltip,
    Legend,
    CategoryScale
} from 'chart.js'

Chart.register(LineController, LineElement, PointElement, LinearScale, Title, Tooltip, Legend, CategoryScale)

const chartRef = ref()
let chartInstance = null
let infoData = ref(null)

// 数据队列（最多保留5条）
const labels = []
const cpuLoadData = []
const cpuTempData = []
const gpuLoadData = []
const gpuTempData = []

const formatValue = (val) => {
    const num = Number(val)
    return Number.isInteger(num) ? num : num.toFixed(1)
}

function pushDataPoint(label, cpuLoad, cpuTemp, gpuLoad, gpuTemp) {
    // 保证队列长度不超过5
    if (labels.length >= 9) {
        labels.shift()
        cpuLoadData.shift()
        cpuTempData.shift()
        gpuLoadData.shift()
        gpuTempData.shift()
    }

    labels.push(label)
    cpuLoadData.push(cpuLoad)
    cpuTempData.push(cpuTemp)
    gpuLoadData.push(gpuLoad)
    gpuTempData.push(gpuTemp)

    updateChart()
}

function updateChart() {
    if (!chartInstance) return
    chartInstance.data.labels = labels
    chartInstance.data.datasets[0].data = cpuTempData
    chartInstance.data.datasets[1].data = cpuLoadData
    chartInstance.data.datasets[2].data = gpuTempData
    chartInstance.data.datasets[3].data = gpuLoadData
    chartInstance.update()
}

onMounted(() => {
    chartInstance = new Chart(chartRef.value, {
        type: 'line',
        data: {
            labels: [],
            datasets: [
                {
                    label: 'CPU 温度 (°C)',
                    data: [],
                    borderColor: 'rgba(231,76,60,0.6)',   // 柔和红
                    backgroundColor: 'rgba(231,76,60,0.1)',
                    fill: false
                },
                {
                    label: 'CPU 占用率 (%)',
                    data: [],
                    borderColor: 'rgba(243,156,18,0.6)',  // 柔和橙
                    backgroundColor: 'rgba(243,156,18,0.1)',
                    fill: false
                },
                {
                    label: 'GPU 温度 (°C)',
                    data: [],
                    borderColor: 'rgba(52,152,219,0.6)',  // 柔和蓝
                    backgroundColor: 'rgba(52,152,219,0.1)',
                    fill: false
                },
                {
                    label: 'GPU 占用率 (%)',
                    data: [],
                    borderColor: 'rgba(46,204,113,0.6)',  // 柔和绿
                    backgroundColor: 'rgba(46,204,113,0.1)',
                    fill: false
                }
            ]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            interaction: {
                mode: 'index',
                intersect: false
            },
            plugins: {
                title: {
                    display: true,
                    text: '系统性能监控'
                },
                tooltip: {
                    enabled: true,
                    callbacks: {
                        label: (context) => {
                            return `${context.dataset.label}: ${context.formattedValue}`
                        }
                    }
                },
                legend: {
                    position: 'bottom',
                    labels: {
                        color: '#333'
                    }
                }
            },
            scales: {
                y: {
                    beginAtZero: true,
                    ticks: {
                        color: '#333'
                    }
                },
                x: {
                    ticks: {
                        color: '#333'
                    }
                }
            }
        }
    })
})

// 接收数据并更新图表
window.api.InfoDataList('sensorsData', (data) => {
    const timeLabel = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
    const cpuLoad = parseFloat(data?.['CPU Total'] ?? 0)
    const cpuTemp = parseFloat(data?.['Core Max'] ?? 0)
    const gpuLoad = parseFloat(data?.['D3D 3D'] ?? 0)
    const gpuTemp = parseFloat(data?.['GPU Hot Spot'] ?? 0)

    infoData.value = data
    pushDataPoint(timeLabel, cpuLoad, cpuTemp, gpuLoad, gpuTemp)
})
</script>


<style scoped>
.chart-container {
    margin-bottom: 10px;
    margin-right: 20px;
    width: calc(70% - 20px);
    height: calc(100% - 10px);
    background-color: #fff;
    border-radius: 8px;
    backdrop-filter: blur(10px);
    box-shadow: rgba(0, 0, 0, 0.24) 0px 3px 8px;
}

.newCode {
    width: calc(30%);
    margin-right: 2px;
    padding: 10px 20px;
    color: #444444;
    background-color: #FAFAFA;
}

.title {
    margin-bottom: 10px;
    color: #222222;
}

.row {
    font-size: 14px;
    color: #444444;
}

.row>div:nth-child(odd) {
    margin-top: 5px;
    color: rgba(0, 0, 0, .6);
}
</style>
