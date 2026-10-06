<template>
  <!-- focusin/focusout bubble from the inner input (EP re-emits focus, but listening on the
       wrapper keeps it native); keydown handlers likewise ride the bubble via fallthrough -->
  <div class="am-expr" @focusin="onFocus" @focusout="onBlur">
    <el-input
      ref="inputRef"
      :model-value="modelValue"
      :placeholder="placeholder"
      clearable
      :prefix-icon="Collection"
      spellcheck="false"
      @update:model-value="onInput"
      @clear="onClear"
      @keydown.up.prevent="move(-1)"
      @keydown.down.prevent="move(1)"
      @keydown.enter="onEnter"
      @keydown.esc="close"
    />
    <el-tooltip :content="help" placement="bottom" :disabled="!help">
      <span class="am-expr-help"
        ><el-icon><InfoFilled /></el-icon
      ></span>
    </el-tooltip>
    <!-- IDE-style completion for the tag name being typed; mousedown.prevent keeps input focus -->
    <div v-if="open && items.length" class="am-expr-pop">
      <button
        v-for="(item, i) in items"
        :key="item"
        type="button"
        class="am-expr-item"
        :class="{ active: i === active }"
        @mousedown.prevent="complete(item)"
        @mouseenter="active = i"
      >
        {{ item }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'
import { Collection, InfoFilled } from '@element-plus/icons-vue'

// Tag expression box with IDE-style tag-name completion: while typing a tag name, a dropdown
// suggests the known tags; selecting one replaces only the token at the caret, so longer
// expressions keep their surroundings. Applied on Enter / clear / selection; typing applies
// through the debounced watcher at the panel layer.
const props = defineProps<{
  modelValue: string
  /** Known tag names, the suggestion source */
  tags: string[]
  placeholder?: string
  /** Syntax help text shown next to the box */
  help?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [string]
  /** Apply the current expression now (Enter / clear / suggestion picked) */
  apply: []
}>()

const inputRef = ref<{ input?: HTMLInputElement } | null>(null)
const open = ref(false)
const active = ref(0)
const token = ref('')
/** True while completing the body of a /regex/ atom: no tag suggestions there */
const regexBody = ref(false)
/** Focus on an empty box lists the whole taxonomy as a starting point */
const showAll = ref(false)

function caret(): number {
  const el = inputRef.value?.input
  // happy-dom (tests) may not track selection: end-of-value is the right fallback while typing
  return el?.selectionStart ?? props.modelValue?.length ?? 0
}

/** The tag-name span around the caret, plus the delimiter right before it (quote for re-closing,
 * slash for regex-mode detection). */
function currentToken(value: string, at: number): { start: number; end: number; quote: string; prev: string } | null {
  const delims = new Set([' ', '\t', '(', ')', '&', '|', '!', '"', "'"])
  let start = at
  while (start > 0 && !delims.has(value[start - 1]!)) start--
  if (start >= at) return null
  const prev = start > 0 ? value[start - 1]! : ''
  const quote = prev === '"' || prev === "'" ? prev : ''
  return { start, end: at, quote, prev }
}

const items = computed(() => {
  if (showAll.value) return props.tags
  // Inside a regex atom (the token itself starts with '/', or the delimiter before it is the
  // slash that opened one) the word is a pattern, not a tag name: no suggestions
  if (token.value.startsWith('/') || regexBody.value) return []
  const q = token.value.toLowerCase()
  if (!q) return []
  const starts: string[] = []
  const incl: string[] = []
  for (const t of props.tags) {
    const lower = t.toLowerCase()
    if (lower === q) continue
    if (lower.startsWith(q)) starts.push(t)
    else if (lower.includes(q)) incl.push(t)
  }
  return [...starts, ...incl]
})

function onInput(value: string) {
  emit('update:modelValue', value)
  showAll.value = false
  const tok = currentToken(value, caret())
  token.value = tok ? value.slice(tok.start, tok.end) : ''
  regexBody.value = tok ? tok.prev === '/' : false
  active.value = 0
  open.value = items.value.length > 0
}

function onFocus() {
  if (!props.modelValue && props.tags.length) {
    showAll.value = true
    active.value = 0
    open.value = true
  }
}

function onBlur() {
  // mousedown.prevent keeps item clicks from blurring; the delay only guards programmatic focus
  setTimeout(() => {
    open.value = false
    showAll.value = false
  }, 120)
}

function close() {
  open.value = false
  showAll.value = false
}

function move(delta: number) {
  if (!open.value || items.value.length === 0) return
  active.value = (active.value + delta + items.value.length) % items.value.length
}

function complete(item: string) {
  const value = props.modelValue ?? ''
  const tok = currentToken(value, caret())
  // Completing inside a quoted name closes the quote; the name itself goes in bare —
  // names needing quotes never contain them unescaped mid-name
  const insert = tok?.quote ? item + tok.quote : item
  const next = tok ? value.slice(0, tok.start) + insert + value.slice(tok.end) : value + item
  const at = (tok?.start ?? value.length) + insert.length
  emit('update:modelValue', next)
  close()
  emit('apply')
  void nextTick(() => {
    const el = inputRef.value?.input
    el?.focus()
    el?.setSelectionRange(at, at)
  })
}

function onEnter() {
  if (open.value && items.value.length > 0) {
    complete(items.value[active.value] ?? items.value[0]!)
    return
  }
  emit('apply')
}

function onClear() {
  emit('update:modelValue', '')
  close()
  emit('apply')
}
</script>

<style scoped>
.am-expr {
  position: relative;
  display: flex;
  align-items: center;
  gap: 4px;
}
.am-expr > .el-input {
  flex: 1;
}
.am-expr-help {
  display: inline-flex;
  color: var(--el-text-color-placeholder);
  cursor: help;
}
.am-expr-pop {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  z-index: 2100;
  min-width: 100%;
  max-height: 240px;
  overflow-y: auto;
  background: var(--el-bg-color-overlay);
  border: 1px solid var(--el-border-color-light);
  border-radius: var(--el-border-radius-base);
  box-shadow: var(--el-box-shadow-light);
  padding: 4px 0;
  display: flex;
  flex-direction: column;
}
.am-expr-item {
  /* The pop is a flex column capped by max-height: without flex:none the items shrink to fit
     instead of overflowing into a scrollbar, squashing their text once the list gets long */
  flex: none;
  border: none;
  background: none;
  text-align: left;
  font: inherit;
  color: var(--el-text-color-regular);
  padding: 6px 14px;
  cursor: pointer;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.am-expr-item.active {
  background: var(--el-fill-color-light);
  color: var(--el-text-color-primary);
}
</style>
