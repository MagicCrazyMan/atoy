<template>
  <div class="scene" id="scene"></div>
  <SceneController
    :render-time="renderTime"
    :pick-time="pickTime"
    v-model:clear-color="clearColor"
    v-model:samples="samples"
    v-model:hdr="hdr"
    v-model:hdr-tone-mapping="hdrToneMapping"
    v-model:hdr-exposure="hdrExposure"
  />
</template>

<script setup lang="ts">
import init, { LogLevel, init_with_log_level, test_cube } from "atoy";
import { onMounted } from "vue";
import SceneController from "./SceneController.vue";
import { ref } from "vue";
import { watch } from "vue";
import { HdrToneMappingType } from "@/types";

const clearColor = ref("#0000");
const renderTime = ref(0);
const pickTime = ref(0);
const samples = ref(0);
const hdr = ref(false);
const hdrToneMapping = ref<HdrToneMappingType>({
  type: "Reinhard",
});
const hdrExposure = ref(1.0);

onMounted(async () => {
  await init();
  init_with_log_level(LogLevel.Info);

  const viewer = test_cube(
    40000,
    200,
    500,
    500,
    (time: number) => {
      renderTime.value = time;
    },
    (time: number) => {
      pickTime.value = time;
    }
  );

  clearColor.value = (() => {
    const color = viewer.clear_color_wasm();
    const r = Math.floor(color[0] * 255)
      .toString(16)
      .padStart(2, "0");
    const g = Math.floor(color[1] * 255)
      .toString(16)
      .padStart(2, "0");
    const b = Math.floor(color[2] * 255)
      .toString(16)
      .padStart(2, "0");
    const a = Math.floor(color[3] * 255)
      .toString(16)
      .padStart(2, "0");
    return `#${r}${g}${b}${a}`;
  })();
  samples.value = viewer.multisample_wasm() ?? 0;
  hdr.value = viewer.hdr_enabled_wasm();

  hdrToneMapping.value =
    viewer.hdr_tone_mapping_type_wasm() as HdrToneMappingType;
  if (hdrToneMapping.value.type === "Exposure") {
    hdrExposure.value = hdrToneMapping.value.value;
  }

  watch(clearColor, (color) => {
    const rgba = new RegExp(/#(\w{2})(\w{2})(\w{2})(\w{0,2})/).exec(color);
    const rs = rgba?.at(1);
    const gs = rgba?.at(2);
    const bs = rgba?.at(3);
    const as = rgba?.at(4);

    const r = rs ? parseInt(rs, 16) / 255 : 0;
    const g = gs ? parseInt(gs, 16) / 255 : 0;
    const b = bs ? parseInt(bs, 16) / 255 : 0;
    const a = as ? parseInt(as, 16) / 255 : 1;

    viewer.set_clear_color_wasm(r, g, b, a);
  });
  watch(samples, (samples) => {
    viewer.set_multisample_wasm(samples);
  });
  watch(hdr, (hdr) => {
    if (hdr) {
      viewer.enable_hdr_wasm();
    } else {
      viewer.disable_hdr_wasm();
    }
  });
  watch(hdrToneMapping, (type) => {
    viewer.set_hdr_tone_mapping_type_wasm(type);
  });
  watch(hdrExposure, (value) => {
    viewer.set_hdr_tone_mapping_type_wasm({
      type: "Exposure",
      value,
    });
  });
});
</script>

<style lang="less" scoped>
.scene {
  width: 100vw;
  height: 100vh;

  position: absolute;
  top: 0;
  left: 0;
  z-index: 1;
}
</style>
