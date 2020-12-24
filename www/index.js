import { Cpu, Pixel, Display } from "wasm-chip8";
import { memory } from "wasm-chip8/wasm_chip8_bg";
import { cpu_execute_cycle } from "wasm-chip8/wasm_chip8_bg.wasm";

const CELL_SIZE = 20; // px
const GRID_COLOR = "#CCCCCC";
const DEAD_COLOR = "#333333";
const ALIVE_COLOR = "#41FF00";

// Construct the display, and get its width and height.
const chip8 = Cpu.new();
const width = 64;
const height = 32;

const programMemory = new Uint8Array(
    memory.buffer,
    chip8.get_memory(),
    4096);

const keyboardMemory = new Uint8Array(
    memory.buffer,
    chip8.get_keyboard(),
    16);

// Give the canvas room for all of our cells and a 1px border
// around each of them.
const canvas = document.getElementById("game-of-life-canvas");
canvas.height = (CELL_SIZE + 1) * height + 1;
canvas.width = (CELL_SIZE + 1) * width + 1;

const ctx = canvas.getContext('2d');

const renderLoop = () => {
  for (let i=0; i < 6; i++){
    chip8.execute_cycle(Math.floor(Math.random() * (0x100)));
  }

  chip8.decrement_timers();
  drawGrid();
  drawCells();

  requestAnimationFrame(renderLoop);
};

const drawGrid = () => {
  ctx.beginPath();
  ctx.strokeStyle = GRID_COLOR;

  // Vertical lines.
  for (let i = 0; i <= width; i++) {
    ctx.moveTo(i * (CELL_SIZE + 1) + 1, 0);
    ctx.lineTo(i * (CELL_SIZE + 1) + 1, (CELL_SIZE + 1) * height + 1);
  }

  // Horizontal lines.
  for (let j = 0; j <= height; j++) {
    ctx.moveTo(0,                           j * (CELL_SIZE + 1) + 1);
    ctx.lineTo((CELL_SIZE + 1) * width + 1, j * (CELL_SIZE + 1) + 1);
  }

  ctx.stroke();
};

const getIndex = (row, column) => {
  return row * width + column;
};

const drawCells = () => {
  const cellsPtr = chip8.get_display();
  const cells = new Uint8Array(memory.buffer, cellsPtr, width * height);

  ctx.beginPath();

  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      const idx = getIndex(row, col);

      ctx.fillStyle = cells[idx] === Pixel.Off
        ? DEAD_COLOR
        : ALIVE_COLOR;

      ctx.fillRect(
        col * (CELL_SIZE + 1) + 1,
        row * (CELL_SIZE + 1) + 1,
        CELL_SIZE,
        CELL_SIZE
      );
    }
  }

  ctx.stroke();
};


document.addEventListener("keydown", keyDownHandler, false);
document.addEventListener("keyup", keyUpHandler, false);

function keyDownHandler(e) {
  keyboardMemory[keyMappings[e.key]] = 1;
}

function keyUpHandler(e) {
    keyboardMemory[keyMappings[e.key]] = 0;
}

const keyMappings = {
  '1': 1,
  '2': 2,
  '3': 3,
  '4': 0xC,
  'q': 4,
  'w': 5,
  'e': 6,
  'r': 0xD,
  'a': 7,
  's': 8,
  'd': 9,
  'f': 0xE,
  'z': 0xA,
  'x': 0,  
  'c': 0xB,  
  'v': 0xF,
}

function readSingleFile(e) {
  var file = e.target.files[0];
  if (!file) {
    return;
  }
  var reader = new FileReader();
  reader.onload = function(e) {
    var buffer = e.target.result;
    console.log(new Uint8Array(buffer));
    const rom = new DataView(buffer, 0, buffer.byteLength);
    chip8.reset();
    for (let i = 0; i < rom.byteLength; i++) {
      programMemory[0x200 + i] = rom.getUint8(i);
    }
    renderLoop();
  };
  reader.readAsArrayBuffer(file);
}

document.getElementById('file-input').addEventListener('change', readSingleFile, false);