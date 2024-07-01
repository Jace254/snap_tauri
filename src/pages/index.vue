<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { listen } from '@tauri-apps/api/event'

const canvas = ref<HTMLCanvasElement>()
const recording = ref(false)
const unlisten = ref<UnlistenFn>()
const loadRecording = ref(true)
const frameCount = ref(0)
const drawnCount = ref(0)

async function setupFrameListener() {
  const c = canvas.value?.getContext('2d')
  c!.fillStyle = '#000'
  let hasntPlayed = true

  unlisten.value = await listen('frame', (event) => {
    frameCount.value += 1
    const frame = JSON.parse(event.payload as string)
    const data = new Uint8Array(frame.data)
    const width = frame.width as number
    const height = frame.height as number
    if (loadRecording.value && recording.value) {
      loadRecording.value = false
    }
    if (hasntPlayed) {
      canvas.value!.width = width
      canvas.value!.height = height
      hasntPlayed = false
    }
    const videoFrame = new VideoFrame(data.buffer, {
      timestamp: frame.pts as number,
      codedHeight: height,
      codedWidth: width,
      format: 'BGRA',
      displayWidth: width,
      displayHeight: height,
    })

    createImageBitmap(videoFrame).then((imageBitmap) => {
      videoFrame.close()

      function draw() {
        c?.clearRect(0, 0, width, height)
        c?.fillRect(0, 0, width, height)
        c?.drawImage(imageBitmap, 0, 0)
        drawnCount.value += 1
      }

      window.requestAnimationFrame(draw)
    })
  })
}

async function startRecording() {
  recording.value = true
  invoke('start_recording').catch(err => console.error(err))
}

function stopRecording() {
  recording.value = false
  loadRecording.value = true
  // video.value?.pause()
  invoke('stop_recording').catch(err => console.error(err))
}

onMounted(async () => {
  await setupFrameListener()
})
onUnmounted(() => {
  if (unlisten.value)
    unlisten.value!()
})
</script>

<template>
  <div>
    <div class="relative h-[100vh] w-full flex flex-col items-center justify-center bg-black">
      <div class="h-50px w-full flex items-center justify-center gap-2 border-b border-border bg-accent py-2">
        <template v-if="recording">
          <Button
            rounded
            bg-rose-7
            class="disabled:bg-rose-7:40"
            p2 text-white :disabled="loadRecording"
            @click="stopRecording"
          >
            <p v-if="!loadRecording">
              Stop Recording
            </p>
            <p v-else>
              Loading
            </p>
          </Button>
        </template>
        <template v-if="!recording">
          <Button
            rounded
            bg-primary p2
            text-black @click="startRecording"
          >
            Start Recording
          </Button>
        </template>
        <p>Frame Count {{ frameCount }}</p>
        <p>Drawn Count {{ drawnCount }}</p>
      </div>
      <canvas
        ref="canvas"
        class="h-[calc(100%_-_50px)] w-[100%]"
      />
    </div>
  </div>
</template>
