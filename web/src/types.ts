export type HdrToneMappingType =
  | HdrReinhardToneMapping
  | HdrExposureToneMapping;

export type HdrReinhardToneMapping = {
  type: "Reinhard";
};

export type HdrExposureToneMapping = {
  type: "Exposure";
  value: number;
};

export type ShadingType =
  | ForwardShading
  | DeferredShading
  | PickingShading;

export type ForwardShading = {
  type: "ForwardShading";
};

export type DeferredShading = {
  type: "DeferredShading";
};

export type PickingShading = {
  type: "PickingShading";
};
