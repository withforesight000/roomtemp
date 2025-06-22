"use client";

import { createContext } from "react";
import { Grpc, GrpcImpl } from "@/interfaces/repositories/grpc";

const repo = new GrpcImpl();
export const GrpcRepoContext = createContext<Grpc>(repo);

export function GrpcRepoProvider({ children }: { children: React.ReactNode }) {
  return <GrpcRepoContext.Provider value={repo}>{children}</GrpcRepoContext.Provider>;
}
