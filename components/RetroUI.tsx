
import React, { useState, useEffect, useRef } from 'react';
import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';
import { X, Check, AlertTriangle, Info, Power, ToggleLeft, ToggleRight, Activity, Clock } from 'lucide-react';

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export const Button: React.FC<React.ButtonHTMLAttributes<HTMLButtonElement> & { variant?: 'primary' | 'danger' | 'ghost', tooltip?: string }> = ({ className, variant = 'primary', tooltip, ...props }) => {
  const baseStyles = "px-3 py-1.5 border font-mono text-xs uppercase font-bold tracking-tight transition-all duration-75 active:translate-y-[2px] active:translate-x-[2px] active:shadow-none disabled:opacity-50 disabled:cursor-not-allowed select-none flex items-center justify-center gap-2";
  
  const variants = {
    primary: "bg-neuro-surface border-neuro-primary text-neuro-primary shadow-hard hover:bg-gray-50",
    danger: "bg-red-50 border-red-600 text-red-600 shadow-hard shadow-red-200 hover:bg-red-100",
    ghost: "border-transparent text-gray-500 hover:text-neuro-primary hover:bg-gray-200/50 shadow-none active:translate-y-0 active:translate-x-0"
  };

  const button = (
    <button className={cn(baseStyles, variants[variant], className)} {...props} />
  );

  if (tooltip) {
    return <Tooltip text={tooltip}>{button}</Tooltip>;
  }

  return button;
};

export const Panel: React.FC<{ title: string; children: React.ReactNode; className?: string; actions?: React.ReactNode }> = ({ title, children, className, actions }) => {
  return (
    <div className={cn("bg-neuro-surface border border-neuro-primary flex flex-col shadow-hard", className)}>
      <div className="bg-gray-50 border-b border-neuro-primary px-3 py-2 text-[10px] uppercase font-bold tracking-wider text-neuro-primary flex justify-between items-center select-none">
        <span>{title}</span>
        <div className="flex items-center gap-2">
          {actions}
          <div className="w-1.5 h-1.5 border border-neuro-primary bg-neuro-primary/20 rounded-full"></div>
        </div>
      </div>
      <div className="p-0 overflow-auto flex-1 custom-scrollbar">
        {children}
      </div>
    </div>
  );
};

export const Input: React.FC<React.InputHTMLAttributes<HTMLInputElement>> = ({ className, ...props }) => {
  return (
    <input 
      className={cn(
        "bg-white border border-neuro-dim text-neuro-primary px-3 py-2 text-xs outline-none focus:border-neuro-primary focus:ring-1 focus:ring-neuro-primary/10 transition-all font-mono w-full placeholder:text-gray-300",
        className
      )} 
      {...props} 
    />
  );
};

export const Label: React.FC<{ children: React.ReactNode }> = ({ children }) => (
  <label className="block text-[9px] uppercase font-bold text-gray-500 mb-1 tracking-widest">{children}</label>
);

export const Tooltip: React.FC<{ text: string; children: React.ReactNode }> = ({ text, children }) => {
  const [show, setShow] = useState(false);
  
  return (
    <div className="relative flex items-center" onMouseEnter={() => setShow(true)} onMouseLeave={() => setShow(false)}>
      {children}
      {show && (
        <div className="absolute top-full left-1/2 -translate-x-1/2 mt-2 px-2 py-1 bg-neuro-primary text-white text-[10px] whitespace-nowrap z-50 pointer-events-none shadow-lg border border-white/20 animate-in fade-in zoom-in-95 duration-100 origin-top">
          {text}
          <div className="absolute -top-1 left-1/2 -translate-x-1/2 border-4 border-transparent border-b-neuro-primary"></div>
        </div>
      )}
    </div>
  );
};

