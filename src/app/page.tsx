"use client";

import { Check, X } from "lucide-react"
import React, { useMemo } from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';
import { useMessage } from "@/hooks/useMessage";
import { chartData } from "@/data/chartData";
import { invoke } from '@tauri-apps/api/core';
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Label } from "@/components/ui/label"


export default function Home() {
  const { message, fetchMessage } = useMessage(32);
  const [connectivityStatus, setConnectivityStatus] = React.useState(false);
  const [graphData, setGraphData] = React.useState<Object>(chartData);

  const connectToGrpcServer = async (e: React.MouseEvent<HTMLButtonElement>) => {
    e.preventDefault();
    try {
      const response = await invoke('connect_to_grpc_server');
      console.log(response);
      setConnectivityStatus(true);
    } catch (error: unknown) {
      console.error("Error connecting to gRPC server:", error);
    }
  };

  const fetchGraphData = async (e: React.MouseEvent<HTMLButtonElement>) => {
    e.preventDefault();
    try {
      const response: Object = await invoke('get_graph_data');
      console.log(response);
      setGraphData(response);
    } catch (error: unknown) {
      console.error("Error fetching graph data:", error);
    }
  };

  // graphData ⇒ Recharts 用データに変換
  const formattedData = useMemo(() => {
    // ambient_conditions プロパティがない or 空なら空配列を返す
    if (
      typeof graphData !== 'object' ||
      graphData === null ||
      !('ambient_conditions' in graphData)
    ) {
      return [];
    }
    const ambient = (graphData as any).ambient_conditions as Record<
      string,
      { temperature: number; humidity: number; illumination: number }
    >;
    return Object.entries(ambient)
      .map(([key, v]) => {
        const ms = Number(key.split('-')[0]);
        return {
          time: new Date(ms).toLocaleTimeString(), // hh:mm:ss
          temperature: v.temperature,
          humidity: v.humidity,
          illumination: v.illumination,
        };
      })
      .sort((a, b) => a.time.localeCompare(b.time));
  }, [graphData]);

  return (
    <div>
      <Card>
        <CardContent className="my-4">
          <div className="flex flex-col w-full gap-4">
            <Label htmlFor="name">1. Connect to gRPC Server</Label>
            <div className='flex flex-row justify-between space-x-2'>
              <Button className="cursor-pointer" onClick={connectToGrpcServer}>Connect To gRPC Server</Button>
              <div className="flex items-center space-x-2">
                <p>status: </p>
                {connectivityStatus === true ? <Check className="h-4 w-4 text-green-500" /> : <X className="h-4 w-4 text-red-500" />}
              </div>
            </div>
            <Label htmlFor="name">2. Get the graph data</Label>
            <div className='flex flex-row justify-between space-x-2'>
              <Button className="cursor-pointer" onClick={fetchGraphData}>Get Graph Data</Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* スマートフォンサイズの場合は全幅、大きな画面の場合は横幅の70% */}
      <div className="w-full h-96">
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
