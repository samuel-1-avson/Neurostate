import { createSignal, For, onMount, createEffect } from "solid-js";
import "./FilterResponsePlot.css";

interface FilterResponseProps {
  filterType?: "lowpass" | "highpass" | "bandpass" | "bandstop";
  cutoffFrequency?: number;
  sampleRate?: number;
  order?: number;
}

export function FilterResponsePlot(props: FilterResponseProps) {
  const [magnitudeData, setMagnitudeData] = createSignal<{ freq: number; mag: number }[]>([]);
  const [phaseData, setPhaseData] = createSignal<{ freq: number; phase: number }[]>([]);
  const [viewMode, setViewMode] = createSignal<"magnitude" | "phase" | "both">("magnitude");
  
  const filterType = () => props.filterType || "lowpass";
  const cutoff = () => props.cutoffFrequency || 1000;
  const sampleRate = () => props.sampleRate || 48000;
  const order = () => props.order || 2;

  // Generate frequency response data
  const generateResponse = () => {
    const data: { freq: number; mag: number }[] = [];
    const phaseArr: { freq: number; phase: number }[] = [];
    const nyquist = sampleRate() / 2;
    const fc = cutoff();
    const n = order();
    
    // Log-scale frequencies from 20Hz to Nyquist
    for (let i = 0; i <= 100; i++) {
      const logFreq = Math.pow(10, 1.3 + (i / 100) * (Math.log10(nyquist) - 1.3));
      const freq = Math.min(logFreq, nyquist - 1);
      
      // Simple Butterworth approximation
      const wc = fc / nyquist;
      const w = freq / nyquist;
      let H: number;
      
      switch (filterType()) {
        case "lowpass":
          H = 1 / Math.sqrt(1 + Math.pow(w / wc, 2 * n));
          break;
        case "highpass":
          H = 1 / Math.sqrt(1 + Math.pow(wc / w, 2 * n));
          break;
        case "bandpass":
          const bw = 0.3; // Bandwidth factor
          H = 1 / Math.sqrt(1 + Math.pow((w * w - wc * wc) / (w * bw * wc), 2 * n));
          break;
        default:
          H = 1 / Math.sqrt(1 + Math.pow(w / wc, 2 * n));
      }
      
      const magDb = 20 * Math.log10(Math.max(H, 0.0001));
      const phase = -n * Math.atan(w / wc) * (180 / Math.PI);
      
      data.push({ freq, mag: magDb });
      phaseArr.push({ freq, phase });
    }
    
    setMagnitudeData(data);
    setPhaseData(phaseArr);
  };

  onMount(() => generateResponse());
  
  createEffect(() => {
    // Re-generate when props change
    filterType();
    cutoff();
    sampleRate();
    order();
    generateResponse();
  });

  const frequencyToX = (freq: number, width: number) => {
    const nyquist = sampleRate() / 2;
    const logMin = 1.3; // 20 Hz
    const logMax = Math.log10(nyquist);
    const logFreq = Math.log10(freq);
    return ((logFreq - logMin) / (logMax - logMin)) * width;
  };

  const magToY = (mag: number, height: number) => {
    // -60dB to +6dB range
    const minDb = -60;
    const maxDb = 6;
    const normalized = (mag - minDb) / (maxDb - minDb);
    return height - normalized * height;
  };

  const phaseToY = (phase: number, height: number) => {
    // -180 to 0 degrees
    const normalized = (phase + 180) / 180;
    return height - normalized * height;
  };

  const getMagnitudePath = () => {
    const width = 400;
    const height = 200;
    const data = magnitudeData();
    if (data.length === 0) return "";
    
    let path = `M ${frequencyToX(data[0].freq, width)} ${magToY(data[0].mag, height)}`;
    for (let i = 1; i < data.length; i++) {
      path += ` L ${frequencyToX(data[i].freq, width)} ${magToY(data[i].mag, height)}`;
    }
    return path;
  };

  const getPhasePath = () => {
    const width = 400;
    const height = 200;
    const data = phaseData();
    if (data.length === 0) return "";
    
    let path = `M ${frequencyToX(data[0].freq, width)} ${phaseToY(data[0].phase, height)}`;
    for (let i = 1; i < data.length; i++) {
      path += ` L ${frequencyToX(data[i].freq, width)} ${phaseToY(data[i].phase, height)}`;
    }
    return path;
  };

  return (
    <div class="filter-response">
      <div class="plot-header">
        <h3>Filter Response</h3>
        <div class="plot-controls">
          <button
            class={viewMode() === "magnitude" ? "active" : ""}
            onClick={() => setViewMode("magnitude")}
          >
            Magnitude
          </button>
          <button
            class={viewMode() === "phase" ? "active" : ""}
            onClick={() => setViewMode("phase")}
          >
            Phase
          </button>
          <button
            class={viewMode() === "both" ? "active" : ""}
            onClick={() => setViewMode("both")}
          >
            Both
          </button>
        </div>
      </div>

      <div class="plot-info">
        <span class="filter-badge">{filterType().toUpperCase()}</span>
        <span>Fc: {cutoff()} Hz</span>
        <span>Fs: {sampleRate()} Hz</span>
        <span>Order: {order()}</span>
      </div>

      <div class="plot-container">
        {(viewMode() === "magnitude" || viewMode() === "both") && (
          <div class="plot-section">
            <div class="y-axis">
              <span>6 dB</span>
              <span>-27 dB</span>
              <span>-60 dB</span>
            </div>
            <svg viewBox="0 0 400 200" class="plot-svg">
              {/* Grid lines */}
              <defs>
                <pattern id="grid" width="40" height="40" patternUnits="userSpaceOnUse">
                  <path d="M 40 0 L 0 0 0 40" fill="none" stroke="#333" stroke-width="0.5" />
                </pattern>
              </defs>
              <rect width="400" height="200" fill="url(#grid)" />
              
              {/* -3dB reference line */}
              <line x1="0" y1={magToY(-3, 200)} x2="400" y2={magToY(-3, 200)} 
                stroke="#ff6b6b" stroke-width="1" stroke-dasharray="4" opacity="0.5" />
              
              {/* Cutoff frequency marker */}
              <line x1={frequencyToX(cutoff(), 400)} y1="0" x2={frequencyToX(cutoff(), 400)} y2="200"
                stroke="#00d4ff" stroke-width="1" stroke-dasharray="4" opacity="0.5" />
              
              {/* Magnitude response curve */}
              <path d={getMagnitudePath()} fill="none" stroke="#3b82f6" stroke-width="2" />
            </svg>
            <div class="x-axis">
              <span>20 Hz</span>
              <span>100 Hz</span>
              <span>1 kHz</span>
              <span>10 kHz</span>
            </div>
            <div class="plot-label">Magnitude Response (dB)</div>
          </div>
        )}

        {(viewMode() === "phase" || viewMode() === "both") && (
          <div class="plot-section">
            <div class="y-axis">
              <span>0°</span>
              <span>-90°</span>
              <span>-180°</span>
            </div>
            <svg viewBox="0 0 400 200" class="plot-svg">
              <defs>
                <pattern id="grid2" width="40" height="40" patternUnits="userSpaceOnUse">
                  <path d="M 40 0 L 0 0 0 40" fill="none" stroke="#333" stroke-width="0.5" />
                </pattern>
              </defs>
              <rect width="400" height="200" fill="url(#grid2)" />
              
              {/* Phase response curve */}
              <path d={getPhasePath()} fill="none" stroke="#10b981" stroke-width="2" />
            </svg>
            <div class="x-axis">
              <span>20 Hz</span>
              <span>100 Hz</span>
              <span>1 kHz</span>
              <span>10 kHz</span>
            </div>
            <div class="plot-label">Phase Response (degrees)</div>
          </div>
        )}
      </div>
    </div>
  );
}

export default FilterResponsePlot;
