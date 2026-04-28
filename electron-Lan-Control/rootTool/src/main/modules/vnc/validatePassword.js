/**
 * 校验不可数字开头
 * @param {string} password - 要校验的密码
 * @returns {boolean} - 校验通过返回 true，校验不通过抛出错误
 */

const validatePassword = (password) => {
  if (password.length <= 0 || password.length > 7) {
    throw new Error('密码长度建议在 1~7 位之间');
  }
  return true;
}

export { validatePassword };