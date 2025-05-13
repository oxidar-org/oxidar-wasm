import React from 'react';
import './App.css'

import init, {
  setup_logs as SetupLogs,
  RustCanvas,
} from "./wasm";

function App() {
  const animationRef = React.useRef();
  const [wasm, setWasm] = React.useState();

  React.useEffect(() => {
    init().then(() => {
      SetupLogs();

      const rustCanvas = RustCanvas.create();
      rustCanvas.init("canvas");
      setWasm(rustCanvas);

      const drawLoop = () => {
        rustCanvas.draw();
        animationRef.current = requestAnimationFrame(drawLoop);
      };

      drawLoop();
    });

    // Cleanup on unmount
    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, []);

  return (
    <>
      <canvas id="canvas" width={275} height={200} onClick={() => {
        wasm?.toggleMode();
      }} />
    </>
  )
}

export default App