export const ProgressBar: React.FC<{ value: number; max?: number; color?: string; label?: string }> = ({ value, max = 100, color = "bg-blue-600", label }) => {
  const pct = Math.min(100, Math.max(0, (value / max) * 100));
  return (
    <div className="w-full">
      {label && <div className="flex justify-between text-[9px] text-gray-500 uppercase font-bold mb-1"><span>{label}</span><span>{Math.round(value)}</span></div>}
      <div className="h-1.5 w-full bg-gray-200 rounded-sm overflow-hidden">
        <div className={cn("h-full transition-all duration-300", color)} style={{ width: `${pct}%` }}></div>
      </div>
    </div>
  );
};

export const MetricCard: React.FC<{ label: string; value: string | number; unit?: string }> = ({ label, value, unit }) => (
  <div className="flex flex-col border border-neuro-dim p-2 bg-white">
    <span className="text-[9px] text-gray-400 font-bold uppercase tracking-wider">{label}</span>
    <span className="text-lg font-bold text-neuro-primary leading-none mt-1">
      {value}<span className="text-[10px] text-gray-400 ml-0.5">{unit}</span>
    </span>
  </div>
);

// Toast Notification System
export interface ToastMessage {
  id: string;
  type: 'success' | 'error' | 'info' | 'warning';
  message: string;
}

export const Toast: React.FC<ToastMessage & { onClose: (id: string) => void }> = ({ id, type, message, onClose }) => {
  useEffect(() => {
    const timer = setTimeout(() => onClose(id), 4000);
    return () => clearTimeout(timer);
  }, [id, onClose]);

  const icons = {
    success: <Check size={14} />,
    error: <X size={14} />,
    warning: <AlertTriangle size={14} />,
    info: <Info size={14} />
  };

  const styles = {
    success: "bg-green-50 border-green-600 text-green-700",
    error: "bg-red-50 border-red-600 text-red-700",
    warning: "bg-orange-50 border-orange-600 text-orange-700",
    info: "bg-white border-neuro-primary text-neuro-primary"
  };

  return (
    <div className={cn(
      "flex items-center gap-3 px-4 py-3 border shadow-hard mb-2 min-w-[280px] animate-slide-in font-mono text-xs",
      styles[type]
    )}>
      <div className={cn("p-1 rounded-full border border-current opacity-70")}>
        {icons[type]}
      </div>
      <span className="flex-1 font-bold">{message}</span>
      <button onClick={() => onClose(id)} className="opacity-50 hover:opacity-100">
        <X size={12} />
      </button>
    </div>
  );
};

// --- VIRTUAL HARDWARE COMPONENTS ---

export const VirtualLED: React.FC<{ label: string; active: boolean; color?: 'red' | 'green' | 'blue' | 'yellow' }> = ({ label, active, color = 'red' }) => {
  const colors = {
    red: active ? "bg-red-500 shadow-[0_0_10px_rgba(239,68,68,0.8)] border-red-600" : "bg-red-900 border-red-950 opacity-20",
    green: active ? "bg-green-500 shadow-[0_0_10px_rgba(34,197,94,0.8)] border-green-600" : "bg-green-900 border-green-950 opacity-20",
    blue: active ? "bg-blue-500 shadow-[0_0_10px_rgba(59,130,246,0.8)] border-blue-600" : "bg-blue-900 border-blue-950 opacity-20",
    yellow: active ? "bg-yellow-400 shadow-[0_0_10px_rgba(250,204,21,0.8)] border-yellow-600" : "bg-yellow-900 border-yellow-950 opacity-20",
  };

  return (
    <div className="flex flex-col items-center gap-2 p-2 border border-neuro-dim bg-white shadow-sm w-[70px]">
      <div className={cn("w-6 h-6 rounded-full border-2 transition-all duration-100", colors[color])}></div>
      <span className="text-[9px] font-bold text-gray-500 uppercase truncate max-w-full">{label}</span>
    </div>
  );
};

