<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { listen } from '@tauri-apps/api/event'

const canvas = ref<HTMLCanvasElement>()
const recording = ref(false)
const unlisten = ref<UnlistenFn>()
const loadRecording = ref(true)


async function setupFrameListener() {
  const c = canvas.value?.getContext('2d')
  c!.fillStyle = '#000'

  unlisten.value = await listen('frame', async (event) => {
    if (loadRecording.value && recording.value) {
      loadRecording.value = false
      console.log('started')
    }
    console.log('received')
    const frame = JSON.parse(event.payload as string)
    const data = new Uint8Array(frame.data)
    const width = frame.width as number
    const height = frame.height as number
    canvas.value!.width = width
    canvas.value!.height = height
    const videoFrame = new VideoFrame(data.buffer, {
      timestamp: frame.pts as number,
      codedHeight: height,
      codedWidth: width,
      format: 'I420',
      displayWidth: width,
      displayHeight: height,
    })

    const imageBitmap = await createImageBitmap(videoFrame)
    videoFrame.close()

    function draw() {
      c?.clearRect(0, 0, width, height)
      c?.fillRect(0, 0, width, height)
      c?.drawImage(imageBitmap, 0, 0)
    }

    window.requestAnimationFrame(draw)
  })
}

async function startRecording() {
  // if (video.value) {
  //   video.value!.src = URL.createObjectURL(mediaSource.value)
  //   video.value!.load()
  recording.value = true
  invoke('start_recording').catch(err => console.error(err))
  // }
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
    <p>
      Stream Screen
    </p>
    <p>
      <em text-sm opacity-75>Screen Streamer to Browser</em>
    </p>

    <div py-4 />
    <div class="flex flex-col items-center justify-center gap-4">
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
      <canvas
        ref="canvas"
        class="h-auto w-[90%]"
      />
    </div>
  </div>
</template>
