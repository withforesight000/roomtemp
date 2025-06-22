import { AmbientCondition } from '@/domain/types';
import { RustType, Decoder } from 'bincode-ts';

export const decoder = new Decoder();

export const GraphDataRustType = RustType.HashMap(
  RustType.Str,
  RustType.Struct<AmbientCondition>([
    ["temperature", RustType.f32],
    ["humidity", RustType.f32],
    ["illumination", RustType.f32],
  ])
);
