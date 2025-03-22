"use client";

import React from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';
import { useMessage } from "@/hooks/useMessage";
import { chartData } from "@/data/chartData";

export default function Home() {
  const { message, fetchMessage } = useMessage(32);

  return (
    <div>
      <div>
        <p className='break-words'>Hello from {message}</p>
        <button
          onClick={fetchMessage}
          onLoad={fetchMessage}
          className="my-2 px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
        >
          Call Rust Command
        </button>
      </div>
      <div style={{ width: '100%', height: 400 }}>
        <ResponsiveContainer>
          <LineChart
            data={chartData}
            margin={{
              top: 5,
              right: 30,
              left: 20,
              bottom: 5,
            }}
          >
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="name" />
            <YAxis />
            <Tooltip />
            <Legend />
            <Line type="monotone" dataKey="pv" stroke="#8884d8" activeDot={{ r: 8 }} />
            <Line type="monotone" dataKey="uv" stroke="#82ca9d" />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}
