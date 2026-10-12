"use client";
import { createContext, useContext, useEffect, useMemo, type ReactNode } from "react";
import { createFrontendServices, type FrontendServices } from "@/frameworks/frontendServices";
import { useGrpcConnect } from "../hooks/useGrpcConnect";

type Connection = ReturnType<typeof useGrpcConnect>;
export type GrpcContextValue = FrontendServices & Pick<Connection,
  "state" | "connect" | "beginSave" | "finishSave" | "connectionFailed">;
const GrpcRepoContext = createContext<GrpcContextValue | null>(null);
export function useFrontend() {
  const value = useContext(GrpcRepoContext);
  if (!value) throw new Error("useFrontend requires GrpcRepoProvider");
  return value;
}
export function GrpcRepoProvider({ children, services: provided }: { children: ReactNode; services?: FrontendServices }) {
  const services = useMemo(() => provided ?? createFrontendServices(), [provided]);
  const { state, connect, beginSave, finishSave, connectionFailed, activate, deactivate } =
    useGrpcConnect(services.operations, services.connectedMessage);
  useEffect(() => { activate(); void connect(); return deactivate; }, [connect, activate, deactivate]);
  return <GrpcRepoContext.Provider value={{ ...services, state, connect, beginSave, finishSave, connectionFailed }}>
    {children}
  </GrpcRepoContext.Provider>;
}
