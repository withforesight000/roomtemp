'use client';

import Image from "next/image";
import { invoke } from '@tauri-apps/api/core';
import { useState, useEffect } from 'react';
import React, { PureComponent } from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';

const data = [
  {
    name: 'Page A',
    uv: 4000,
    pv: 2400,
    amt: 2400,
  },
  {
    name: 'Page B',
    uv: 3000,
    pv: 1398,
    amt: 2210,
  },
  {
    name: 'Page C',
    uv: 2000,
    pv: 9800,
    amt: 2290,
  },
  {
    name: 'Page D',
    uv: 2780,
    pv: 3908,
    amt: 2000,
  },
  {
    name: 'Page E',
    uv: 1890,
    pv: 4800,
    amt: 2181,
  },
  {
    name: 'Page F',
    uv: 2390,
    pv: 3800,
    amt: 2500,
  },
  {
    name: 'Page G',
    uv: 3490,
    pv: 4300,
    amt: 2100,
  },
];

export default function Home() {
  const [message, setMessage] = useState<string>('');

  async function fetchMessage() {
    try {
      const response = await invoke('my_custom_command');
      setMessage(response as string);
    } catch (error) {
      console.error('Error invoking Tauri command:', error);
    }
  }
  useEffect(() => {
    fetchMessage();
  }, []);

  return (
    <div>
      <div>
        <p>Hello from {message}</p>
        <button
          onClick={fetchMessage}
          onLoad={fetchMessage}
          className="mt-4 px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
        >
          Call Rust Command
        </button>
      </div>
      <div style={{ width: '100%', height: 400 }}>
        <ResponsiveContainer>
          <LineChart
            data={data}
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
