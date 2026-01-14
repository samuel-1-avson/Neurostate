/**
 * CanvasRulers - Professional rulers for canvas measurement
 * Pixel-accurate measurement with zoom support
 */

import { Component, Show } from "solid-js";
import "./CanvasRulers.css";

interface CanvasRulersProps {
  zoom?: number;
  panOffset?: { x: number; y: number };
  width: number;
  height: number;
  showRulers?: boolean;
  rulerSize?: number;
  tickUnit?: number;
}

export const CanvasRulers: Component<CanvasRulersProps> = (props) => {
  const zoom = () => props.zoom || 1;
  const panOffset = () => props.panOffset || { x: 0, y: 0 };
  const rulerSize = () => props.rulerSize || 20;
  const tickUnit = () => props.tickUnit || 10;
  
  // Generate ruler ticks
  const generateTicks = (length: number, isHorizontal: boolean) => {
    const ticks: Array<{ pos: number; label: string; major: boolean }> = [];
    const scale = zoom();
    const offset = isHorizontal ? panOffset().x : panOffset().y;
    
    const startValue = Math.floor(-offset / scale / tickUnit()) * tickUnit();
    const endValue = Math.ceil((length + -offset) / scale / tickUnit()) * tickUnit();
    
    for (let value = startValue; value <= endValue; value += tickUnit()) {
      const pos = (value * scale) + offset;
      if (pos >= 0 && pos <= length) {
        const isMajor = value % (tickUnit() * 10) === 0;
        ticks.push({
          pos,
          label: isMajor ? String(value) : "",
          major: isMajor
        });
      }
    }
    
    return ticks;
  };
  
  return (
    <Show when={props.showRulers !== false}>
      <div class="canvas-rulers">
        {/* Corner square */}
        <div 
          class="ruler-corner" 
          style={{ width: `${rulerSize()}px`, height: `${rulerSize()}px` }}
        >
          <span class="corner-unit">px</span>
        </div>
        
        {/* Horizontal ruler */}
        <div 
          class="ruler ruler-horizontal" 
          style={{ 
            left: `${rulerSize()}px`, 
            height: `${rulerSize()}px`,
            width: `calc(100% - ${rulerSize()}px)` 
          }}
        >
          <svg width="100%" height="100%">
            {generateTicks(props.width - rulerSize(), true).map((tick) => (
              <>
                <line
                  x1={tick.pos}
                  y1={tick.major ? 0 : rulerSize() * 0.6}
                  x2={tick.pos}
                  y2={rulerSize()}
                  class={tick.major ? "ruler-tick-major" : "ruler-tick-minor"}
                />
                {tick.label && (
                  <text
                    x={tick.pos + 3}
                    y={10}
                    class="ruler-label"
                  >
                    {tick.label}
                  </text>
                )}
              </>
            ))}
          </svg>
        </div>
        
        {/* Vertical ruler */}
        <div 
          class="ruler ruler-vertical" 
          style={{ 
            top: `${rulerSize()}px`, 
            width: `${rulerSize()}px`,
            height: `calc(100% - ${rulerSize()}px)` 
          }}
        >
          <svg width="100%" height="100%">
            {generateTicks(props.height - rulerSize(), false).map((tick) => (
              <>
                <line
                  y1={tick.pos}
                  x1={tick.major ? 0 : rulerSize() * 0.6}
                  y2={tick.pos}
                  x2={rulerSize()}
                  class={tick.major ? "ruler-tick-major" : "ruler-tick-minor"}
                />
                {tick.label && (
                  <text
                    y={tick.pos + 3}
                    x={2}
                    class="ruler-label ruler-label-vertical"
                    transform={`rotate(-90, 10, ${tick.pos})`}
                  >
                    {tick.label}
                  </text>
                )}
              </>
            ))}
          </svg>
        </div>
      </div>
    </Show>
  );
};

// Alignment Guides
interface AlignmentGuidesProps {
  guides?: Array<{
    type: "horizontal" | "vertical";
    position: number;
    color?: string;
  }>;
  zoom?: number;
  panOffset?: { x: number; y: number };
}

export const AlignmentGuides: Component<AlignmentGuidesProps> = (props) => {
  const zoom = () => props.zoom || 1;
  const panOffset = () => props.panOffset || { x: 0, y: 0 };
  
  return (
    <div class="alignment-guides">
      {(props.guides || []).map((guide) => {
        const pos = guide.position * zoom() + (guide.type === "horizontal" ? panOffset().y : panOffset().x);
        
        return (
          <div
            class={`alignment-guide ${guide.type}`}
            style={{
              [guide.type === "horizontal" ? "top" : "left"]: `${pos}px`,
              "background-color": guide.color || "var(--accent, #6366f1)"
            }}
          />
        );
      })}
    </div>
  );
};

export default CanvasRulers;
