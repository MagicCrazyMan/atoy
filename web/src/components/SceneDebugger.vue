<template>
  <div class="container">
    <canvas class="scene" ref="debuggerSceneCanvas"></canvas>
    <div class="msg">This is a test page, A WebGL2RenderingContext had been mounted to global variable</div>
  </div>
</template>

<script setup lang="ts">
import { onBeforeUnmount } from "vue";
import { onMounted, ref } from "vue";

const debuggerSceneCanvas = ref<HTMLCanvasElement | null>(null);

onMounted(() => {
  if (debuggerSceneCanvas.value) {
    const gl = debuggerSceneCanvas.value.getContext('webgl2')
    if (!gl) {
      alert('WebGL2 is not available')
    } else {
      window.gl = gl
    }
  }
});
onBeforeUnmount(() => {
  delete window.gl
});

declare global {
  interface Window {
    gl: WebGL2RenderingContext | undefined;
  }
}
</script>

<style lang="less" scoped>
.container {
  width: 100vw;
  height: 100vh;

  position: relative;

  top: 0;
  left: 0;
}

.scene {
  width: 100%;
  height: 100%;
}

.msg {
  position: absolute;

  top: 1rem;
  left: 1rem;

  z-index: 1;
}
</style>
