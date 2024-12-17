"use client";

import React, { useRef, useState } from 'react';
import { core } from "@tauri-apps/api";

const App = () => {
  const [deviceConnected, setDeviceConnected] = useState(false);
  const [streaming, setStreaming] = useState(false);
  const [lslName, setLslName] = useState('UDL');
  const portRef = useRef<unknown>(null); // Use unknown for now

  const handleConnectDevice = async () => {
    try {
      // Request the backend to auto detect and connect to the device
      const portName = await core.invoke('auto_detect_arduino') as string;  // Type assertion here
      console.log(`Connected to device on port: ${portName}`);
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
      await core.invoke('monitor_device_connection', { portName: portRef.current, stream_name: lslName });
      setStreaming(true);
      console.log('Started LSL streaming');
    } catch (error) {
      console.error('Failed to start streaming:', error);
    }
  };
  



  return (
    <div>
      <div>
        <button onClick={handleConnectDevice}>Connect Device</button>
      </div>
      <div>
      </div>
      <div>
        <button onClick={handleStartStreaming} disabled={!deviceConnected || streaming}>
          Start Streaming
        </button>
      </div>
      {streaming && <p>Streaming data to LSL with name: {lslName}</p>}
    </div>
  );
};

export default App;
