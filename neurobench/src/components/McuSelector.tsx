import { createSignal, For, Show, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface McuInfo {
  family: string;
  display_name: string;
  vendor: string;
  architecture: string;
  max_freq_mhz: number;
  flash_kb: number;
  ram_kb: number;
  has_fpu: boolean;
  has_dsp: boolean;
  has_ble: boolean;
  has_wifi: boolean;
}

interface McuSelectorProps {
  onSelect?: (mcu: McuInfo) => void;
  onLog?: (source: string, message: string, type?: "info" | "success" | "warning" | "error") => void;
}

export function McuSelector(props: McuSelectorProps) {
  const [mcus, setMcus] = createSignal<McuInfo[]>([]);
  const [selectedMcu, setSelectedMcu] = createSignal<McuInfo | null>(null);
  const [filterVendor, setFilterVendor] = createSignal<string>("all");
  const [loading, setLoading] = createSignal(true);

  const vendors = ["all", "STMicroelectronics", "Espressif", "Raspberry Pi", "Nordic Semiconductor", "NXP"];

  onMount(async () => {
    try {
      const result = await invoke("get_supported_mcus") as any;
      setMcus(result.mcus);
      addLog("MCU", `Loaded ${result.count} MCU families`, "success");
    } catch (e) {
      addLog("ERROR", `Failed to load MCUs: ${e}`, "error");
    }
    setLoading(false);
  });

  const addLog = (source: string, message: string, type: "info" | "success" | "warning" | "error") => {
    props.onLog?.(source, message, type);
  };

  const filteredMcus = () => {
    const vendor = filterVendor();
    if (vendor === "all") return mcus();
    return mcus().filter(m => m.vendor === vendor);
  };

  const selectMcu = (mcu: McuInfo) => {
    setSelectedMcu(mcu);
    props.onSelect?.(mcu);
    addLog("MCU", `Selected ${mcu.display_name}`, "info");
  };

  const getVendorColor = (vendor: string) => {
    switch (vendor) {
      case "STMicroelectronics": return "#03234b";
      case "Espressif": return "#e7352c";
      case "Raspberry Pi": return "#c51a4a";
      case "Nordic Semiconductor": return "#00a9ce";
      case "NXP": return "#000080";
      default: return "#666";
    }
  };

  return (
    <div class="mcu-selector">
      <div class="mcu-header">
        <h3>🎯 Target MCU</h3>
        <Show when={selectedMcu()}>
          <span class="selected-badge">{selectedMcu()?.family}</span>
        </Show>
      </div>

      {/* Vendor Filter */}
      <div class="vendor-filter">
        <For each={vendors}>
          {(vendor) => (
            <button
              class={`vendor-btn ${filterVendor() === vendor ? "active" : ""}`}
              onClick={() => setFilterVendor(vendor)}
              style={filterVendor() === vendor ? { "background-color": getVendorColor(vendor) } : {}}
            >
              {vendor === "all" ? "All" : vendor.split(" ")[0]}
            </button>
          )}
        </For>
      </div>

      {/* MCU Grid */}
      <Show when={!loading()} fallback={<div class="loading">Loading MCUs...</div>}>
        <div class="mcu-grid">
          <For each={filteredMcus()}>
            {(mcu) => (
              <div
                class={`mcu-card ${selectedMcu()?.family === mcu.family ? "selected" : ""}`}
                onClick={() => selectMcu(mcu)}
              >
                <div class="mcu-card-header" style={{ "border-left-color": getVendorColor(mcu.vendor) }}>
                  <span class="mcu-family">{mcu.family}</span>
                  <span class="mcu-arch">{mcu.architecture}</span>
                </div>
                <div class="mcu-card-body">
                  <div class="mcu-stat">
                    <span class="stat-label">Freq</span>
                    <span class="stat-value">{mcu.max_freq_mhz} MHz</span>
                  </div>
                  <div class="mcu-stat">
                    <span class="stat-label">Flash</span>
                    <span class="stat-value">{mcu.flash_kb >= 1024 ? `${mcu.flash_kb/1024}MB` : `${mcu.flash_kb}KB`}</span>
                  </div>
                  <div class="mcu-stat">
                    <span class="stat-label">RAM</span>
                    <span class="stat-value">{mcu.ram_kb >= 1024 ? `${mcu.ram_kb/1024}MB` : `${mcu.ram_kb}KB`}</span>
                  </div>
                </div>
                <div class="mcu-features">
                  {mcu.has_fpu && <span class="feature-badge fpu">FPU</span>}
                  {mcu.has_dsp && <span class="feature-badge dsp">DSP</span>}
                  {mcu.has_ble && <span class="feature-badge ble">BLE</span>}
                  {mcu.has_wifi && <span class="feature-badge wifi">WiFi</span>}
                </div>
              </div>
            )}
          </For>
        </div>
      </Show>

      {/* Selected MCU Details */}
      <Show when={selectedMcu()}>
        <div class="mcu-details">
          <h4>{selectedMcu()?.display_name}</h4>
          <div class="details-grid">
            <div class="detail-item">
              <span class="detail-label">Vendor</span>
              <span class="detail-value">{selectedMcu()?.vendor}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">Architecture</span>
              <span class="detail-value">{selectedMcu()?.architecture}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">Max Frequency</span>
              <span class="detail-value">{selectedMcu()?.max_freq_mhz} MHz</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">Flash Memory</span>
              <span class="detail-value">{selectedMcu()?.flash_kb} KB</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">RAM</span>
              <span class="detail-value">{selectedMcu()?.ram_kb} KB</span>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
}
