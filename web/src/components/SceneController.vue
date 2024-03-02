<template>
  <v-card class="container" density="compact">
    <v-card-title>Scene State</v-card-title>
    <v-card-text>
      <div class="controllers">
        <div class="controller">
          Clear Color
          <v-color-picker
            hide-inputs
            mode="rgba"
            width="16rem"
            canvas-height="100px"
            :model-value="clearColor"
            @update:model-value="(value) => emit('update:clear-color', value)"
          ></v-color-picker>
        </div>

        <div class="controller">
          Render When Needed
          <v-switch
            hide-details
            density="compact"
            color="primary"
            :model-value="renderWhenNeeded"
            @update:model-value="
              (value) => emit('update:render-when-needed', !!value)
            "
          ></v-switch>
        </div>

        <div class="controller">
          Enable Bounding Volume Culling
          <v-switch
            hide-details
            density="compact"
            color="primary"
            :model-value="culling"
            @update:model-value="(value) => emit('update:culling', !!value)"
          ></v-switch>
        </div>

        <div class="controller">
          Enable Distance Sorting
          <v-switch
            hide-details
            density="compact"
            color="primary"
            :model-value="sorting"
            @update:model-value="(value) => emit('update:sorting', !!value)"
          ></v-switch>
        </div>

        <div class="controller">
          Enable Lighting
          <v-switch
            hide-details
            density="compact"
            color="primary"
            :model-value="lighting"
            @update:model-value="(value) => emit('update:lighting', !!value)"
          ></v-switch>
        </div>

        <div class="controller">
          Enable Gamma Correction
          <v-switch
            hide-details
            density="compact"
            color="primary"
            :model-value="gammaCorrection"
            @update:model-value="
              (value) => emit('update:gamma-correction', !!value)
            "
          ></v-switch>
        </div>

        <div class="controller" v-if="gammaCorrection">
          Gamma
          <v-slider
            hide-details
            :min="0"
            :max="4"
            :step="0.1"
            :model-value="gamma"
            @update:model-value="(value) => emit('update:gamma', value)"
          >
            <template v-slot:append>
              <div>{{ gamma.toFixed(2) }}</div>
            </template>
          </v-slider>
        </div>

        <div class="controller">
          Shading Type
          <v-select
            hide-details
            density="compact"
            :items="shadingTypes"
            :model-value="shadingType.type"
            @update:model-value="
              (type) => {
                if (type === 'ForwardShading') {
                  emit('update:shading-type', {
                    type: 'ForwardShading',
                  });
                } else {
                  emit('update:shading-type', {
                    type: 'DeferredShading',
                  });
                }
              }
            "
          ></v-select>
        </div>

        <div class="controller" v-if="shadingType.type === 'ForwardShading'">
          Enable Multisamples
          <v-switch
            hide-details
            density="compact"
            color="primary"
            :model-value="multisamples"
            @update:model-value="(value) => emit('update:multisamples', !!value)"
          ></v-switch>
        </div>

        <div class="controller" v-if="multisamples && shadingType.type === 'ForwardShading'">
          Multisamples Count
          <v-slider
            show-ticks="always"
            hide-details
            :min="0"
            :max="8"
            :step="1"
            :model-value="multisamples_count"
            @update:model-value="(value) => emit('update:multisamples_count', value)"
          >
            <template v-slot:append>
              <div>{{ multisamples_count }}</div>
            </template>
          </v-slider>
        </div>

        <div class="controller" v-if="shadingType.type === 'ForwardShading'">
          High Dynamic Range
          <v-switch
            hide-details
            density="compact"
            color="primary"
            :model-value="hdr"
            @update:model-value="(value) => emit('update:hdr', !!value)"
          ></v-switch>
        </div>

        <div class="controller" v-if="shadingType.type === 'ForwardShading' && hdr">
          HDR Tone Mapping
          <v-select
            hide-details
            density="compact"
            :items="hdrToneMappings"
            :model-value="hdrToneMapping.type"
            @update:model-value="
              (type) => {
                if (type === 'Exposure') {
                  emit('update:hdr-tone-mapping', {
                    type: 'Exposure',
                    value: hdrExposure,
                  });
                } else {
                  emit('update:hdr-tone-mapping', {
                    type: 'Reinhard',
                  });
                }
              }
            "
          ></v-select>
        </div>

        <div
          class="controller"
          v-if="shadingType.type === 'ForwardShading' && hdr && hdrToneMapping.type === 'Exposure'"
        >
          HDR Exposure
          <v-slider
            hide-details
            :min="0.1"
            :max="10.0"
            :model-value="hdrExposure"
            @update:model-value="(value) => emit('update:hdr-exposure', value)"
          >
            <template v-slot:append>
              <div>{{ hdrExposure.toFixed(2) }}</div>
            </template>
          </v-slider>
        </div>

        <div class="controller" v-if="shadingType.type === 'ForwardShading' && hdr">
          Bloom
          <v-switch
            hide-details
            density="compact"
            color="primary"
            :model-value="bloom"
            @update:model-value="(value) => emit('update:bloom', !!value)"
          ></v-switch>
        </div>

        <div class="controller" v-if="shadingType.type === 'ForwardShading' && hdr && bloom">
          Bloom Blur Epoch
          <v-slider
            hide-details
            :min="0"
            :max="20"
            :step="1"
            :model-value="bloomBlurEpoch"
            @update:model-value="
              (value) => emit('update:bloom-blur-epoch', value)
            "
          >
            <template v-slot:append>
              <div>{{ bloomBlurEpoch }}</div>
            </template>
          </v-slider>
        </div>

        <div class="controller">
          Pick Render Time:
          <span class="time">{{ pickTime.toFixed(2) }}</span>
          ms
        </div>

        <div class="controller">
          Last Frame Render Time:
          <span class="time">{{ renderTime.toFixed(2) }}</span>
          ms
        </div>
      </div>
    </v-card-text>
  </v-card>
