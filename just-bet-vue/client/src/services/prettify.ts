export const prettify = (value: number) => {
  if (!value) return 0
  // data stored on server are now in cents rather than ££
  value /= 100
  if (value >= 10) {
    return truncate(value, 0)
      .toString()
      .replace(/(\d)(?=(\d{3})+$)/g, '$1,')
  } else if (value >= 0) {
    return value % 1 === 0 ? value : truncate(value, 2)
  }
  return Number(`-${prettify(value * -1)}`)
}

function truncate(value: number, point: number) {
  const decimal = value.toString().split('.')
  if (decimal.length !== 2) return value
  if (point === 0) return Number(decimal[0])
  decimal[1] = decimal[1].substring(0, point)
  return Number(decimal.join('.'))
}
