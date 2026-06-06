<template>
  <div class="page-container active mesugaki-page">
    <div class="mesugaki-container">
      <div class="mesugaki-face" :class="{ shake: isShaking }" @click="showQuote">
        <div class="mesugaki-emoji">{{ currentEmoji }}</div>
      </div>
      <div class="mesugaki-bubble">
        <div id="mesugakiText" class="mesugaki-text" :class="{ animate: animateQuote }" v-html="currentQuote"></div>
      </div>
      <div class="mesugaki-hint">点击她可以换一句话哦~</div>
      <div class="mesugaki-counter" id="mesugakiCounter">{{ counterText }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

const quotes = [
  '啊嘞嘞～这么想要别的功能呀～<br>真是杂鱼呢～♡',
  '诶～杂鱼先生还想要更多工具？<br>就凭你也配吗～♡',
  '嘻嘻～点进来什么都没有哦～<br>杂鱼就是杂鱼呢～♡',
  '哈？你以为这里会有什么新功能？<br>真是天真呢～杂鱼♡',
  '呜哇～杂鱼先生好贪心啊～<br>明明有转换器和播放器还不够吗～♡',
  '杂鱼杂鱼～点多少次都不会有的哦～<br>新功能什么的～♡',
  '欸嘿嘿～被杂鱼盯着看了呢～<br>好恶心哦～♡',
  '杂鱼先生～你是不是以为<br>多按几次就会出新功能呀～<br>真是可怜呢～♡',
  '啊啦～杂鱼又来了呢～<br>这么闲的话不如去转换几个文件？♡',
  '哼～杂鱼的需求倒是挺多的～<br>可惜本小姐不想做呢～♡',
  '杂鱼先生～你的表情好期待啊～<br>但是呢～什么都没有哦～♡',
  '噗～杂鱼居然还在点？<br>毅力倒是不错呢～<br>虽然毫无意义就是了～♡',
  '杂鱼～你知道吗？<br>贪心的杂鱼是没有糖吃的哦～♡',
  '嗯？杂鱼想要新功能？<br>那先夸本小姐一百遍再说吧～♡',
  '啊～杂鱼的眼神好可怜啊～<br>但是本小姐就是不给呢～♡',
]

const emojis = ['😏', '😈', '🤭', '😜', '💅', '🙄', '😤', '👿', '😼', '🫣']

const currentQuote = ref('杂鱼，有什么新功能想要呀～<br>点击本小姐试试看吧～♡')
const currentEmoji = ref(emojis[0])
const animateQuote = ref(false)
const isShaking = ref(false)
const clickCount = ref(0)
let lastQuoteIndex = -1

function showQuote() {
  clickCount.value++

  // 抖动动画
  isShaking.value = false
  void 0 // 触发 reflow
  setTimeout(() => { isShaking.value = true }, 0)

  let idx: number
  do {
    idx = Math.floor(Math.random() * quotes.length)
  } while (idx === lastQuoteIndex && quotes.length > 1)

  lastQuoteIndex = idx
  currentQuote.value = quotes[idx]
  currentEmoji.value = emojis[Math.floor(Math.random() * emojis.length)]

  animateQuote.value = false
  void document.getElementById('mesugakiText')?.offsetWidth
  animateQuote.value = true
}

const counterText = computed(() => {
  return clickCount.value > 0 ? `已经被嘲笑了 ${clickCount.value} 次...` : ''
})
</script>
