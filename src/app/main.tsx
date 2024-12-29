"use client";

import React, { useRef, useState } from 'react';
import { core } from "@tauri-apps/api";
import { Button } from "./ui/Button";
import { listen, Event } from '@tauri-apps/api/event';

const App = () => {
  const [deviceConnected, setDeviceConnected] = useState(false);
  const [streaming, setStreaming] = useState(false);
  const [lslName, setLslName] = useState('UDL');
  const [refreshRate, setRefreshRate] = useState<number | null>(null);
  const [samplesPerSecond, setSamplesPerSecond] = useState<number | null>(null);
  const [bytesPerSecond, setBytesPerSecond] = useState<number | null>(null);
  const portRef = useRef<unknown>(null);

  const handleConnectDevice = async () => {
    try {
      const portName = await core.invoke('auto_detect_arduino') as string;
      console.log(`Connected to device on port: ${portName}`);
      setLslName("UDL");
      portRef.current = portName;
      setDeviceConnected(true);
      setStreaming(true);
      await core.invoke('monitor_device_connection', { portName: portRef.current, stream_name: lslName });
      setStreaming(true);
    } catch (error) {
      console.error('Failed to connect to device:', error);
    }
  };

  // Listen for the 'updatePerformance' event and update the state
  listen<{ refreshRate: number; samplesPerSecond: number; bytesPerSecond: number }>(
    'updatePerformance',
    (event: Event<{ refreshRate: number; samplesPerSecond: number; bytesPerSecond: number }>) => {
      const { refreshRate, samplesPerSecond, bytesPerSecond } = event.payload;
      console.log(`Refresh Rate: ${refreshRate} Hz`);
      console.log(`Samples Per Second: ${samplesPerSecond}`);
      console.log(`Bytes Per Second: ${bytesPerSecond}`);

      setRefreshRate(refreshRate);
      setSamplesPerSecond(samplesPerSecond);
      setBytesPerSecond(bytesPerSecond);
    }
  );

  return (
    <div className="flex flex-col items-center justify-center h-screen bg-gray-100">
      {/* Buttons Container */}
      {!streaming && (
        <div className="flex gap-4 mb-4">
          <Button
            className="flex items-center justify-center gap-1 py-3 px-8 rounded-full font-semibold bg-blue-600 text-white shadow-lg hover:bg-blue-700 transition"
            onClick={handleConnectDevice}
            disabled={deviceConnected}
          >
            Connect Device
          </Button>
        </div>
      )}

      {/* Performance Data Display */}
      {streaming && (

<div className="w-full max-w-md bg-white shadow-xl rounded-2xl p-6">
          <table className="table-auto w-full h-200px border-collapse border border-gray-300">
            <thead>
              <tr className="bg-gray-100">
                <th className="px-4 py-2 border border-gray-300 text-sm text-gray-700 font-medium">Metric</th>
                <th className="px-4 py-2 border border-gray-300 text-sm text-gray-700 font-medium">Value</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td className="px-4 py-2 border border-gray-300 text-sm text-gray-600">FPS</td>
                <td className="px-4 py-2 border border-gray-300 text-sm text-gray-600">{refreshRate} Hz</td>
              </tr>
              <tr>
                <td className="px-4 py-2 border border-gray-300 text-sm text-gray-600">Samples Per Second</td>
                <td className="px-4 py-2 border border-gray-300 text-sm text-gray-600">{samplesPerSecond}</td>
              </tr>
              <tr>
                <td className="px-4 py-2 border border-gray-300 text-sm text-gray-600">Bytes Per Second</td>
                <td className="px-4 py-2 border border-gray-300 text-sm text-gray-600">{bytesPerSecond}</td>
              </tr>
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
};

export default App;
