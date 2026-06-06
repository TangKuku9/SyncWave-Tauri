<template>
  <span class="icon" v-html="iconHtml" :class="className"></span>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { icons } from '../utils/icons'

const props = defineProps<{
  name: string
  size?: number
  className?: string
}>()

const iconHtml = computed(() => {
  const fn = icons[props.name]
  return fn ? fn(props.size) : ''
})
</script>

<style scoped>
.icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  line-height: 0;
}
</style>
