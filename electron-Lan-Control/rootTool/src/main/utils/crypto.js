function encode(str) {
  return Buffer.from(str).toString('hex')
}

function decode(hex) {
  return Buffer.from(hex, 'hex').toString()
}

export { encode, decode }