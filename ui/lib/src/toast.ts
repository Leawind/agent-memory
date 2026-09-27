// 统一的消息提示出口：所有成功/失败反馈都经 toastSuccess/toastError 弹出，
// 保证行为一致——可点击任意位置关闭（不必瞄准右上角小叉），且起始位置
// 在站点顶栏（60px）之下，不遮挡导航。嵌入宿主顶栏更高时可整体替换本模块。
import { ElMessage, type MessageOptions } from 'element-plus'

const TOAST_OFFSET = 72

function pop(kind: 'success' | 'error', message: string) {
  let handler!: ReturnType<typeof ElMessage>
  // onClick 未进 EP 的 MessageOptions 类型，但多余 props 会作为 attrs 落到消息根元素——
  // 这正是「点击任意处关闭」的实现通道；经展开传入绕开字面量的多余属性检查
  const clickToClose = { onClick: () => handler.close() } as MessageOptions
  handler = ElMessage({
    type: kind,
    message,
    offset: TOAST_OFFSET,
    showClose: true,
    grouping: true,
    ...clickToClose,
  })
  return handler
}

export function toastSuccess(message: string) {
  return pop('success', message)
}

export function toastError(message: string) {
  return pop('error', message)
}
