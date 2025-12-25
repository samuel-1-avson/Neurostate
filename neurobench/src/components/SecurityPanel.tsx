import { createSignal, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { ValidationPanel } from "./ValidationPanel";
import { CodePreview } from "./CodePreview";

interface SecurityPanelProps {
  onLog?: (source: string, message: string, type?: "info" | "success" | "warning" | "error") => void;
}

export function SecurityPanel(props: SecurityPanelProps) {
  // Security type selection
  const [secType, setSecType] = createSignal<"bootloader" | "ota" | "secureboot" | "crypto">("bootloader");
  
  // Bootloader config
  const [bootName, setBootName] = createSignal("bootloader");
  const [bootType, setBootType] = createSignal("dual");
  const [flashBase, setFlashBase] = createSignal(0x08000000);
  const [flashSize, setFlashSize] = createSignal(512);
  const [bootloaderSize, setBootloaderSize] = createSignal(32);
  const [appSize, setAppSize] = createSignal(224);
  const [bootWatchdog, setBootWatchdog] = createSignal(true);
  const [bootCrc, setBootCrc] = createSignal(true);
  
  // OTA config
  const [otaName, setOtaName] = createSignal("ota_updater");
  const [otaTransport, setOtaTransport] = createSignal("https");
  const [otaServer, setOtaServer] = createSignal("https://firmware.example.com");
  const [otaPath, setOtaPath] = createSignal("/firmware/latest.bin");
  const [otaChunkSize, setOtaChunkSize] = createSignal(4096);
  const [otaVerifySig, setOtaVerifySig] = createSignal(true);
  
  // Secure boot config
  const [sbName, setSbName] = createSignal("secure_boot");
  const [sbAlgorithm, setSbAlgorithm] = createSignal("ecdsa256");
  const [sbRollback, setSbRollback] = createSignal(true);
  const [sbDebugLock, setSbDebugLock] = createSignal(true);
  
  // Crypto config
  const [cryptoName, setCryptoName] = createSignal("crypto_utils");
  const [cryptoAes, setCryptoAes] = createSignal(true);
  const [cryptoHash, setCryptoHash] = createSignal(true);
  const [cryptoRng, setCryptoRng] = createSignal(true);
  const [cryptoEcdsa, setCryptoEcdsa] = createSignal(true);
  const [cryptoHashAlgo, setCryptoHashAlgo] = createSignal("sha256");
  
  // Generated code
  const [generatedCode, setGeneratedCode] = createSignal("");
  const [isGenerating, setIsGenerating] = createSignal(false);

  const addLog = (source: string, message: string, type: "info" | "success" | "warning" | "error" = "info") => {
    props.onLog?.(source, message, type);
  };

  const generateBootloader = async () => {
    setIsGenerating(true);
    try {
      const result = await invoke("generate_bootloader", {
        name: bootName(),
        bootloaderType: bootType(),
        flashBase: flashBase(),
        flashSize: flashSize() * 1024,
        bootloaderSize: bootloaderSize() * 1024,
        appSize: appSize() * 1024,
        enableWatchdog: bootWatchdog(),
        enableCrc: bootCrc(),
      }) as any;
      setGeneratedCode(result.code);
      addLog("Security", `Generated ${bootType()} bootloader: ${bootName()}`, "success");
    } catch (e) {
      addLog("ERROR", `Failed to generate bootloader: ${e}`, "error");
    }
    setIsGenerating(false);
  };

  const generateOta = async () => {
    setIsGenerating(true);
    try {
      const result = await invoke("generate_ota_client", {
        name: otaName(),
        transport: otaTransport(),
        serverUrl: otaServer(),
        firmwarePath: otaPath(),
        chunkSize: otaChunkSize(),
        verifySignature: otaVerifySig(),
      }) as any;
      setGeneratedCode(result.code);
      addLog("Security", `Generated OTA client: ${otaName()} (${otaTransport()})`, "success");
    } catch (e) {
      addLog("ERROR", `Failed to generate OTA: ${e}`, "error");
    }
    setIsGenerating(false);
  };

  const generateSecureBoot = async () => {
    setIsGenerating(true);
    try {
      const result = await invoke("generate_secure_boot", {
        name: sbName(),
        algorithm: sbAlgorithm(),
        enableRollback: sbRollback(),
        enableDebugLock: sbDebugLock(),
      }) as any;
      setGeneratedCode(result.code);
      addLog("Security", `Generated secure boot: ${sbName()} (${sbAlgorithm()})`, "success");
    } catch (e) {
      addLog("ERROR", `Failed to generate secure boot: ${e}`, "error");
    }
    setIsGenerating(false);
  };

  const generateCrypto = async () => {
    setIsGenerating(true);
    try {
      const result = await invoke("generate_crypto_utils", {
        name: cryptoName(),
        includeAes: cryptoAes(),
        includeHash: cryptoHash(),
        includeRng: cryptoRng(),
        includeEcdsa: cryptoEcdsa(),
        hashAlgorithm: cryptoHashAlgo(),
      }) as any;
      setGeneratedCode(result.code);
      addLog("Security", `Generated crypto utils: ${cryptoName()}`, "success");
    } catch (e) {
      addLog("ERROR", `Failed to generate crypto: ${e}`, "error");
    }
    setIsGenerating(false);
  };

  const copyToClipboard = () => {
    navigator.clipboard.writeText(generatedCode());
    addLog("Security", "Code copied to clipboard", "info");
  };

  return (
    <div class="security-panel">
      <div class="security-header">
        <h3>🔒 Security Configuration</h3>
      </div>

      {/* Security Type Tabs */}
      <div class="security-tabs">
        <button class={`tab ${secType() === "bootloader" ? "active" : ""}`} onClick={() => setSecType("bootloader")}>
          Boot
        </button>
        <button class={`tab ${secType() === "ota" ? "active" : ""}`} onClick={() => setSecType("ota")}>
          OTA
        </button>
        <button class={`tab ${secType() === "secureboot" ? "active" : ""}`} onClick={() => setSecType("secureboot")}>
          Secure
        </button>
        <button class={`tab ${secType() === "crypto" ? "active" : ""}`} onClick={() => setSecType("crypto")}>
          Crypto
        </button>
      </div>

      {/* Bootloader Config */}
      <Show when={secType() === "bootloader"}>
        <div class="config-section">
          <div class="config-row">
            <label>Bootloader Name</label>
            <input type="text" value={bootName()} onInput={(e) => setBootName(e.target.value)} />
          </div>
          <div class="config-row">
            <label>Type</label>
            <select value={bootType()} onChange={(e) => setBootType(e.target.value)}>
              <option value="single">Single Bank</option>
              <option value="dual">Dual Bank</option>
              <option value="dual_rollback">Dual Bank + Rollback</option>
            </select>
          </div>
          <div class="config-row">
            <label>Flash Base (hex)</label>
            <input type="text" value={`0x${flashBase().toString(16).toUpperCase()}`} 
                   onInput={(e) => setFlashBase(parseInt(e.target.value, 16) || 0x08000000)} />
          </div>
          <div class="config-row-group">
            <div class="config-row">
              <label>Flash Size (KB)</label>
              <input type="number" value={flashSize()} onInput={(e) => setFlashSize(parseInt(e.target.value) || 512)} />
            </div>
            <div class="config-row">
              <label>Boot Size (KB)</label>
              <input type="number" value={bootloaderSize()} onInput={(e) => setBootloaderSize(parseInt(e.target.value) || 32)} />
            </div>
          </div>
          <div class="config-row">
            <label>App Size per Bank (KB)</label>
            <input type="number" value={appSize()} onInput={(e) => setAppSize(parseInt(e.target.value) || 224)} />
          </div>
          <div class="config-row checkbox">
            <label>
              <input type="checkbox" checked={bootWatchdog()} onChange={(e) => setBootWatchdog(e.target.checked)} />
              Enable Watchdog
            </label>
          </div>
          <div class="config-row checkbox">
            <label>
              <input type="checkbox" checked={bootCrc()} onChange={(e) => setBootCrc(e.target.checked)} />
              Enable CRC Check
            </label>
          </div>
          <button class="generate-btn" onClick={generateBootloader} disabled={isGenerating()}>
            {isGenerating() ? "Generating..." : "Generate Bootloader"}
          </button>
        </div>
      </Show>

      {/* OTA Config */}
      <Show when={secType() === "ota"}>
        <div class="config-section">
          <div class="config-row">
            <label>OTA Client Name</label>
            <input type="text" value={otaName()} onInput={(e) => setOtaName(e.target.value)} />
          </div>
          <div class="config-row">
            <label>Transport</label>
            <select value={otaTransport()} onChange={(e) => setOtaTransport(e.target.value)}>
              <option value="https">HTTPS</option>
              <option value="http">HTTP</option>
              <option value="mqtt">MQTT</option>
              <option value="ble">BLE</option>
            </select>
          </div>
          <div class="config-row">
            <label>Server URL</label>
            <input type="text" value={otaServer()} onInput={(e) => setOtaServer(e.target.value)} />
          </div>
          <div class="config-row">
            <label>Firmware Path</label>
            <input type="text" value={otaPath()} onInput={(e) => setOtaPath(e.target.value)} />
          </div>
          <div class="config-row">
            <label>Chunk Size (bytes)</label>
            <input type="number" value={otaChunkSize()} onInput={(e) => setOtaChunkSize(parseInt(e.target.value) || 4096)} />
          </div>
          <div class="config-row checkbox">
            <label>
              <input type="checkbox" checked={otaVerifySig()} onChange={(e) => setOtaVerifySig(e.target.checked)} />
              Verify Signature
            </label>
          </div>
          <button class="generate-btn" onClick={generateOta} disabled={isGenerating()}>
            {isGenerating() ? "Generating..." : "Generate OTA Client"}
          </button>
        </div>
      </Show>

      {/* Secure Boot Config */}
      <Show when={secType() === "secureboot"}>
        <div class="config-section">
          <div class="config-row">
            <label>Secure Boot Name</label>
            <input type="text" value={sbName()} onInput={(e) => setSbName(e.target.value)} />
          </div>
          <div class="config-row">
            <label>Algorithm</label>
            <select value={sbAlgorithm()} onChange={(e) => setSbAlgorithm(e.target.value)}>
              <option value="ecdsa256">ECDSA P-256</option>
              <option value="ecdsa384">ECDSA P-384</option>
              <option value="rsa2048">RSA-2048</option>
              <option value="rsa4096">RSA-4096</option>
              <option value="ed25519">Ed25519</option>
            </select>
          </div>
          <div class="config-row checkbox">
            <label>
              <input type="checkbox" checked={sbRollback()} onChange={(e) => setSbRollback(e.target.checked)} />
              Anti-Rollback Protection
            </label>
          </div>
          <div class="config-row checkbox">
            <label>
              <input type="checkbox" checked={sbDebugLock()} onChange={(e) => setSbDebugLock(e.target.checked)} />
              Lock Debug Access
            </label>
          </div>
          <button class="generate-btn" onClick={generateSecureBoot} disabled={isGenerating()}>
            {isGenerating() ? "Generating..." : "Generate Secure Boot"}
          </button>
        </div>
      </Show>

      {/* Crypto Config */}
      <Show when={secType() === "crypto"}>
        <div class="config-section">
          <div class="config-row">
            <label>Crypto Module Name</label>
            <input type="text" value={cryptoName()} onInput={(e) => setCryptoName(e.target.value)} />
          </div>
          <div class="config-row">
            <label>Hash Algorithm</label>
            <select value={cryptoHashAlgo()} onChange={(e) => setCryptoHashAlgo(e.target.value)}>
              <option value="sha256">SHA-256</option>
              <option value="sha384">SHA-384</option>
              <option value="sha512">SHA-512</option>
              <option value="sha3">SHA3-256</option>
            </select>
          </div>
          <div class="crypto-checkboxes">
            <div class="config-row checkbox">
              <label>
                <input type="checkbox" checked={cryptoAes()} onChange={(e) => setCryptoAes(e.target.checked)} />
                AES Encryption
              </label>
            </div>
            <div class="config-row checkbox">
              <label>
                <input type="checkbox" checked={cryptoHash()} onChange={(e) => setCryptoHash(e.target.checked)} />
                Hash Functions
              </label>
            </div>
            <div class="config-row checkbox">
              <label>
                <input type="checkbox" checked={cryptoRng()} onChange={(e) => setCryptoRng(e.target.checked)} />
                RNG
              </label>
            </div>
            <div class="config-row checkbox">
              <label>
                <input type="checkbox" checked={cryptoEcdsa()} onChange={(e) => setCryptoEcdsa(e.target.checked)} />
                ECDSA Signatures
              </label>
            </div>
          </div>
          <button class="generate-btn" onClick={generateCrypto} disabled={isGenerating()}>
            {isGenerating() ? "Generating..." : "Generate Crypto Utils"}
          </button>
        </div>
      </Show>

      {/* Generated Code */}
      <Show when={generatedCode()}>
        <div class="code-output">
          <CodePreview 
            code={generatedCode()} 
            language="c" 
            showLineNumbers={true}
            onCopy={() => addLog("Security", "Code copied to clipboard", "info")}
          />
          
          {/* Code Validation */}
          <ValidationPanel 
            code={generatedCode()} 
            language="c" 
            onLog={props.onLog} 
          />
        </div>
      </Show>
    </div>
  );
}
