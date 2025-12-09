
import { Node, Edge, MarkerType } from 'reactflow';

export interface FSMTemplate {
  id: string;
  name: string;
  category: 'BASIC' | 'BOOTLOADER' | 'COMMUNICATION' | 'MOTOR_CONTROL' | 'RTOS' | 'POWER_MGMT' | 'SAFETY' | 'IOT' | 'DSP' | 'DRIVER' | 'SYSTEM';
  description: string;
  nodes: Node[];
  edges: Edge[];
}

const createNode = (id: string, type: 'input'|'process'|'output'|'error'|'hardware'|'decision'|'uart', label: string, x: number, y: number, entry: string = ''): Node => ({
  id, type, position: { x, y }, data: { label, type, entryAction: entry, exitAction: '' }
});

const createEdge = (source: string, target: string, label: string): Edge => ({
  id: `e_${source}_${target}`, source, target, label, type: 'retro', markerEnd: { type: MarkerType.ArrowClosed }
});

export const TEMPLATES: FSMTemplate[] = [
  // --- BASIC TEMPLATES ---
  {
    id: 'blank',
    name: 'Blank Canvas',
    category: 'BASIC',
    description: 'Start from scratch with an empty workspace.',
    nodes: [],
    edges: []
  },

  // --- MOTOR CONTROL ---
  {
    id: 'bldc_sensorless',
    name: 'BLDC Sensorless 6-Step',
    category: 'MOTOR_CONTROL',
    description: 'Trapezoidal control with Back-EMF zero-crossing detection.',
    nodes: [
      createNode('align', 'input', 'ALIGN_ROTOR', 100, 200, 'HAL.setPWM(1, 10); dispatch("ALIGNED", 500);'),
      createNode('ramp', 'process', 'OPEN_LOOP_RAMP', 300, 200, 'ctx.speed += 10; dispatch(ctx.speed > 100 ? "BEMF_VALID" : "RAMPING", 100);'),
      createNode('run', 'process', 'CLOSED_LOOP', 500, 200, 'ctx.zcd = HAL.getADC(0); dispatch("ZCD_DETECT", 1);'),
      createNode('com', 'process', 'COMMUTATE', 500, 400, 'ctx.step = (ctx.step + 1) % 6; dispatch("NEXT", 0);'),
      createNode('stall', 'error', 'STALL_DETECT', 300, 400, 'HAL.setPWM(1, 0); dispatch("RETRY", 2000);')
    ],
    edges: [
      createEdge('align', 'ramp', 'ALIGNED'),
      createEdge('ramp', 'ramp', 'RAMPING'),
      createEdge('ramp', 'run', 'BEMF_VALID'),
      createEdge('run', 'com', 'ZCD_DETECT'),
      createEdge('com', 'run', 'NEXT'),
      createEdge('run', 'stall', 'TIMEOUT'),
      createEdge('stall', 'align', 'RETRY')
    ]
  },
  {
    id: 'stepper_profile',
    name: 'Stepper Motion Profile',
    category: 'MOTOR_CONTROL',
    description: 'S-Curve acceleration and deceleration.',
    nodes: [
      createNode('idle', 'input', 'IDLE', 100, 100, 'dispatch("MOVE_CMD", 0);'),
      createNode('accel', 'process', 'ACCEL', 300, 100, 'ctx.step_delay *= 0.95; dispatch(ctx.speed >= ctx.target ? "CRUISE" : "ACC", 10);'),
      createNode('cruise', 'process', 'CONST_SPEED', 500, 100, 'dispatch(ctx.steps_left < ctx.decel_steps ? "DECEL" : "RUN", 10);'),
      createNode('decel', 'process', 'DECEL', 500, 300, 'ctx.step_delay *= 1.05; dispatch(ctx.speed < 10 ? "STOP" : "DEC", 10);'),
      createNode('stop', 'output', 'TARGET_REACHED', 300, 300, 'dispatch("DONE", 500);')
    ],
    edges: [
      createEdge('idle', 'accel', 'MOVE_CMD'),
      createEdge('accel', 'accel', 'ACC'),
      createEdge('accel', 'cruise', 'CRUISE'),
      createEdge('cruise', 'cruise', 'RUN'),
      createEdge('cruise', 'decel', 'DECEL'),
      createEdge('decel', 'decel', 'DEC'),
      createEdge('decel', 'stop', 'STOP'),
      createEdge('stop', 'idle', 'DONE')
    ]
  },
  {
    id: 'foc_svm',
    name: 'FOC Space Vector Mod',
    category: 'MOTOR_CONTROL',
    description: 'Field Oriented Control loop implementation.',
    nodes: [
      createNode('adc', 'input', 'READ_CURRENT', 100, 200, 'ctx.Ia = HAL.getADC(1); ctx.Ib = HAL.getADC(2); dispatch("CLARKE", 0);'),
      createNode('clarke', 'process', 'CLARKE_TRANS', 300, 200, 'ctx.Ialpha = ctx.Ia; dispatch("PARK", 0);'),
      createNode('park', 'process', 'PARK_TRANS', 500, 200, 'ctx.Id = 0; ctx.Iq = 0; dispatch("PI_LOOP", 0);'),
      createNode('pi', 'process', 'PI_CONTROLLER', 700, 200, 'ctx.Vd = 0; ctx.Vq = 100; dispatch("INV_PARK", 0);'),
      createNode('pwm', 'process', 'SVM_UPDATE', 500, 400, 'HAL.setPWM(1, ctx.Ta); HAL.setPWM(2, ctx.Tb); dispatch("ADC_TRIG", 1);')
    ],
    edges: [
      createEdge('adc', 'clarke', 'CLARKE'),
      createEdge('clarke', 'park', 'PARK'),
      createEdge('park', 'pi', 'PI_LOOP'),
      createEdge('pi', 'pwm', 'INV_PARK'),
      createEdge('pwm', 'adc', 'ADC_TRIG')
    ]
  },
  {
    id: 'servo_pid',
    name: 'Servo Positioner',
    category: 'MOTOR_CONTROL',
    description: 'DC Motor position control with encoder feedback.',
    nodes: [
      createNode('wait', 'input', 'WAIT_CMD', 100, 100, 'dispatch("NEW_POS", 0);'),
      createNode('read', 'process', 'READ_ENCODER', 300, 100, 'ctx.pos = HAL.getADC(0); dispatch("CALC", 0);'),
      createNode('pid', 'process', 'PID_CALC', 500, 100, 'ctx.err = ctx.target - ctx.pos; ctx.out = ctx.kp*ctx.err; dispatch("DRIVE", 0);'),
      createNode('drive', 'hardware', 'H_BRIDGE', 500, 300, 'HAL.setPWM(1, Math.abs(ctx.out)); dispatch("LOOP", 10);'),
      createNode('dead', 'process', 'IN_POSITION', 300, 300, 'HAL.setPWM(1, 0); dispatch("WAIT", 0);')
    ],
    edges: [
      createEdge('wait', 'read', 'NEW_POS'),
      createEdge('read', 'pid', 'CALC'),
      createEdge('pid', 'drive', 'DRIVE'),
      createEdge('drive', 'read', 'LOOP'),
      createEdge('drive', 'dead', 'TARGET_HIT'),
      createEdge('dead', 'wait', 'WAIT')
    ]
  },

  // --- COMMUNICATION ---
  {
    id: 'i2c_master',
    name: 'I2C Master Driver',
    category: 'COMMUNICATION',
    description: 'Bit-banged or peripheral I2C transaction.',
    nodes: [
      createNode('idle', 'input', 'IDLE', 100, 100, 'dispatch("TX_REQ", 0);'),
      createNode('start', 'hardware', 'START_COND', 300, 100, 'HAL.writePin(1, 0); dispatch("ADDR", 10);'),
      createNode('addr', 'process', 'SEND_ADDR', 500, 100, 'dispatch("ACK_WAIT", 10);'),
      createNode('ack', 'process', 'CHECK_ACK', 500, 300, 'ctx.ack = !HAL.readPin(1); dispatch(ctx.ack ? "DATA" : "ERR", 0);'),
      createNode('data', 'process', 'TX_DATA', 300, 300, 'dispatch("STOP", 10);'),
      createNode('stop', 'hardware', 'STOP_COND', 100, 300, 'HAL.writePin(1, 1); dispatch("DONE", 0);')
    ],
    edges: [
      createEdge('idle', 'start', 'TX_REQ'),
      createEdge('start', 'addr', 'ADDR'),
      createEdge('addr', 'ack', 'ACK_WAIT'),
      createEdge('ack', 'data', 'DATA'),
      createEdge('ack', 'stop', 'ERR'),
      createEdge('data', 'stop', 'STOP'),
      createEdge('stop', 'idle', 'DONE')
    ]
  },
  {
    id: 'spi_flash',
    name: 'SPI Flash Manager',
    category: 'COMMUNICATION',
    description: 'W25Qxx Flash Erase/Write cycle.',
    nodes: [
      createNode('idle', 'input', 'READY', 100, 100, 'dispatch("WRITE_REQ", 0);'),
      createNode('wren', 'process', 'SEND_WREN', 300, 100, 'HAL.writePin(CS, 0); dispatch("CMD_06", 1);'),
      createNode('erase', 'process', 'SECTOR_ERASE', 500, 100, 'dispatch("WAIT_BUSY", 100);'),
      createNode('busy', 'process', 'POLL_STATUS', 500, 300, 'ctx.sr = HAL.readPin(MISO); dispatch(ctx.sr&1 ? "STILL_BUSY" : "DONE", 10);'),
      createNode('write', 'process', 'PAGE_PROG', 300, 300, 'dispatch("FINISH", 50);')
    ],
    edges: [
      createEdge('idle', 'wren', 'WRITE_REQ'),
      createEdge('wren', 'erase', 'CMD_06'),
      createEdge('erase', 'busy', 'WAIT_BUSY'),
      createEdge('busy', 'busy', 'STILL_BUSY'),
      createEdge('busy', 'write', 'DONE'),
      createEdge('write', 'idle', 'FINISH')
    ]
  },
  {
    id: 'uart_parser',
    name: 'UART Command Parser',
    category: 'COMMUNICATION',
    description: 'ASCII Command Interface (CLI).',
    nodes: [
      createNode('wait', 'input', 'WAIT_RX', 100, 100, 'ctx.char = HAL.UART_Receive(); dispatch(ctx.char ? "RX" : "WAIT", 10);'),
      createNode('buffer', 'process', 'BUFFER_CHAR', 300, 100, 'ctx.buf += ctx.char; dispatch(ctx.char == 13 ? "EOL" : "MORE", 0);'),
      createNode('parse', 'process', 'PARSE_CMD', 500, 100, 'ctx.cmd = ctx.buf.trim(); dispatch("EXEC", 0);'),
      createNode('exec', 'process', 'EXECUTE', 500, 300, 'console.log("CMD:", ctx.cmd); dispatch("RESP", 10);'),
      createNode('resp', 'uart', 'SEND_OK', 300, 300, 'HAL.UART_Transmit("OK\\n"); dispatch("RESET", 0);')
    ],
    edges: [
      createEdge('wait', 'wait', 'WAIT'),
      createEdge('wait', 'buffer', 'RX'),
      createEdge('buffer', 'wait', 'MORE'),
      createEdge('buffer', 'parse', 'EOL'),
      createEdge('parse', 'exec', 'EXEC'),
      createEdge('exec', 'resp', 'RESP'),
      createEdge('resp', 'wait', 'RESET')
    ]
  },
  {
    id: 'ble_adv',
    name: 'BLE Advertising',
    category: 'COMMUNICATION',
    description: 'Bluetooth Low Energy Peripheral state.',
    nodes: [
      createNode('init', 'input', 'STACK_INIT', 100, 100, 'dispatch("READY", 100);'),
      createNode('config', 'process', 'SET_PARAMS', 300, 100, 'ctx.name = "NeuroDevice"; dispatch("START_ADV", 50);'),
      createNode('adv', 'process', 'ADVERTISING', 500, 100, 'HAL.writePin(LED, 1); dispatch("CONN_REQ", 2000);'),
      createNode('conn', 'process', 'CONNECTED', 500, 300, 'HAL.writePin(LED, 0); dispatch("DISCONNECT", 5000);'),
      createNode('sleep', 'process', 'DEEP_SLEEP', 300, 300, 'dispatch("WAKE", 5000);')
    ],
    edges: [
      createEdge('init', 'config', 'READY'),
      createEdge('config', 'adv', 'START_ADV'),
      createEdge('adv', 'conn', 'CONN_REQ'),
      createEdge('adv', 'sleep', 'TIMEOUT'),
      createEdge('conn', 'adv', 'DISCONNECT'),
      createEdge('sleep', 'adv', 'WAKE')
    ]
  },
  {
    id: 'eth_dhcp',
    name: 'Ethernet DHCP Client',
    category: 'COMMUNICATION',
    description: 'Network Link and IP Acquisition.',
    nodes: [
      createNode('down', 'input', 'LINK_DOWN', 100, 100, 'ctx.link = HAL.readPin(PHY_INT); dispatch(ctx.link ? "UP" : "WAIT", 500);'),
      createNode('disc', 'process', 'DHCP_DISCOVER', 300, 100, 'dispatch("OFFER", 1000);'),
      createNode('req', 'process', 'DHCP_REQUEST', 500, 100, 'dispatch("ACK", 200);'),
      createNode('bound', 'output', 'IP_BOUND', 500, 300, 'ctx.ip = "192.168.1.100"; dispatch("RENEW", 5000);'),
      createNode('renew', 'process', 'RENEWING', 300, 300, 'dispatch("ACK", 200);')
    ],
    edges: [
      createEdge('down', 'down', 'WAIT'),
      createEdge('down', 'disc', 'UP'),
      createEdge('disc', 'req', 'OFFER'),
      createEdge('req', 'bound', 'ACK'),
      createEdge('bound', 'renew', 'RENEW'),
      createEdge('renew', 'bound', 'ACK')
    ]
  },
  {
    id: 'modbus_slave',
    name: 'Modbus RTU Slave',
    category: 'COMMUNICATION',
    description: 'Industrial serial protocol handling.',
    nodes: [
      createNode('rx', 'input', 'RECEIVING', 100, 200, 'dispatch("FRAME_END", 20);'),
      createNode('crc', 'process', 'CHECK_CRC', 300, 200, 'dispatch(ctx.crc_ok ? "PARSE" : "SILENT", 0);'),
      createNode('parse', 'process', 'FUNC_CODE', 500, 200, 'dispatch("READ_REGS", 0);'),
      createNode('resp', 'uart', 'TX_RESPONSE', 700, 200, 'HAL.UART_Transmit(ctx.data); dispatch("DONE", 10);'),
      createNode('err', 'uart', 'TX_EXCEPTION', 500, 400, 'dispatch("DONE", 10);')
    ],
    edges: [
      createEdge('rx', 'crc', 'FRAME_END'),
      createEdge('crc', 'parse', 'PARSE'),
      createEdge('crc', 'rx', 'SILENT'),
      createEdge('parse', 'resp', 'READ_REGS'),
      createEdge('parse', 'err', 'INVALID_ADDR'),
      createEdge('resp', 'rx', 'DONE'),
      createEdge('err', 'rx', 'DONE')
    ]
  },
  {
    id: 'lin_master',
    name: 'LIN Bus Master',
    category: 'COMMUNICATION',
    description: 'Local Interconnect Network schedule.',
    nodes: [
      createNode('break', 'uart', 'SYNC_BREAK', 100, 100, 'HAL.UART_Break(); dispatch("SYNC", 1);'),
      createNode('sync', 'uart', 'SYNC_FIELD', 300, 100, 'HAL.UART_Transmit(0x55); dispatch("ID", 1);'),
      createNode('id', 'uart', 'PROTECTED_ID', 500, 100, 'HAL.UART_Transmit(0x3C); dispatch("DATA", 1);'),
      createNode('data', 'process', 'RX_RESPONSE', 500, 300, 'dispatch("NEXT_SLOT", 10);'),
      createNode('sleep', 'process', 'BUS_SLEEP', 300, 300, 'dispatch("WAKE", 1000);')
    ],
    edges: [
      createEdge('break', 'sync', 'SYNC'),
      createEdge('sync', 'id', 'ID'),
      createEdge('id', 'data', 'DATA'),
      createEdge('data', 'break', 'NEXT_SLOT'),
      createEdge('data', 'sleep', 'IDLE_TIMEOUT'),
      createEdge('sleep', 'break', 'WAKE')
    ]
  },
  {
    id: 'usb_hid',
    name: 'USB HID Mouse',
    category: 'COMMUNICATION',
    description: 'Human Interface Device report loop.',
    nodes: [
      createNode('idle', 'input', 'IDLE', 100, 200, 'dispatch("POLL", 10);'),
      createNode('scan', 'process', 'SCAN_BUTTONS', 300, 200, 'ctx.click = HAL.readPin(BTN); dispatch("MOVE", 0);'),
      createNode('calc', 'process', 'CALC_DELTA', 500, 200, 'ctx.dx = 5; dispatch("BUILD", 0);'),
      createNode('send', 'process', 'USB_IN_EP', 700, 200, 'dispatch("ACK", 1);')
    ],
    edges: [
      createEdge('idle', 'scan', 'POLL'),
      createEdge('scan', 'calc', 'MOVE'),
      createEdge('calc', 'send', 'BUILD'),
      createEdge('send', 'idle', 'ACK')
    ]
  },

  // --- POWER MANAGEMENT ---
  {
    id: 'li_ion_charge',
    name: 'Li-Ion Charger',
    category: 'POWER_MGMT',
    description: 'Standard CC-CV charging profile.',
    nodes: [
      createNode('check', 'input', 'BAT_CHECK', 100, 200, 'ctx.v = HAL.getADC(0); dispatch(ctx.v < 3.0 ? "PRE" : "CC", 100);'),
      createNode('pre', 'process', 'PRE_CHARGE', 300, 100, 'HAL.setPWM(1, 10); dispatch("V_OK", 1000);'),
      createNode('cc', 'process', 'CONST_CURRENT', 300, 300, 'HAL.setPWM(1, 100); dispatch(ctx.v > 4.1 ? "CV" : "WAIT", 100);'),
      createNode('cv', 'process', 'CONST_VOLT', 500, 300, 'ctx.i = HAL.getADC(1); dispatch(ctx.i < 50 ? "DONE" : "WAIT", 100);'),
      createNode('done', 'output', 'FULL', 700, 200, 'HAL.setPWM(1, 0); dispatch("MONITOR", 5000);')
    ],
    edges: [
      createEdge('check', 'pre', 'PRE'),
      createEdge('check', 'cc', 'CC'),
      createEdge('pre', 'cc', 'V_OK'),
      createEdge('cc', 'cv', 'CV'),
      createEdge('cc', 'cc', 'WAIT'),
      createEdge('cv', 'done', 'DONE'),
      createEdge('cv', 'cv', 'WAIT'),
      createEdge('done', 'check', 'MONITOR')
    ]
  },
  {
    id: 'mppt_solar',
    name: 'Solar MPPT',
    category: 'POWER_MGMT',
    description: 'Maximum Power Point Tracking (P&O).',
    nodes: [
      createNode('meas', 'input', 'MEASURE', 100, 200, 'ctx.p = HAL.getADC(V) * HAL.getADC(I); dispatch("COMPARE", 50);'),
      createNode('pert', 'process', 'PERTURB', 300, 200, 'ctx.pwm += ctx.dir; HAL.setPWM(1, ctx.pwm); dispatch("WAIT", 100);'),
      createNode('obs', 'process', 'OBSERVE', 500, 200, 'ctx.p_new = HAL.getADC(V) * HAL.getADC(I); dispatch(ctx.p_new > ctx.p ? "SAME" : "FLIP", 0);'),
      createNode('flip', 'process', 'FLIP_DIR', 300, 400, 'ctx.dir *= -1; dispatch("NEXT", 0);')
    ],
    edges: [
      createEdge('meas', 'pert', 'COMPARE'),
      createEdge('pert', 'obs', 'WAIT'),
      createEdge('obs', 'meas', 'SAME'),
      createEdge('obs', 'flip', 'FLIP'),
      createEdge('flip', 'meas', 'NEXT')
    ]
  },
  {
    id: 'sleep_mgr',
    name: 'Sleep Manager',
    category: 'POWER_MGMT',
    description: 'System low power state machine.',
    nodes: [
      createNode('run', 'input', 'RUN_MODE', 100, 200, 'dispatch("IDLE_TIMEOUT", 1000);'),
      createNode('idle', 'process', 'SLEEP_WFI', 300, 200, 'HAL.enterSleepMode("WFI"); dispatch("DEEP_REQ", 2000);'),
      createNode('deep', 'process', 'STOP_MODE', 500, 200, 'HAL.enterSleepMode("STOP"); dispatch("WAKE_IRQ", 5000);'),
      createNode('wake', 'process', 'RESTORE', 300, 400, 'HAL.Init(); dispatch("READY", 100);')
    ],
    edges: [
      createEdge('run', 'idle', 'IDLE_TIMEOUT'),
      createEdge('idle', 'run', 'IRQ'),
      createEdge('idle', 'deep', 'DEEP_REQ'),
      createEdge('deep', 'wake', 'WAKE_IRQ'),
      createEdge('wake', 'run', 'READY')
    ]
  },
  {
    id: 'buck_conv',
    name: 'Digital Buck Converter',
    category: 'POWER_MGMT',
    description: 'Soft-start and regulation loop.',
    nodes: [
      createNode('init', 'input', 'INIT', 100, 200, 'ctx.duty = 0; dispatch("SOFT_START", 10);'),
      createNode('ramp', 'process', 'SOFT_START', 300, 200, 'ctx.duty++; HAL.setPWM(1, ctx.duty); dispatch(ctx.duty > 20 ? "REGULATE" : "RAMP", 10);'),
      createNode('reg', 'process', 'REGULATE', 500, 200, 'ctx.vout = HAL.getADC(0); dispatch(ctx.vout > 3000 ? "OVP" : "LOOP", 5);'),
      createNode('ovp', 'error', 'OVER_VOLTAGE', 500, 400, 'HAL.setPWM(1, 0); dispatch("RESET", 1000);')
    ],
    edges: [
      createEdge('init', 'ramp', 'SOFT_START'),
      createEdge('ramp', 'ramp', 'RAMP'),
      createEdge('ramp', 'reg', 'REGULATE'),
      createEdge('reg', 'reg', 'LOOP'),
      createEdge('reg', 'ovp', 'OVP'),
      createEdge('ovp', 'init', 'RESET')
    ]
  },

  // --- IOT ---
  {
    id: 'mqtt_client',
    name: 'MQTT Client',
    category: 'IOT',
    description: 'Publish/Subscribe cycle.',
    nodes: [
      createNode('tcp', 'input', 'TCP_CONNECT', 100, 200, 'dispatch("CONNECTED", 500);'),
      createNode('conn', 'process', 'MQTT_CONNECT', 300, 200, 'dispatch("CONNACK", 500);'),
      createNode('sub', 'process', 'SUBSCRIBE', 500, 200, 'dispatch("SUBACK", 200);'),
      createNode('loop', 'process', 'MAIN_LOOP', 700, 200, 'dispatch("PUB_REQ", 1000);'),
      createNode('pub', 'process', 'PUBLISH', 700, 400, 'dispatch("DONE", 100);'),
      createNode('ping', 'process', 'PING_REQ', 500, 400, 'dispatch("PONG", 100);')
    ],
    edges: [
      createEdge('tcp', 'conn', 'CONNECTED'),
      createEdge('conn', 'sub', 'CONNACK'),
      createEdge('sub', 'loop', 'SUBACK'),
      createEdge('loop', 'pub', 'PUB_REQ'),
      createEdge('loop', 'ping', 'KEEPALIVE'),
      createEdge('pub', 'loop', 'DONE'),
      createEdge('ping', 'loop', 'PONG')
    ]
  },
  {
    id: 'wifi_sta',
    name: 'WiFi Station',
    category: 'IOT',
    description: 'Connection manager.',
    nodes: [
      createNode('scan', 'input', 'SCANNING', 100, 200, 'dispatch("SSID_FOUND", 1000);'),
      createNode('auth', 'process', 'AUTHENTICATING', 300, 200, 'dispatch("ASSOC", 500);'),
      createNode('dhcp', 'process', 'GET_IP', 500, 200, 'dispatch("IP_OK", 1000);'),
      createNode('conn', 'output', 'CONNECTED', 700, 200, 'dispatch("RSSI_LOW", 5000);'),
      createNode('lost', 'error', 'LINK_LOST', 400, 400, 'dispatch("RETRY", 1000);')
    ],
    edges: [
      createEdge('scan', 'auth', 'SSID_FOUND'),
      createEdge('auth', 'dhcp', 'ASSOC'),
      createEdge('dhcp', 'conn', 'IP_OK'),
      createEdge('conn', 'lost', 'RSSI_LOW'),
      createEdge('lost', 'scan', 'RETRY')
    ]
  },
  {
    id: 'ota_flow',
    name: 'OTA Updater',
    category: 'IOT',
    description: 'Over-The-Air firmware update.',
    nodes: [
      createNode('check', 'input', 'CHECK_VER', 100, 200, 'dispatch("NEW_FW", 500);'),
      createNode('dl', 'process', 'DOWNLOAD', 300, 200, 'ctx.pct += 10; dispatch(ctx.pct<100?"MORE":"DONE", 200);'),
      createNode('crc', 'process', 'VERIFY_CRC', 500, 200, 'dispatch("VALID", 100);'),
      createNode('flash', 'process', 'WRITE_FLASH', 700, 200, 'dispatch("REBOOT", 500);'),
      createNode('boot', 'output', 'REBOOTING', 900, 200, 'NVIC_SystemReset();')
    ],
    edges: [
      createEdge('check', 'dl', 'NEW_FW'),
      createEdge('dl', 'dl', 'MORE'),
      createEdge('dl', 'crc', 'DONE'),
      createEdge('crc', 'flash', 'VALID'),
      createEdge('flash', 'boot', 'REBOOT')
    ]
  },
  {
    id: 'sensor_fusion',
    name: 'Sensor Fusion',
    category: 'IOT',
    description: 'AHRS loop reading IMU data.',
    nodes: [
      createNode('accel', 'input', 'READ_ACCEL', 100, 200, 'dispatch("GYRO", 10);'),
      createNode('gyro', 'process', 'READ_GYRO', 300, 200, 'dispatch("MAG", 10);'),
      createNode('mag', 'process', 'READ_MAG', 500, 200, 'dispatch("FUSE", 10);'),
      createNode('kalman', 'process', 'KALMAN_FILTER', 700, 200, 'ctx.pitch = 0.98; dispatch("OUT", 10);'),
      createNode('out', 'process', 'OUTPUT', 500, 400, 'dispatch("LOOP", 10);')
    ],
    edges: [
      createEdge('accel', 'gyro', 'GYRO'),
      createEdge('gyro', 'mag', 'MAG'),
      createEdge('mag', 'kalman', 'FUSE'),
      createEdge('kalman', 'out', 'OUT'),
      createEdge('out', 'accel', 'LOOP')
    ]
  },

  // --- SAFETY ---
  {
    id: 'estop',
    name: 'Emergency Stop',
    category: 'SAFETY',
    description: 'SIL-rated safety interlocking.',
    nodes: [
      createNode('run', 'input', 'NORMAL_RUN', 100, 200, 'ctx.estop = !HAL.readPin(IN); dispatch(ctx.estop?"TRIP":"OK", 100);'),
      createNode('trip', 'error', 'ESTOP_ACTIVE', 300, 200, 'HAL.writePin(RELAY, 0); dispatch("WAIT_RELEASE", 100);'),
      createNode('safe', 'process', 'SAFE_STATE', 500, 200, 'ctx.reset = HAL.readPin(BTN); dispatch(ctx.reset?"RESET":"WAIT", 100);'),
      createNode('check', 'process', 'SELF_TEST', 300, 400, 'dispatch("PASS", 500);')
    ],
    edges: [
      createEdge('run', 'run', 'OK'),
      createEdge('run', 'trip', 'TRIP'),
      createEdge('trip', 'safe', 'WAIT_RELEASE'),
      createEdge('safe', 'check', 'RESET'),
      createEdge('check', 'run', 'PASS')
    ]
  },
  {
    id: 'temp_prot',
    name: 'Thermal Protection',
    category: 'SAFETY',
    description: 'Hysteresis thermal management.',
    nodes: [
      createNode('norm', 'input', 'NORMAL', 100, 200, 'ctx.t = HAL.getADC(TEMP); dispatch(ctx.t>60?"WARN":"OK", 1000);'),
      createNode('warn', 'process', 'WARNING', 300, 200, 'HAL.setPWM(FAN, 50); dispatch(ctx.t>80?"CRIT":"COOL", 1000);'),
      createNode('crit', 'error', 'CRITICAL', 500, 200, 'HAL.setPWM(FAN, 100); dispatch(ctx.t>100?"SHUTDOWN":"COOL", 500);'),
      createNode('off', 'output', 'SHUTDOWN', 700, 200, 'HAL.writePin(PWR, 0);')
    ],
    edges: [
      createEdge('norm', 'norm', 'OK'),
      createEdge('norm', 'warn', 'WARN'),
      createEdge('warn', 'crit', 'CRIT'),
      createEdge('warn', 'norm', 'COOL'),
      createEdge('crit', 'off', 'SHUTDOWN'),
      createEdge('crit', 'warn', 'COOL')
    ]
  },
  {
    id: 'door_lock',
    name: 'Electronic Door Lock',
    category: 'SAFETY',
    description: 'Solenoid control logic.',
    nodes: [
      createNode('locked', 'input', 'LOCKED', 100, 200, 'dispatch("UNLOCK_REQ", 0);'),
      createNode('sol', 'hardware', 'SOLENOID_ON', 300, 200, 'HAL.writePin(SOL, 1); dispatch("TIMEOUT", 3000);'),
      createNode('open', 'process', 'UNLOCKED', 500, 200, 'dispatch("CLOSE_EVT", 0);'),
      createNode('lock', 'hardware', 'LOCKING', 300, 400, 'HAL.writePin(SOL, 0); dispatch("DONE", 500);')
    ],
    edges: [
      createEdge('locked', 'sol', 'UNLOCK_REQ'),
      createEdge('sol', 'open', 'TIMEOUT'),
      createEdge('open', 'lock', 'CLOSE_EVT'),
      createEdge('lock', 'locked', 'DONE')
    ]
  },
  {
    id: 'infusion_pump',
    name: 'Infusion Pump',
    category: 'SAFETY',
    description: 'Medical pump control flow.',
    nodes: [
      createNode('idle', 'input', 'IDLE', 100, 200, 'dispatch("START", 0);'),
      createNode('run', 'process', 'INFUSING', 300, 200, 'dispatch("OCCLUSION", 100);'),
      createNode('alarm', 'error', 'ALARM_HIGH', 300, 400, 'HAL.writePin(BUZZ, 1); dispatch("MUTE", 0);'),
      createNode('stop', 'output', 'STOPPED', 500, 200, 'HAL.setPWM(MOTOR, 0);')
    ],
    edges: [
      createEdge('idle', 'run', 'START'),
      createEdge('run', 'run', 'TICK'),
      createEdge('run', 'alarm', 'OCCLUSION'),
      createEdge('run', 'stop', 'STOP'),
      createEdge('alarm', 'stop', 'MUTE')
    ]
  },

  // --- DRIVER ---
  {
    id: 'keypad_scan',
    name: 'Matrix Keypad',
    category: 'DRIVER',
    description: 'Row/Column scanning loop.',
    nodes: [
      createNode('r1', 'input', 'SCAN_R1', 100, 200, 'HAL.writePin(R1, 1); dispatch("CHK", 5);'),
      createNode('r2', 'process', 'SCAN_R2', 300, 200, 'HAL.writePin(R1, 0); HAL.writePin(R2, 1); dispatch("CHK", 5);'),
      createNode('r3', 'process', 'SCAN_R3', 500, 200, 'HAL.writePin(R2, 0); HAL.writePin(R3, 1); dispatch("CHK", 5);'),
      createNode('deb', 'process', 'DEBOUNCE', 300, 400, 'dispatch("VALID", 20);')
    ],
    edges: [
      createEdge('r1', 'r2', 'CHK'),
      createEdge('r2', 'r3', 'CHK'),
      createEdge('r3', 'r1', 'CHK'),
      createEdge('r1', 'deb', 'KEY_DOWN')
    ]
  },
  {
    id: 'rotary_enc',
    name: 'Rotary Encoder',
    category: 'DRIVER',
    description: 'Quadrature decoding state machine.',
    nodes: [
      createNode('00', 'input', '00', 200, 200, 'dispatch("CHG", 0);'),
      createNode('01', 'process', '01', 200, 100, 'ctx.cnt++; dispatch("CHG", 0);'),
      createNode('10', 'process', '10', 400, 200, 'ctx.cnt--; dispatch("CHG", 0);'),
      createNode('11', 'process', '11', 200, 300, 'dispatch("CHG", 0);')
    ],
    edges: [
      createEdge('00', '01', 'A_RISE'),
      createEdge('01', '11', 'B_RISE'),
      createEdge('11', '10', 'A_FALL'),
      createEdge('10', '00', 'B_FALL')
    ]
  },
  {
    id: 'sd_logger',
    name: 'SD Card Logger',
    category: 'DRIVER',
    description: 'FatFS file operations.',
    nodes: [
      createNode('mount', 'input', 'MOUNT_FS', 100, 200, 'dispatch("OPEN", 100);'),
      createNode('open', 'process', 'F_OPEN', 300, 200, 'dispatch("WRITE", 10);'),
      createNode('write', 'process', 'F_WRITE', 500, 200, 'ctx.buf_len++; dispatch(ctx.buf_len>512?"SYNC":"MORE", 10);'),
      createNode('sync', 'process', 'F_SYNC', 500, 400, 'ctx.buf_len=0; dispatch("WRITE", 10);')
    ],
    edges: [
      createEdge('mount', 'open', 'OPEN'),
      createEdge('open', 'write', 'WRITE'),
      createEdge('write', 'write', 'MORE'),
      createEdge('write', 'sync', 'SYNC'),
      createEdge('sync', 'write', 'WRITE')
    ]
  },
  {
    id: 'ws2812',
    name: 'WS2812 LED Strip',
    category: 'DRIVER',
    description: 'Addressable LED timing.',
    nodes: [
      createNode('reset', 'input', 'RESET_50US', 100, 200, 'HAL.writePin(DAT, 0); dispatch("SEND", 50);'),
      createNode('bit', 'process', 'SEND_BIT', 300, 200, 'dispatch("NEXT", 1);'),
      createNode('latch', 'process', 'LATCH', 500, 200, 'dispatch("RESET", 10);')
    ],
    edges: [
      createEdge('reset', 'bit', 'SEND'),
      createEdge('bit', 'bit', 'NEXT'),
      createEdge('bit', 'latch', 'DONE'),
      createEdge('latch', 'reset', 'RESET')
    ]
  },
  {
    id: 'ir_decode',
    name: 'IR Remote Decoder',
    category: 'DRIVER',
    description: 'NEC Protocol receiver.',
    nodes: [
      createNode('idle', 'input', 'IDLE', 100, 200, 'dispatch("LEADER", 0);'),
      createNode('leader', 'process', 'LEADER_9MS', 300, 200, 'dispatch("SPACE", 9);'),
      createNode('space', 'process', 'SPACE_4MS', 500, 200, 'dispatch("DATA", 4);'),
      createNode('data', 'process', 'READ_BITS', 700, 200, 'dispatch("VALID", 30);')
    ],
    edges: [
      createEdge('idle', 'leader', 'EDGE_FALL'),
      createEdge('leader', 'space', 'EDGE_RISE'),
      createEdge('space', 'data', 'EDGE_FALL'),
      createEdge('data', 'idle', 'VALID')
    ]
  },

  // --- SYSTEM ---
  {
    id: 'scheduler',
    name: 'Cooperative Scheduler',
    category: 'SYSTEM',
    description: 'Simple round-robin task switcher.',
    nodes: [
      createNode('t1', 'input', 'TASK_1', 200, 100, 'dispatch("T2", 10);'),
      createNode('t2', 'process', 'TASK_2', 400, 100, 'dispatch("T3", 10);'),
      createNode('t3', 'process', 'TASK_3', 300, 300, 'dispatch("IDLE", 10);'),
      createNode('idle', 'process', 'IDLE_WAIT', 100, 300, 'dispatch("T1", 5);')
    ],
    edges: [
      createEdge('t1', 't2', 'T2'),
      createEdge('t2', 't3', 'T3'),
      createEdge('t3', 'idle', 'IDLE'),
      createEdge('idle', 't1', 'T1')
    ]
  },
  {
    id: 'bist',
    name: 'Built-In Self Test',
    category: 'SYSTEM',
    description: 'Power-on hardware verification.',
    nodes: [
      createNode('ram', 'input', 'RAM_TEST', 100, 200, 'dispatch("FLASH", 100);'),
      createNode('flash', 'process', 'FLASH_CRC', 300, 200, 'dispatch("PERIPH", 200);'),
      createNode('periph', 'process', 'CHECK_IO', 500, 200, 'dispatch("OK", 100);'),
      createNode('boot', 'output', 'BOOT_OS', 700, 200, 'console.log("System OK");'),
      createNode('fail', 'error', 'SYS_HALT', 400, 400, 'HAL.writePin(LED_ERR, 1);')
    ],
    edges: [
      createEdge('ram', 'flash', 'FLASH'),
      createEdge('flash', 'periph', 'PERIPH'),
      createEdge('periph', 'boot', 'OK'),
      createEdge('periph', 'fail', 'ERR')
    ]
  },
  {
    id: 'factory_rst',
    name: 'Factory Reset',
    category: 'SYSTEM',
    description: 'Long press handling to wipe config.',
    nodes: [
      createNode('mon', 'input', 'MONITOR_BTN', 100, 200, 'dispatch(HAL.readPin(0)?"HOLD":"IDLE", 100);'),
      createNode('count', 'process', 'COUNT_5S', 300, 200, 'ctx.cnt++; dispatch(ctx.cnt>50?"WIPE":"MON", 100);'),
      createNode('wipe', 'process', 'ERASING', 500, 200, 'Flash_Erase(); dispatch("DONE", 2000);'),
      createNode('reboot', 'output', 'REBOOT', 700, 200, 'NVIC_Reset();')
    ],
    edges: [
      createEdge('mon', 'count', 'HOLD'),
      createEdge('count', 'mon', 'MON'),
      createEdge('count', 'wipe', 'WIPE'),
      createEdge('wipe', 'reboot', 'DONE')
    ]
  },
  {
    id: 'audio_player',
    name: 'Audio Player',
    category: 'SYSTEM',
    description: 'DMA Buffer management.',
    nodes: [
      createNode('idle', 'input', 'IDLE', 100, 200, 'dispatch("PLAY", 0);'),
      createNode('fill', 'process', 'FILL_BUFFER', 300, 200, 'dispatch("START_DMA", 10);'),
      createNode('play', 'process', 'PLAYING', 500, 200, 'dispatch("HALF_CPLT", 100);'),
      createNode('refill', 'process', 'REFILL_HALF', 500, 400, 'dispatch("RESUME", 10);')
    ],
    edges: [
      createEdge('idle', 'fill', 'PLAY'),
      createEdge('fill', 'play', 'START_DMA'),
      createEdge('play', 'refill', 'HALF_CPLT'),
      createEdge('refill', 'play', 'RESUME')
    ]
  },
  {
    id: 'game_loop',
    name: 'Game Loop',
    category: 'SYSTEM',
    description: 'Classic Input-Update-Render loop.',
    nodes: [
      createNode('input', 'input', 'POLL_INPUT', 200, 100, 'dispatch("PHYSICS", 0);'),
      createNode('phys', 'process', 'UPDATE_PHYS', 400, 100, 'dispatch("RENDER", 0);'),
      createNode('draw', 'process', 'RENDER_FRAME', 300, 300, 'dispatch("VSYNC", 0);'),
      createNode('vsync', 'process', 'WAIT_VSYNC', 100, 300, 'dispatch("NEXT", 16);')
    ],
    edges: [
      createEdge('input', 'phys', 'PHYSICS'),
      createEdge('phys', 'draw', 'RENDER'),
      createEdge('draw', 'vsync', 'VSYNC'),
      createEdge('vsync', 'input', 'NEXT')
    ]
  },

  // --- DSP ---
  {
    id: 'fir_filter',
    name: 'FIR Filter',
    category: 'DSP',
    description: 'Finite Impulse Response implementation.',
    nodes: [
      createNode('in', 'input', 'SAMPLE_IN', 100, 200, 'ctx.x0 = HAL.getADC(0); dispatch("SHIFT", 0);'),
      createNode('shift', 'process', 'SHIFT_BUF', 300, 200, 'ctx.x1=ctx.x0; dispatch("MAC", 0);'),
      createNode('mac', 'process', 'MAC_OP', 500, 200, 'ctx.y = c0*x0 + c1*x1; dispatch("OUT", 0);'),
      createNode('out', 'output', 'DAC_OUT', 700, 200, 'HAL.setDAC(ctx.y); dispatch("NEXT", 1);')
    ],
    edges: [
      createEdge('in', 'shift', 'SHIFT'),
      createEdge('shift', 'mac', 'MAC'),
      createEdge('mac', 'out', 'OUT'),
      createEdge('out', 'in', 'NEXT')
    ]
  },
  {
    id: 'fft_proc',
    name: 'FFT Processor',
    category: 'DSP',
    description: 'Fast Fourier Transform stages.',
    nodes: [
      createNode('sample', 'input', 'FILL_BUF', 100, 200, 'dispatch(ctx.i==1024?"PROCESS":"MORE", 0);'),
      createNode('window', 'process', 'WINDOWING', 300, 200, 'ApplyHamming(); dispatch("BITREV", 0);'),
      createNode('bitrev', 'process', 'BIT_REVERSE', 500, 200, 'dispatch("BUTTERFLY", 0);'),
      createNode('fly', 'process', 'BUTTERFLY', 700, 200, 'dispatch("MAG", 0);'),
      createNode('mag', 'output', 'MAGNITUDE', 500, 400, 'dispatch("DONE", 0);')
    ],
    edges: [
      createEdge('sample', 'sample', 'MORE'),
      createEdge('sample', 'window', 'PROCESS'),
      createEdge('window', 'bitrev', 'BITREV'),
      createEdge('bitrev', 'fly', 'BUTTERFLY'),
      createEdge('fly', 'mag', 'MAG'),
      createEdge('mag', 'sample', 'DONE')
    ]
  },
  {
    id: 'audio_fx',
    name: 'Audio Effects Chain',
    category: 'DSP',
    description: 'Simple reverb/delay pipeline.',
    nodes: [
      createNode('in', 'input', 'I2S_RX', 100, 200, 'dispatch("EQ", 0);'),
      createNode('eq', 'process', '3BAND_EQ', 300, 200, 'dispatch("REVERB", 0);'),
      createNode('rev', 'process', 'REVERB', 500, 200, 'dispatch("LIMIT", 0);'),
      createNode('lim', 'process', 'LIMITER', 700, 200, 'dispatch("OUT", 0);'),
      createNode('out', 'output', 'I2S_TX', 900, 200, 'dispatch("NEXT", 0);')
    ],
    edges: [
      createEdge('in', 'eq', 'EQ'),
      createEdge('eq', 'rev', 'REVERB'),
      createEdge('rev', 'lim', 'LIMIT'),
      createEdge('lim', 'out', 'OUT'),
      createEdge('out', 'in', 'NEXT')
    ]
  },

  // --- EXISTING (BOOTLOADER) ---
  {
    id: 'usb_enum',
    name: 'USB Device Enumeration',
    category: 'COMMUNICATION',
    description: 'Chapter 9 USB Device State Machine (Attached -> Configured).',
    nodes: [
      createNode('detach', 'input', 'DETACHED', 100, 100, 'ctx.usb_addr = 0; ctx.configured = false; dispatch("VBUS_DETECT", 1500);'),
      createNode('att', 'process', 'ATTACHED', 300, 100, 'ctx.vbus_present = true; dispatch("USB_RESET", 1000);'),
      createNode('pow', 'process', 'POWERED', 500, 100, 'ctx.current_lim = 100; dispatch("USB_RESET", 1000);'),
      createNode('def', 'process', 'DEFAULT', 500, 300, 'ctx.usb_addr = 0; dispatch("SET_ADDRESS", 1000);'),
      createNode('addr', 'process', 'ADDRESS', 300, 300, 'ctx.usb_addr = 5; dispatch("SET_CONFIG", 1000);'),
      createNode('conf', 'output', 'CONFIGURED', 100, 300, 'ctx.configured = true; ctx.current_lim = 500; dispatch("IDLE_3MS", 2000);'),
      createNode('susp', 'process', 'SUSPENDED', 300, 500, 'ctx.low_power = true; dispatch("RESUME_SIG", 3000);')
    ],
    edges: [
      createEdge('detach', 'att', 'VBUS_DETECT'),
      createEdge('att', 'pow', 'USB_RESET'),
      createEdge('pow', 'def', 'USB_RESET'),
      createEdge('def', 'addr', 'SET_ADDRESS'),
      createEdge('addr', 'def', 'USB_RESET'),
      createEdge('addr', 'conf', 'SET_CONFIG'),
      createEdge('conf', 'addr', 'SET_ADDRESS'),
      createEdge('conf', 'def', 'USB_RESET'),
      createEdge('conf', 'susp', 'IDLE_3MS'),
      createEdge('susp', 'conf', 'RESUME_SIG')
    ]
  },
  {
    id: 'can_err',
    name: 'CAN Bus Error Handler',
    category: 'COMMUNICATION',
    description: 'CAN 2.0B Error Confinement State Machine.',
    nodes: [
      createNode('active', 'input', 'ERROR_ACTIVE', 250, 100, 'ctx.tec = 0; ctx.rec = 0; ctx.mode = "ACTIVE"; dispatch("TEC_GT_127", 2000);'),
      createNode('passive', 'process', 'ERROR_PASSIVE', 250, 300, 'ctx.mode = "PASSIVE"; console.warn("CAN Passive"); dispatch("TEC_GT_255", 2000);'),
      createNode('busoff', 'error', 'BUS_OFF', 250, 500, 'ctx.mode = "BUS_OFF"; ctx.tx_enable = false; dispatch("USR_RESET_REQ", 3000);'),
      createNode('reset', 'process', 'RESET_WAIT', 500, 400, 'ctx.rec_128_cnt = 0; dispatch("128_IDLE_CYCLES", 1000);')
    ],
    edges: [
      createEdge('active', 'passive', 'TEC_GT_127'),
      createEdge('active', 'passive', 'REC_GT_127'),
      createEdge('passive', 'active', 'ERR_CNT_LOW'),
      createEdge('passive', 'busoff', 'TEC_GT_255'),
      createEdge('busoff', 'reset', 'USR_RESET_REQ'),
      createEdge('reset', 'active', '128_IDLE_CYCLES')
    ]
  },
  {
    id: 'secure_boot',
    name: 'Secure Bootloader',
    category: 'BOOTLOADER',
    description: 'Cryptographic signature verification and firmware update logic.',
    nodes: [
      createNode('rst', 'input', 'RESET_VECTOR', 100, 200, 'dispatch("START", 500);'),
      createNode('check', 'hardware', 'CHECK_GPIO', 250, 200, 'ctx.force_update = HAL.readPin(0); dispatch(ctx.force_update ? "BTN_HELD" : "BTN_RELEASED", 500);'),
      createNode('verify', 'process', 'VERIFY_SIG', 450, 200, 'ctx.hash_valid = true; dispatch(ctx.hash_valid ? "SIG_VALID" : "SIG_INVALID", 1500);'),
      createNode('app', 'output', 'JUMP_APP', 650, 200, 'console.log("Booting OS...");'),
      createNode('update', 'process', 'UPDATE_MODE', 250, 400, 'USB_Init(); dispatch("PKT_RX", 1000);'),
      createNode('flash', 'process', 'FLASH_WRITE', 450, 400, 'FLASH_Unlock(); dispatch("PAGE_OK", 500);'),
      createNode('err', 'error', 'WDT_RESET', 450, 50, 'NVIC_SystemReset();')
    ],
    edges: [
      createEdge('rst', 'check', 'START'),
      createEdge('check', 'update', 'BTN_HELD'),
      createEdge('check', 'verify', 'BTN_RELEASED'),
      createEdge('verify', 'app', 'SIG_VALID'),
      createEdge('verify', 'update', 'SIG_INVALID'),
      createEdge('update', 'flash', 'PKT_RX'),
      createEdge('flash', 'update', 'PAGE_OK'),
      createEdge('flash', 'verify', 'EOF'),
      createEdge('verify', 'err', 'HARD_FAULT')
    ]
  },
  {
    id: 'dual_bank',
    name: 'Dual Bank Update',
    category: 'BOOTLOADER',
    description: 'A/B Partition switching.',
    nodes: [
      createNode('check', 'input', 'CHECK_BANK', 100, 200, 'dispatch(ctx.bank=="A"?"BOOT_A":"BOOT_B", 100);'),
      createNode('boota', 'process', 'BOOT_BANK_A', 300, 100, 'dispatch("UPDATE_REQ", 5000);'),
      createNode('dl', 'process', 'DL_TO_B', 500, 100, 'dispatch("VERIFY", 1000);'),
      createNode('swap', 'process', 'SWAP_FLAGS', 700, 200, 'ctx.bank="B"; dispatch("RESET", 100);'),
      createNode('reset', 'output', 'RESET', 500, 400, 'NVIC_Reset();')
    ],
    edges: [
      createEdge('check', 'boota', 'BOOT_A'),
      createEdge('boota', 'dl', 'UPDATE_REQ'),
      createEdge('dl', 'swap', 'VERIFY'),
      createEdge('swap', 'reset', 'RESET')
    ]
  },
  {
    id: 'ymodem',
    name: 'Serial YMODEM',
    category: 'BOOTLOADER',
    description: 'Classic serial file transfer.',
    nodes: [
      createNode('init', 'input', 'SEND_C', 100, 200, 'HAL.UART_Transmit("C"); dispatch("WAIT_SOH", 1000);'),
      createNode('soh', 'process', 'RX_SOH', 300, 200, 'dispatch("RX_1024", 100);'),
      createNode('data', 'process', 'RX_DATA', 500, 200, 'dispatch("CRC", 10);'),
      createNode('ack', 'process', 'SEND_ACK', 700, 200, 'HAL.UART_Transmit(ACK); dispatch("NEXT", 10);'),
      createNode('eot', 'output', 'RX_EOT', 500, 400, 'HAL.UART_Transmit(ACK);')
    ],
    edges: [
      createEdge('init', 'init', 'TIMEOUT'),
      createEdge('init', 'soh', 'RX_SOH'),
      createEdge('soh', 'data', 'PACKET'),
      createEdge('data', 'ack', 'CRC_OK'),
      createEdge('ack', 'soh', 'NEXT'),
      createEdge('ack', 'eot', 'EOT')
    ]
  },

  // --- EXISTING (MISC) ---
  {
    id: 'traffic',
    name: 'Traffic Light',
    category: 'SYSTEM',
    description: 'Standard Red-Yellow-Green sequence.',
    nodes: [
      createNode('red', 'input', 'RED', 250, 100, 'HAL.writePin(1, true); dispatch("TIMER_30S", 3000);'),
      createNode('green', 'process', 'GREEN', 250, 500, 'HAL.writePin(1, false); HAL.writePin(2, true); dispatch("TIMER_20S", 2000);'),
      createNode('yel', 'process', 'YELLOW', 250, 300, 'HAL.writePin(2, false); HAL.writePin(3, true); dispatch("TIMER_5S", 1000);')
    ],
    edges: [
      createEdge('red', 'green', 'TIMER_30S'),
      createEdge('green', 'yel', 'TIMER_20S'),
      createEdge('yel', 'red', 'TIMER_5S')
    ]
  },
  {
    id: 'pid_ctrl',
    name: 'PID Temperature Control',
    category: 'DSP',
    description: 'Proportional-Integral-Derivative feedback loop.',
    nodes: [
      createNode('sample', 'input', 'SAMPLE_TEMP', 200, 200, 'ctx.input = HAL.getADC(0); dispatch("ADC_RDY", 200);'),
      createNode('error', 'process', 'CALC_ERROR', 400, 200, 'ctx.setpoint = 2000; ctx.err = ctx.setpoint - ctx.input; dispatch("ALWAYS", 100);'),
      createNode('pid', 'process', 'PID_CALC', 600, 200, 'ctx.p = 0.5*ctx.err; ctx.i = 0; ctx.d = 0; dispatch("ALWAYS", 100);'),
      createNode('out', 'process', 'PWM_UPDATE', 400, 400, 'HAL.setPWM(1, 50); dispatch("TIMER_TICK", 500);')
    ],
    edges: [
      createEdge('sample', 'error', 'ADC_RDY'),
      createEdge('error', 'pid', 'ALWAYS'),
      createEdge('pid', 'out', 'ALWAYS'),
      createEdge('out', 'sample', 'TIMER_TICK')
    ]
  },
  {
    id: 'watchdog',
    name: 'Watchdog Supervisor',
    category: 'SAFETY',
    description: 'External Watchdog kicker and task monitor.',
    nodes: [
      createNode('mon', 'input', 'MONITOR', 300, 200, 'ctx.flags = "OK"; dispatch("ALL_TASKS_OK", 1000);'),
      createNode('kick', 'process', 'KICK_WDT', 500, 200, 'HAL.writePin(9, 1); dispatch("TIMER_100MS", 100);'),
      createNode('fail', 'error', 'SYSTEM_RESET', 300, 400, 'console.error("WDT FAILURE");')
    ],
    edges: [
      createEdge('mon', 'kick', 'ALL_TASKS_OK'),
      createEdge('mon', 'fail', 'TASK_STUCK'),
      createEdge('kick', 'mon', 'TIMER_100MS')
    ]
  },
  {
    id: 'lorawan_join',
    name: 'LoRaWAN OTAA Join',
    category: 'IOT',
    description: 'Over-The-Air Activation flow for LoRa Nodes.',
    nodes: [
      createNode('init', 'input', 'INIT', 100, 200, 'dispatch("START", 500);'),
      createNode('tx', 'process', 'TX_JOIN_REQ', 300, 200, 'console.log("LoRa TX..."); dispatch("TX_DONE", 1000);'),
      createNode('rx1', 'process', 'RX_WINDOW_1', 500, 100, 'console.log("Listening RX1..."); dispatch("RX_TIMEOUT", 1500);'),
      createNode('rx2', 'process', 'RX_WINDOW_2', 500, 300, 'console.log("Listening RX2..."); dispatch("RX_TIMEOUT", 1500);'),
      createNode('sleep', 'process', 'SLEEP_RETRY', 300, 400, 'ctx.retries = (ctx.retries || 0) + 1; dispatch("DUTY_CYCLE_OK", 2000);'),
      createNode('joined', 'output', 'JOINED', 700, 200, 'ctx.devAddr = "01:23:45:67";')
    ],
    edges: [
      createEdge('init', 'tx', 'START'),
      createEdge('tx', 'rx1', 'TX_DONE'),
      createEdge('rx1', 'joined', 'JOIN_ACCEPT'),
      createEdge('rx1', 'rx2', 'RX_TIMEOUT'),
      createEdge('rx2', 'joined', 'JOIN_ACCEPT'),
      createEdge('rx2', 'sleep', 'RX_TIMEOUT'),
      createEdge('sleep', 'tx', 'DUTY_CYCLE_OK')
    ]
  }
];
