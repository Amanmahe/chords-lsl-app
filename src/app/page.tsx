import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri"; // Import the invoke function

const Home = () => {
  const [isStreaming, setIsStreaming] = useState(false);
  const [streamData, setStreamData] = useState<string[]>([]); // Store streamed data

  // Start the LSL stream
  const startStreaming = async () => {
    setIsStreaming(true);
    await invoke("start_lsl_stream");
  };

  // Stop the LSL stream
  const stopStreaming = async () => {
    setIsStreaming(false);
    await invoke("stop_lsl_stream");

  // Use `listen` to receive real-time updates
  useEffect(() => {
    const unlisten = listen<string>("stream-data", (event) => {
      setStreamData((prevData) => [...prevData, event.payload]); // Append new data
    });

    // Clean up listener on unmount
    return () => {
      unlisten.then((dispose) => dispose());
    };
  }, []);

  return (
    <div>
      <h1>LSL Connector</h1>
      <button onClick={startStreaming} disabled={isStreaming}>
        Start Streaming
      </button>
      <button onClick={stopStreaming} disabled={!isStreaming}>
        Stop Streaming
      </button>
      <div>
        <h2>Stream Data:</h2>
        <ul>
          {streamData.map((data, index) => (
            <li key={index}>{data}</li>
          ))}
        </ul>
      </div>
    </div>
  );
};

export default Home;
