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
