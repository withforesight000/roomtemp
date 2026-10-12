"use client";

import React, { createContext, useMemo, useEffect } from "react";
import { GrpcRepository, GrpcRepositoryImpl } from "@/interfaces/repositories/grpc";
import { GrpcError } from "@/domain/grpcError";
import { ConnectionState, initialConnectionState, useGrpcConnect } from "../hooks/useGrpcConnect";

export interface GrpcContextValue {
  grpcRepo: GrpcRepository;
  connect: () => Promise<boolean>;
  state: ConnectionState;
  beginSave: () => void;
  finishSave: (_saved: boolean) => void;
  connectionFailed: (_error: GrpcError, _generation: number) => void;
}

const dummy: GrpcContextValue = {
  grpcRepo: new GrpcRepositoryImpl(), connect: async () => false,
  state: initialConnectionState, beginSave: () => {}, finishSave: () => {},
  connectionFailed: () => {},
};

export const GrpcRepoContext = createContext<GrpcContextValue>(dummy);

export function GrpcRepoProvider({ children }: { children: React.ReactNode }) {
  const grpcRepo = useMemo(() => new GrpcRepositoryImpl(), []);
  const { state, connect, beginSave, finishSave, connectionFailed, activate, deactivate } = useGrpcConnect(grpcRepo);
  useEffect(() => {
    activate();
    void connect();
    return deactivate;
  }, [connect, activate, deactivate]);

  return (
    <GrpcRepoContext.Provider value={{ grpcRepo, connect, state, beginSave, finishSave, connectionFailed }}>
      {children}
    </GrpcRepoContext.Provider>
  );
}
