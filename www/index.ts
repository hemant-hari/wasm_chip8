import { Cpu } from 'wasm_chip8';
import { memory } from 'wasm_chip8/wasm_chip8_bg.wasm';
import CanvasWrapper from './CanvasWrapper';

// Construct the display, and get its width and height.
const chip8 = Cpu.new();
const width = 64;
const height = 32;

const programMemory = new Uint8Array(
  memory.buffer,
  chip8.get_memory(),
  4096,
);

const keyboardMemory = new Uint8Array(
  memory.buffer,
  chip8.get_keyboard(),
  16,
);

// Give the canvas room for all of our cells and a 1px border
// around each of them.
const canvas = <HTMLCanvasElement> document.getElementById('game-of-life-canvas');
const canvasWrapper = new CanvasWrapper(canvas, memory, height, width);

const renderLoop = () => {
  for (let i = 0; i < 6; i += 1) {
    chip8.execute_cycle(Math.floor(Math.random() * (0x100)));
  }

  chip8.decrement_timers();
  // canvasWrapper.drawGrid();
  canvasWrapper.drawCells(chip8.get_display());

  requestAnimationFrame(renderLoop);
};

const keyMappings = new Map<string, number>(Object.entries({
  1: 1,
  2: 2,
  3: 3,
  4: 0xC,
  q: 4,
  w: 5,
  e: 6,
  r: 0xD,
  a: 7,
  s: 8,
  d: 9,
  f: 0xE,
  z: 0xA,
  x: 0,
  c: 0xB,
  v: 0xF,
}));

function keyDownHandler(e: KeyboardEvent) {
  keyboardMemory[keyMappings.get(e.key)] = 1;
}

function keyUpHandler(e: KeyboardEvent) {
  keyboardMemory[keyMappings.get(e.key)] = 0;
}

document.addEventListener('keydown', keyDownHandler, false);
document.addEventListener('keyup', keyUpHandler, false);

function readSingleFile(e: InputEvent) {
  const file = (e.target as HTMLInputElement).files[0];
  if (!file) {
    return;
  }
  const reader = new FileReader();
  reader.onload = function (ev: ProgressEvent<FileReader>) {
    const buffer = ev.target.result as ArrayBuffer;
    const rom = new DataView(buffer, 0, buffer.byteLength);
    chip8.reset();
    for (let i = 0; i < rom.byteLength; i += 1) {
      programMemory[0x200 + i] = rom.getUint8(i);
    }
    renderLoop();
  };
  reader.readAsArrayBuffer(file);
}

document.getElementById('file-input').addEventListener('change', readSingleFile, false);
