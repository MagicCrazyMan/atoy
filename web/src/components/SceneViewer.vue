<template>
  <div class="scene" id="scene"></div>
  <SceneController
    :render-time="renderTime"
    :pick-time="pickTime"
    v-model:clear-color="clearColor"
    v-model:render-when-needed="renderWhenNeeded"
    v-model:random-color="randomColor"
    v-model:culling="culling"
    v-model:sorting="sorting"
    v-model:gamma-correction="gammaCorrection"
    v-model:gamma="gamma"
    v-model:lighting="lighting"
    v-model:shading-type="shadingType"
    v-model:multisamples="multisamples"
    v-model:multisamples_count="multisamples_count"
    v-model:hdr="hdr"
    v-model:hdr-tone-mapping="hdrToneMapping"
    v-model:hdr-exposure="hdrExposure"
    v-model:bloom="bloom"
    v-model:bloom-blur-epoch="bloomBlurEpoch"
  />
</template>

<script setup lang="ts">
import init, {
  LogLevel,
  init_with_log_level,
  test_cube,
  test_pick,
} from "atoy";
import { onMounted } from "vue";
import SceneController from "./SceneController.vue";
import { ref } from "vue";
import { watch } from "vue";
import { HdrToneMappingType, ShadingType } from "@/types";

const renderWhenNeeded = ref(true);
const randomColor = ref(true);
const clearColor = ref("#0000");
const culling = ref(false);
const sorting = ref(false);
const gammaCorrection = ref(true);
const gamma = ref(2.2);
const lighting = ref(false);
const shadingType = ref<ShadingType>({
  type: "ForwardShading",
});
const multisamples = ref(false);
const multisamples_count = ref(0);
const hdr = ref(false);
const hdrToneMapping = ref<HdrToneMappingType>({
  type: "Reinhard",
});
const hdrExposure = ref(1.0);
const bloom = ref(false);
const bloomBlurEpoch = ref(10);
const renderTime = ref(0);
const pickTime = ref(0);

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
    const color = viewer.clear_color();
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
  renderWhenNeeded.value = viewer.render_when_needed();
  randomColor.value = viewer.random_color_on_tick_enabled();
  culling.value = viewer.culling_enabled();
  sorting.value = viewer.distance_sorting_enabled();
  gammaCorrection.value = viewer.gamma_correction_enabled();
  gamma.value = viewer.gamma();
  lighting.value = viewer.lighting_enabled();
  shadingType.value = viewer.pipeline_shading();
  multisamples.value = viewer.multisamples_enabled();
  multisamples_count.value = viewer.multisamples_count();
  hdr.value = viewer.hdr_enabled();
  bloom.value = viewer.bloom_enabled();
  bloomBlurEpoch.value = viewer.bloom_blur_epoch();

  hdrToneMapping.value = viewer.hdr_tone_mapping_type() as HdrToneMappingType;
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

    viewer.set_clear_color(r, g, b, a);
  });
  watch(renderWhenNeeded, (renderWhenNeeded) => {
    if (renderWhenNeeded) {
      viewer.enable_render_when_needed();
    } else {
      viewer.disable_render_when_needed();
    }
  });
  watch(randomColor, (randomColor) => {
    if (randomColor) {
      viewer.enable_random_color_on_tick();
    } else {
      viewer.disable_random_color_on_tick();
    }
  });
  watch(culling, (culling) => {
    if (culling) {
      viewer.enable_culling();
    } else {
      viewer.disable_culling();
    }
  });
  watch(sorting, (sorting) => {
    if (sorting) {
      viewer.enable_distance_sorting();
    } else {
      viewer.disable_distance_sorting();
    }
  });
  watch(gammaCorrection, (gammaCorrection) => {
    if (gammaCorrection) {
      viewer.enable_gamma_correction();
    } else {
      viewer.disable_gamma_correction();
    }
  });
  watch(gamma, (gamma) => {
    viewer.set_gamma(gamma);
  });
  watch(lighting, (lighting) => {
    if (lighting) {
      viewer.enable_lighting();
    } else {
      viewer.disable_lighting();
    }
  });
  watch(shadingType, (shadingType) => {
    if (
      shadingType.type === "ForwardShading" ||
      shadingType.type === "DeferredShading"
    ) {
      viewer.set_pipeline_shading(shadingType);
    }
  });
  watch(multisamples, (multisamples) => {
    if (multisamples) {
      viewer.enable_multisamples();
    } else {
      viewer.disable_multisamples();
    }
  });
  watch(multisamples_count, (count) => {
    viewer.set_multisamples_count(count);
  });
  watch(hdr, (hdr) => {
    if (hdr) {
      viewer.enable_hdr();
    } else {
      viewer.disable_hdr();
    }
  });
  watch(hdrToneMapping, (type) => {
    viewer.set_hdr_tone_mapping_type(type);
  });
  watch(hdrExposure, (value) => {
    viewer.set_hdr_tone_mapping_type({
      type: "Exposure",
      value,
    });
  });
  watch(bloom, (bloom) => {
    if (bloom) {
      viewer.enable_bloom();
    } else {
      viewer.disable_bloom();
    }
  });
  watch(bloomBlurEpoch, (bloom_blur_epoch) => {
    viewer.set_bloom_blur_epoch(bloom_blur_epoch);
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
