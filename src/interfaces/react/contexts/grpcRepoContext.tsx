"use client";

import { createContext } from "react";
import {
  GrpcRepository,
  GrpcRepositoryImpl,
} from "@/interfaces/repositories/grpc";

const repo = new GrpcRepositoryImpl();
export const GrpcRepoContext = createContext<GrpcRepository>(repo);

export function GrpcRepoProvider({ children }: { children: React.ReactNode }) {
  return (
    <GrpcRepoContext.Provider value={repo}>{children}</GrpcRepoContext.Provider>
  );
}
