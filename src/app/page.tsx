"use client";

import { Check, X } from "lucide-react"
import React, { useEffect, useMemo, useReducer, useState } from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';
import { invoke } from '@tauri-apps/api/core';
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
} from "@/components/ui/card"
import { DateTimePicker } from '@/components/ui/expansions/datetime-picker';
import { Label } from "@/components/ui/label"

const UnknownError = "Unknown error";

type State = {
  connectivityStatus: boolean
  graphData?: Object
  info?: string
}

function stateReducer(state: State, action: { type: string; payload: { connectivityStatus?: boolean; graphData?: Object; info?: string } }): State {
  switch (action.type) {
    case "SET_CONNECTIVITY_STATUS":
      if (action.payload.connectivityStatus === undefined) {
        return state;
      }

      return { ...state, connectivityStatus: action.payload.connectivityStatus }
    case "SET_GRAPH_DATA":
      if (action.payload.graphData === undefined) {
        return state;
      }

      return { ...state, graphData: action.payload.graphData }
    case "SET_INFO":
      return { ...state, info: action.payload.info }
    default:
      return state
  }
}

export default function Home() {
  const [state, dispatch] = useReducer(stateReducer, {
    connectivityStatus: false,
    graphData: undefined,
    info: undefined,
  } as State);
  const [startDate24, setStartDate24] = useState<Date | undefined>(() => {
    const date = new Date();
    date.setHours(date.getHours() - 6);
    return date;
  });
  const [endDate24, setEndDate24] = useState<Date | undefined>(() => {
    const date = new Date();
    return date;
  });

  useEffect(() => {
    connectToGrpcServer();
  }, []);

  const connectToGrpcServer = async () => {
    try {
      const response = await invoke('connect_to_grpc_server');
      dispatch({ type: "SET_CONNECTIVITY_STATUS", payload: { connectivityStatus: true } });
      dispatch({ type: "SET_INFO", payload: { info: response !== undefined ? response as string : UnknownError } });
    } catch (error: unknown) {
      console.error("Error connecting to gRPC server:", error);
      dispatch({ type: "SET_INFO", payload: { info: error instanceof Error ? error.message : UnknownError } });
    }
  };

  const fetchGraphData = async (e: React.MouseEvent<HTMLButtonElement>) => {
    e.preventDefault();
    try {
      const response: Object = await invoke('get_graph_data', {startTime: startDate24?.getTime(), endTime: endDate24?.getTime()});
      dispatch({ type: "SET_GRAPH_DATA", payload: { graphData: response } });
    } catch (error: any) {
      if (error instanceof Error) {
        dispatch({ type: "SET_INFO", payload: { info: error.message } });
      } else {
        dispatch({ type: "SET_INFO", payload: { info: error } });
      }
    }
  };

  // graphData ⇒ Recharts 用データに変換
  const formattedData = useMemo(() => {
    // ambient_conditions プロパティがない or 空なら空配列を返す
    if (
      typeof state.graphData !== 'object' ||
      state.graphData === null ||
      !('ambient_conditions' in state.graphData)
    ) {
      return [];
    }
    const ambient = (state.graphData as any).ambient_conditions as Record<
      string,
      { temperature: number; humidity: number; illumination: number }
    >;
    return Object.entries(ambient)
      .map(([key, v]) => {
        const ms = Number(key.split('-')[0]);
        return {
          time: new Date(ms).toString(),
          temperature: v.temperature,
          humidity: v.humidity,
          illumination: v.illumination,
        };
      })
      .sort((a, b) => a.time.localeCompare(b.time));
  }, [state.graphData]);

  return (
    <div>
      <Card>
        <CardContent className="my-4">
          <div className="flex flex-col w-full gap-4">
            <Label htmlFor="name">1. Connect to gRPC Server</Label>
            <div className='flex flex-row justify-between space-x-2'>
              <Button
                className="cursor-pointer"
                type="button"
                onClick={connectToGrpcServer}
              >
                Connect To gRPC Server
              </Button>
              <div className="flex items-center space-x-2">
                <p>status: </p>
                {state.connectivityStatus === true ? (
                  <Check className="h-4 w-4 text-green-500" />
                ) : (
                  <X className="h-4 w-4 text-red-500" />
                )}
              </div>
            </div>
              <div className="flex justify-end">
                <p>status info: {state.info}</p>
              </div>
            <Label htmlFor="name">2. Get the graph data</Label>
            <div className='flex flex-row justify-between space-x-2'>
              <Button className="cursor-pointer" onClick={fetchGraphData}>Get Graph Data</Button>
            </div>
          </div>
        </CardContent>
      </Card>

      <div className="flex w-full gap-4 my-4">
        <div className="flex-1 flex-col gap-2">
          <Label>Start Time</Label>
          <DateTimePicker hourCycle={24} value={startDate24} onChange={setStartDate24} />
        </div>

        <div className="flex-1 flex-col gap-2">
          <Label>End Time</Label>
          <DateTimePicker hourCycle={24} value={endDate24} onChange={setEndDate24} />
        </div>
      </div>

      {/* スマートフォンサイズの場合は全幅、縦幅は残り画面いっぱい使う*/}
      <div className="w-full h-[calc(100vh-30rem)]">
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={formattedData} margin={{ top: 20, right: 20, bottom: 20, left: 0 }}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="time" tick={{ fontSize: 12 }} />
            <YAxis />
            <Tooltip />
            <Legend verticalAlign="top" height={36} />
            <Line type="monotone" dataKey="temperature" name="Temperature (℃)" />
            <Line type="monotone" dataKey="humidity" name="Humidity (%)" />
            <Line type="monotone" dataKey="illumination" name="Illumination (lx)" />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}
