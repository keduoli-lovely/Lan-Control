import os from 'os'

// 获取本地 192.168 开头的 IP 地址
const getLocalIP = () => {
  const interfaces = os.networkInterfaces()
  const result = []

  // 优先收集 192.168 开头的地址
  for (const name of Object.keys(interfaces)) {
    for (const iface of interfaces[name]) {
      if (iface.family === 'IPv4' && !iface.internal && iface.address.startsWith('192.168')) {
        result.push({ name, address: iface.address })
      }
    }
  }

  // 如果没有找到，则尝试查找 WLAN/以太网/Ethernet 接口的 IPv4 地址
  if (result.length === 0) {
    for (const name of Object.keys(interfaces)) {
      if (/WLAN|以太网|Ethernet/i.test(name)) {
        for (const iface of interfaces[name]) {
          if (iface.family === 'IPv4' && !iface.internal) {
            result.push({ name, address: iface.address })
          }
        }
      }
    }
  }

  return result
}

export { getLocalIP }
