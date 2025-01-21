import React from "react";
import "./style.css";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

const App: React.FC = () => {
  const [warpStatus, setWarpStatus] = useState<string>("");
  const [connectionAddress, setConnectionAddress] = useState<string>("");

  useEffect(() => {
    const address = listen("connection_address", (event) => {
      setConnectionAddress(event.payload);
    });

    invoke("get_warp_status").then((res) => {
      console.log(res);
      setWarpStatus(res);
    });
  });

  return (
    <div className="container">
      <h1>Pier</h1>
      <p>{warpStatus}</p>
      <p>connection address : {connectionAddress}</p>
      <button
        onClick={() => {
          invoke("get_warp_status").then((res) => {
            console.log(res);
            setWarpStatus(res);
          });
        }}
      >
        status
      </button>

      <button
        onClick={() => {
          invoke("disconnect_from_warp").then((res) => {
            console.log(res);
          });
        }}
      >
        Disconnect
      </button>
      <p>Still in development.</p>
    </div>
  );
};

export default App;
