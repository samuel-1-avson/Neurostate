
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

  // --- EXISTING TEMPLATES ---
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

  // --- NEW DRIVER TEMPLATES ---
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