export const VirtualSwitch: React.FC<{ label: string; active: boolean; onChange: (v: boolean) => void }> = ({ label, active, onChange }) => {
  return (
    <div className="flex flex-col items-center gap-2 p-2 border border-neuro-dim bg-white shadow-sm w-[70px]">
      <button onClick={() => onChange(!active)} className="text-neuro-primary hover:text-neuro-accent transition-colors">
        {active ? <ToggleRight size={24} className="fill-neuro-primary text-white" /> : <ToggleLeft size={24} className="text-gray-400"/>}
      </button>
      <span className="text-[9px] font-bold text-gray-500 uppercase truncate max-w-full">{label}</span>
    </div>
  );
};

export const VirtualDisplay: React.FC<{ label: string; value: string | number }> = ({ label, value }) => {
  return (
    <div className="flex flex-col items-center gap-1 p-2 border border-neuro-dim bg-white shadow-sm w-[80px]">
      <div className="bg-gray-900 text-red-500 font-mono text-lg px-2 py-0.5 border-2 border-gray-700 w-full text-center shadow-inner tracking-widest font-bold">
        {String(value).substring(0, 4)}
      </div>
      <span className="text-[9px] font-bold text-gray-500 uppercase truncate max-w-full">{label}</span>
    </div>
  );
};

// --- LOGIC ANALYZER COMPONENT ---

interface LogicSample {
  timestamp: number;
  signals: Record<string, number | boolean>;
}

export const LogicAnalyzer: React.FC<{ 
  history: LogicSample[]; 
  channels: string[]; 
  height?: number 
}> = ({ history, channels, height = 200 }) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || history.length < 2) return;
    
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    ctx.scale(dpr, dpr);

    // Styling
    ctx.fillStyle = '#111827'; // Dark BG
    ctx.fillRect(0, 0, rect.width, rect.height);
    
    // Grid Lines
    ctx.strokeStyle = '#374151';
    ctx.lineWidth = 0.5;
    ctx.beginPath();
    for (let x = 0; x < rect.width; x += 50) {
      ctx.moveTo(x, 0); ctx.lineTo(x, rect.height);
    }
    ctx.stroke();

    const channelHeight = (rect.height - 20) / channels.length;
    const timeWindow = 5000; // Show last 5 seconds
    const now = history[history.length - 1].timestamp;
    const startTime = now - timeWindow;

    channels.forEach((ch, idx) => {
      const yBase = 10 + (idx * channelHeight) + (channelHeight * 0.8);
      const yHigh = yBase - (channelHeight * 0.6);
      
      // Draw Label
      ctx.fillStyle = '#9ca3af';
      ctx.font = '10px monospace';
      ctx.fillText(ch, 5, yBase - (channelHeight * 0.3));

      // Draw Waveform
      ctx.strokeStyle = '#22c55e'; // Green
      ctx.lineWidth = 1.5;
      ctx.beginPath();

      let started = false;

      history.forEach((sample, i) => {
        if (sample.timestamp < startTime) return;
        
        const x = ((sample.timestamp - startTime) / timeWindow) * rect.width;
        const val = sample.signals[ch];
        const y = val ? yHigh : yBase;

        if (!started) {
          ctx.moveTo(x, y);
          started = true;
        } else {
          // Digital square wave logic
          const prevSample = history[i - 1];
          const prevVal = prevSample?.signals[ch];
          const prevY = prevVal ? yHigh : yBase;
          
          ctx.lineTo(x, prevY); // Hold previous value
          ctx.lineTo(x, y);     // Vertical transition
        }
      });
      
      // Continue to end of screen
      if (history.length > 0) {
         const lastVal = history[history.length - 1].signals[ch];
         const lastY = lastVal ? yHigh : yBase;
         ctx.lineTo(rect.width, lastY);
      }

      ctx.stroke();
    });

  }, [history, channels]);

  return (
    <div className="w-full h-full relative">
      <div className="absolute top-2 right-2 text-[9px] text-green-500 font-bold flex items-center gap-1 bg-black/50 px-2 rounded">
         <Activity size={10} className="animate-pulse"/> LIVE CAPTURE
      </div>
      <canvas ref={canvasRef} className="w-full h-full block" style={{ height: `${height}px` }} />
    </div>
  );
};
