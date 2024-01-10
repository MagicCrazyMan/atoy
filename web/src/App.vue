<template>
  <SceneViewer />
</template>

<script setup lang="ts">
import SceneViewer from "./components/SceneViewer.vue";

const worker = new Worker("./worker.js")
</script>

<style lang="less">
html,
body {
  padding: 0;
  margin: 0;

  overflow: hidden;
}
</style>
