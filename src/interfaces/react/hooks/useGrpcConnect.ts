import { useState } from "react";

import { GrpcImpl } from "@/interfaces/repositories/grpc";

export function useGrpcConnect(grpcRepo: GrpcImpl) {
  const [status, setStatus] = useState<string>("");

  const connect = async () => {
    try {
      const status = await grpcRepo.connect();
      setStatus(status);
    } catch (error) {
      const status = `Failed to connect to gRPC server: ${error}`;
      setStatus(status);
    }
  };

  return { status, connect };
}