</template>

<script setup lang="ts">
import { HdrToneMappingType, ShadingType } from "@/types";
import { PropType } from "vue";
import { VListItem } from "vuetify/lib/components/index.mjs";

defineProps({
  renderTime: {
    type: Number,
    required: true,
  },
  pickTime: {
    type: Number,
    required: true,
  },
  clearColor: {
    type: String,
    required: true,
  },
  culling: {
    type: Boolean,
    required: true,
  },
  sorting: {
    type: Boolean,
    required: true,
  },
  gammaCorrection: {
    type: Boolean,
    required: true,
  },
  gamma: {
    type: Number,
    required: true,
  },
  lighting: {
    type: Boolean,
    required: true,
  },
  shadingType: {
    type: Object as PropType<ShadingType>,
    required: true,
  },
  renderWhenNeeded: {
    type: Boolean,
    required: true,
  },
  multisamples: {
    type: Boolean,
    required: true,
  },
  multisamples_count: {
    type: Number,
    required: true,
  },
  hdr: {
    type: Boolean,
    required: true,
  },
  hdrToneMapping: {
    type: Object as PropType<HdrToneMappingType>,
    required: true,
  },
  hdrExposure: {
    type: Number,
    required: true,
  },
  bloom: {
    type: Boolean,
    required: true,
  },
  bloomBlurEpoch: {
    type: Number,
    required: true,
  },
});

const hdrToneMappings = ["Reinhard", "Exposure"];
const shadingTypes = [
  {
    title: "Forward Shading",
    value: "ForwardShading",
  },
  {
    title: "Deferred Shading",
    value: "DeferredShading",
  },
];

const emit = defineEmits<{
  (event: "update:clear-color", value: string): void;
  (event: "update:render-when-needed", value: boolean): void;
  (event: "update:culling", value: boolean): void;
  (event: "update:sorting", value: boolean): void;
  (event: "update:gamma-correction", value: boolean): void;
  (event: "update:gamma", value: number): void;
  (event: "update:lighting", value: boolean): void;
  (event: "update:shading-type", value: ShadingType): void;
  (event: "update:multisamples", value: boolean): void;
  (event: "update:multisamples_count", value: number): void;
  (event: "update:hdr", value: boolean): void;
  (event: "update:hdr-tone-mapping", value: HdrToneMappingType): void;
  (event: "update:hdr-exposure", value: number): void;
  (event: "update:bloom", value: boolean): void;
  (event: "update:bloom-blur-epoch", value: number): void;
}>();
</script>

<style lang="less" scoped>
.container {
  position: absolute;
  z-index: 2;

  max-height: 100vh;
  overflow-y: auto;

  top: 0;
  left: 0;
}

.time {
  display: inline-block;
  text-align: right;
  width: 3rem;
}
.controllers {
  .controller {
    & + .controller {
      margin-top: 0.5rem;
    }
  }
}
</style>
