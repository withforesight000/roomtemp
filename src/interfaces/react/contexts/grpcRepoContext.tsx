"use client";

import React, { createContext, useMemo, useEffect } from "react";
import {
  GrpcRepository,
  GrpcRepositoryImpl,
} from "@/interfaces/repositories/grpc";
import { useGrpcConnect } from "../hooks/useGrpcConnect";

export interface GrpcContextValue {
  grpcRepo: GrpcRepository;
  connect: () => Promise<void>;
  state: {
    status: string;
    isLoading: boolean;
    hasError: boolean;
  };
}

const dummy: GrpcContextValue = {
  grpcRepo: new GrpcRepositoryImpl(),
  connect: async () => {},
  state: {
    status: "",
    isLoading: false,
    hasError: false,
  },
};

export const GrpcRepoContext = createContext<GrpcContextValue>(dummy);

export function GrpcRepoProvider({ children }: { children: React.ReactNode }) {
  const grpcRepo = useMemo(() => new GrpcRepositoryImpl(), []);
  const { connect, state } = useGrpcConnect(grpcRepo);

  useEffect(() => {
    connect();
  }, [connect]);

  return (
    <GrpcRepoContext.Provider value={{ grpcRepo, connect, state }}>
      {children}
    </GrpcRepoContext.Provider>
  );
}
