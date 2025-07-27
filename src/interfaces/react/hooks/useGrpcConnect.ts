import { useCallback, useReducer } from "react";

import { GrpcRepositoryImpl } from "@/interfaces/repositories/grpc";

type State = {
  status: string;
  isLoading: boolean;
  hasError: boolean;
}

function reducer(state: State, action: { type: string; payload?: string }): State {
  switch (action.type) {
    case "SUCCESS":
      return { ...state, status: action.payload || "", hasError: false };
    case "ERROR":
      return { ...state, status: action.payload || "", hasError: true };
    case "LOADING":
      return { ...state, isLoading: true };
    case "LOADED":
      return { ...state, isLoading: false };
    default:
      return state;
  }
}

export function useGrpcConnect(grpcRepo: GrpcRepositoryImpl){
  const [state, dispatch] = useReducer(reducer, { status: "", isLoading: true, hasError: false });

  const connect = useCallback(async () => {

    try {
      dispatch({ type: "LOADING" });
      const status = await grpcRepo.connect();
      dispatch({ type: "SUCCESS", payload: status });
    } catch (error) {
      const status = `${error}`;
      dispatch({ type: "ERROR", payload: status });
    } finally {
      dispatch({ type: "LOADED" });
    }
  }, [grpcRepo]);

  return { state, connect };
}
