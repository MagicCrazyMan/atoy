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
          Multisample
          <v-slider
            show-ticks="always"
            hide-details
            :min="0"
            :max="8"
            :step="1"
            :model-value="samples"
            @update:model-value="(value) => emit('update:samples', value)"
          >
            <template v-slot:append>
              <div>{{ samples }}</div>
            </template>
          </v-slider>
        </div>

        <div class="controller">
          High Dynamic Range
          <v-switch
            hide-details
            density="compact"
            color="primary"
            :model-value="hdr"
            @update:model-value="(value) => emit('update:hdr', !!value)"
          ></v-switch>
        </div>

        <div class="controller" v-if="hdr">
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
          v-if="hdr && hdrToneMapping.type === 'Exposure'"
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

        <div class="controller" v-if="hdr">
          Bloom
          <v-switch
            hide-details
            density="compact"
            color="primary"
            :model-value="bloom"
            @update:model-value="(value) => emit('update:bloom', !!value)"
          ></v-switch>
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
import { HdrToneMappingType } from "@/types";
import { PropType } from "vue";

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
  lighting: {
    type: Boolean,
    required: true,
  },
  renderWhenNeeded: {
    type: Boolean,
    required: true,
  },
  samples: {
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
});

const hdrToneMappings = ["Reinhard", "Exposure"];

const emit = defineEmits<{
  (event: "update:clear-color", value: string): void;
  (event: "update:render-when-needed", value: boolean): void;
  (event: "update:culling", value: boolean): void;
  (event: "update:sorting", value: boolean): void;
  (event: "update:lighting", value: boolean): void;
  (event: "update:samples", value: number): void;
  (event: "update:hdr", value: boolean): void;
  (event: "update:hdr-tone-mapping", value: HdrToneMappingType): void;
  (event: "update:hdr-exposure", value: number): void;
  (event: "update:bloom", value: boolean): void;
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
