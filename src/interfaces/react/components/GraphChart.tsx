import React from "react";
import {
  ResponsiveContainer,
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
} from "recharts";
import { RechartPoint } from "@/interfaces/presenters/graphPresenter";

type GraphChartProps = {
  data: RechartPoint[];
  dataKey: string;
  description: string;
};

export const GraphChart: React.FC<GraphChartProps> = ({ data, dataKey, description }) => {
  return (
    <ResponsiveContainer width="100%" height={400}>
      <LineChart
        data={data}
        margin={{ top: 20, right: 20, bottom: 20, left: 0 }}
      >
        <CartesianGrid strokeDasharray="3 3" />
        <XAxis dataKey="time" tick={{ fontSize: 12 }} />
        <YAxis />
        <Tooltip />
        <Legend verticalAlign="top" height={36} />
        {/* <Line type="monotone" dataKey="temperature" name="Temperature (℃)" />
        <Line type="monotone" dataKey="humidity" name="Humidity (%)" />
        <Line type="monotone" dataKey="illumination" name="Illumination (lx)" /> */}
        <Line type="monotone" dataKey={dataKey} name={description} dot={false} />
      </LineChart>
    </ResponsiveContainer>
  );
};

// export const GraphChartForTemperature: React.FC<GraphChartProps> = ({ data }) => {
//   return (
//     <ResponsiveContainer width="100%" height={400}>
//       <LineChart
//         data={data}
//         margin={{ top: 20, right: 20, bottom: 20, left: 0 }}
//       >
//         <CartesianGrid strokeDasharray="3 3" />
//         <XAxis dataKey="time" tick={{ fontSize: 12 }} />
//         <YAxis />
//         <Tooltip />
//         <Legend verticalAlign="top" height={36} />
//         <Line type="monotone" dataKey="temperature" name="Temperature (℃)" />
//       </LineChart>
//     </ResponsiveContainer>
//   );
// };
