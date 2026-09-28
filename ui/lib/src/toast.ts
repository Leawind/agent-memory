// Unified toast entry point: all success/failure feedback pops through toastSuccess/toastError,
// guaranteeing consistent behavior — dismissible by clicking anywhere (no need to aim for the
// small close icon in the corner), and starting below the site top bar (60px) so navigation
// is not covered. When the host's top bar is taller, replace this module wholesale.
import { ElMessage, type MessageOptions } from 'element-plus'

const TOAST_OFFSET = 72

function pop(kind: 'success' | 'error', message: string) {
  let handler!: ReturnType<typeof ElMessage>
  // onClick is not part of EP's MessageOptions type, but extra props land on the message root
  // element as attrs — which is exactly the channel for "click anywhere to close"; spread
  // instead of inline so the literal bypasses the excess property check
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
