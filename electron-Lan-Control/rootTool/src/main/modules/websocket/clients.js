// modules/websocket/clients.js
const clients = new Map()

// 注册客户端
const registerClient = (ip, ws) => {
  clients.set(ip, { ws, lastSeen: Date.now(), nikename: 'keduoli', syncSta: false })
}
// 删除客户端
const removeClient = (ip) => {
  clients.delete(ip)
}
// 获取单个客户端信息
const getClient = (ip) => {
  return clients.get(ip)
}
// 获取所有客户端信息
const getAllClients = () => {
  return clients.entries()
}
// 更新心跳时间
const updateHeartbeat = (ip) => {
  const client = clients.get(ip)
  if (client) client.lastSeen = Date.now()
}
// 页面需要的数据格式
const formatNewClientData = () => {
  return Array.from(clients.entries()).map(([ip, value]) => ({
    ip,
    name: value?.nikename || `keduoli_${Date.now()}`,
    syncSta: value?.syncSta ?? false // 默认值 false
  }))
}

// 返回clients数量
const getClientCount = () => {
  return clients.size
}

export {
  registerClient,
  removeClient,
  getClient,
  getAllClients,
  updateHeartbeat,
  formatNewClientData,
  getClientCount
}
