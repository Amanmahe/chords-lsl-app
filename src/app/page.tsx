"use client";

import React, { useRef, useState } from 'react';
import { core } from "@tauri-apps/api";
import { Button } from "./ui/Button";

const App = () => {
  const [deviceConnected, setDeviceConnected] = useState(false);
  const [streaming, setStreaming] = useState(false);
  const [lslName, setLslName] = useState('UDL');
  const portRef = useRef<unknown>(null);

  const handleConnectDevice = async () => {
    try {
      const portName = await core.invoke('auto_detect_arduino') as string;
      console.log(`Connected to device on port: ${portName}`);
      setLslName("UDL");
      portRef.current = portName;
      setDeviceConnected(true);
    } catch (error) {
      console.error('Failed to connect to device:', error);
    }
  };

  const handleStartStreaming = async () => {
    if (!deviceConnected) {
      console.error('Device is not connected.');
      return;
    }
    try {
      setStreaming(true);
      await core.invoke('monitor_device_connection', { portName: portRef.current, stream_name: lslName });
      
      console.log('Started LSL streaming');
    } catch (error) {
      console.error('Failed to start streaming:', error);
    }
  };

  return (
    <div className="flex flex-col items-center justify-center h-screen bg-gray-50">
      {/* Buttons Container */}
      <div className="flex gap-4 mb-4">
        <Button
          className="flex items-center justify-center gap-1 py-2 px-6 sm:py-3 sm:px-8 rounded-xl font-semibold bg-blue-600 text-white hover:bg-blue-700"
          onClick={handleConnectDevice}
          disabled={deviceConnected}
        >
          Connect Device
        </Button>
        <Button
          className="flex items-center justify-center gap-1 py-2 px-6 sm:py-3 sm:px-8 rounded-xl font-semibold bg-green-600 text-white hover:bg-green-700 disabled:bg-gray-400"
          onClick={handleStartStreaming}
          disabled={!deviceConnected || streaming}
        >
          Start Streaming
        </Button>
      </div>

      {/* Status Message */}
      {streaming && (
        <p className="text-lg font-medium text-gray-700">
          Streaming data to LSL with name: <span className="text-blue-600">{lslName}</span>
        </p>
      )}
    </div>
  );
};

export default App;
