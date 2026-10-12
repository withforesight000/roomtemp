"use client";
import { createContext, useContext, useMemo, type ReactNode } from "react";
import { createFrontendServices, type FrontendServices } from "@/frameworks/frontendServices";
import { useGrpcConnect } from "../hooks/useGrpcConnect";
import { useSettingsWorkflow } from "../hooks/useSettingsWorkflow";

type Connection = ReturnType<typeof useGrpcConnect>;
export type GrpcContextValue = FrontendServices & Pick<Connection,
  "state" | "snapshot" | "connect" | "connectionFailed"> & { saveSettings: ReturnType<typeof useSettingsWorkflow> };
const GrpcRepoContext = createContext<GrpcContextValue | null>(null);
export function useFrontend() {
  const value = useContext(GrpcRepoContext);
  if (!value) throw new Error("useFrontend requires GrpcRepoProvider");
  return value;
}
export function GrpcRepoProvider({ children, services: provided }: { children: ReactNode; services?: FrontendServices }) {
  const services = useMemo(() => provided ?? createFrontendServices(), [provided]);
  const connection = useGrpcConnect(services.operations);
  const saveSettings = useSettingsWorkflow(services.operations, connection);
  return <GrpcRepoContext.Provider value={{ ...services, state: connection.state, snapshot: connection.snapshot,
    connect: connection.connect, connectionFailed: connection.connectionFailed, saveSettings }}>
    {children}
  </GrpcRepoContext.Provider>;
}
