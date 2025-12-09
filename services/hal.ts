
type HalListener = (snapshot: HalSnapshot) => void;

export interface HalSnapshot {
  gpio: Record<number, boolean>;
  pwm: Record<number, number>;
  uartTx: string[];
  uartRx: string[];
}

export const HAL = {
  // Simulated Hardware State (Persistent across calls)
  _gpio: new Map<number, boolean>(),
  _pwm: new Map<number, number>(),
  _uartTxBuffer: [] as string[],
  _uartRxBuffer: ["OK", "ERROR", "READY"] as string[], // Mock incoming data
  
  _listeners: [] as HalListener[],

  subscribe(listener: HalListener): () => void {
    this._listeners.push(listener);
    // Send immediate state
    listener(this.getSnapshot());
    
    return () => {
      this._listeners = this._listeners.filter(l => l !== listener);
    };
  },

  notify() {
    const snapshot = this.getSnapshot();
    this._listeners.forEach(l => l(snapshot));
  },

  getSnapshot(): HalSnapshot {
    const gpio: Record<number, boolean> = {};
    this._gpio.forEach((v, k) => gpio[k] = v);
    
    const pwm: Record<number, number> = {};
    this._pwm.forEach((v, k) => pwm[k] = v);

    return {
      gpio,
      pwm,
      // Guard against potentially undefined buffers
      uartTx: (this._uartTxBuffer || []).slice(),
      uartRx: (this._uartRxBuffer || []).slice()
    };
  },

  /**
   * Reads the digital state of a GPIO pin.
   * Default state is LOW (false).
   * @param pin Pin number
   * @returns boolean High (true) or Low (false)
   */
  readPin(pin: number): boolean {
    const val = this._gpio.get(pin) || false;
    console.log(`[HAL] READ GPIO_${pin}: ${val ? 'HIGH' : 'LOW'}`);
    return val;
  },

  /**
   * Writes a digital value to a GPIO pin.
   * Updates internal state for subsequent reads.
   * @param pin Pin number
   * @param value High (true) or Low (false)
   */
  writePin(pin: number, value: boolean): void {
    this._gpio.set(pin, value);
    console.log(`[HAL] WRITE GPIO_${pin}: ${value ? 'HIGH' : 'LOW'}`);
    this.notify();
  },

  /**
   * Reads an Analog-to-Digital Converter channel.
   * Returns a simulated 12-bit value (0-4095).
   * Uses a sine wave function based on time to simulate changing sensor data.
   * @param channel Channel number
   * @returns number 12-bit value
   */
  getADC(channel: number): number {
    // Generate a predictable but changing value for simulation "liveliness"
    const time = Date.now() / 2000; // Slow oscillation
    // Base 2048 +/- 1000, offset by channel to vary different inputs
    const val = Math.floor(2048 + 1000 * Math.sin(time + channel)); 
    // Clamp to 12-bit range
    const clamped = Math.max(0, Math.min(4095, val));
    
    console.log(`[HAL] READ ADC_${channel}: ${clamped}`);
    return clamped;
  },

  /**
   * Sets the Duty Cycle for a PWM channel.
   * @param channel Channel number
   * @param dutyCycle Percentage (0-100)
   */
  setPWM(channel: number, dutyCycle: number): void {
    const safeDuty = Math.max(0, Math.min(100, dutyCycle));
    this._pwm.set(channel, safeDuty);
    console.log(`[HAL] SET PWM_${channel}: ${safeDuty}%`);
    this.notify();
  },

  /**
   * Simulates entering a low-power sleep mode.
   * @param mode Sleep depth
   */
  enterSleepMode(mode: 'WFI' | 'STOP' | 'STANDBY'): void {
    console.log(`[HAL] CPU ENTERING SLEEP MODE: [${mode}]`);
    // In a full simulation, this might pause the FSMExecutor
  },

  /**
   * Transmits data via UART (Simulated).
   * Logs to console and stores in a generic TX buffer.
   * @param data String data to send
   */
  UART_Transmit(data: string): void {
    if (!this._uartTxBuffer) this._uartTxBuffer = [];
    this._uartTxBuffer.push(data);
    // Keep buffer manageable
    if (this._uartTxBuffer.length > 20) this._uartTxBuffer.shift();
    console.log(`[HAL] UART_TX >> "${data}"`);
    this.notify();
  },

  /**
   * Checks for received data in the simulated UART buffer.
   * Returns null if empty.
   */
  UART_Receive(): string | null {
    if (!this._uartRxBuffer) this._uartRxBuffer = [];
    
    // Simulate random incoming data if buffer is empty for "liveness"
    if (Math.random() > 0.98 && this._uartRxBuffer.length === 0) {
       // Only inject random noise if explicit injection hasn't happened recently
       // this._uartRxBuffer.push("PING"); 
       // this.notify();
    }

    if (this._uartRxBuffer.length > 0) {
       const data = this._uartRxBuffer.shift() || null;
       console.log(`[HAL] UART_RX << "${data}"`);
       this.notify(); // Rx buffer changed
       return data;
    }
    return null;
  },

  /**
   * Externally inject data into the RX buffer (e.g. from UI Serial Monitor)
   */
  mockReceive(data: string): void {
    if (!this._uartRxBuffer) this._uartRxBuffer = [];
    this._uartRxBuffer.push(data);
    console.log(`[HAL] MOCK_RX_INJECT >> "${data}"`);
    this.notify();
  },

  /**
   * Reset the hardware state (e.g. on simulation stop)
   */
  reset(): void {
    this._gpio.clear();
    this._pwm.clear();
    this._uartTxBuffer = [];
    this._uartRxBuffer = ["OK", "ERROR", "READY"]; // Reset mock data
    console.log(`[HAL] HARDWARE RESET`);
    this.notify();
  }
};
