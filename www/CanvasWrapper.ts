import { Pixel } from 'wasm_chip8';

const CELL_SIZE = 20; // px
const GRID_COLOR = '#CCCCCC';
const DEAD_COLOR = '#333333';
const ALIVE_COLOR = '#41FF00';

export default class CanvasWrapper {
  canvas: HTMLCanvasElement;

  ctx: CanvasRenderingContext2D;

  memory: any;

  height: number;

  width: number;

  constructor(canvas: HTMLCanvasElement, memory: any, height: number, width: number) {
    this.canvas = canvas;
    this.canvas.height = (CELL_SIZE + 1) * height + 1;
    this.canvas.width = (CELL_SIZE + 1) * width + 1;
    this.memory = memory;
    this.height = height;
    this.width = width;
    this.ctx = canvas.getContext('2d');
  }

  drawGrid() {
    this.ctx.beginPath();
    this.ctx.strokeStyle = GRID_COLOR;

    // Vertical lines.
    for (let i = 0; i <= this.width; i += 1) {
      this.ctx.moveTo(i * (CELL_SIZE + 1) + 1, 0);
      this.ctx.lineTo(i * (CELL_SIZE + 1) + 1, (CELL_SIZE + 1) * this.height + 1);
    }

    // Horizontal lines.
    for (let j = 0; j <= this.height; j += 1) {
      this.ctx.moveTo(0, j * (CELL_SIZE + 1) + 1);
      this.ctx.lineTo((CELL_SIZE + 1) * this.width + 1, j * (CELL_SIZE + 1) + 1);
    }

    this.ctx.stroke();
  }

  drawCells(cellsPtr: any) {
    const cells = new Uint8Array(this.memory.buffer, cellsPtr, this.width * this.height);

    this.ctx.beginPath();

    this.drawPixels(cells, ALIVE_COLOR, Pixel.On);
    this.drawPixels(cells, DEAD_COLOR, Pixel.Off);

    this.ctx.stroke();
  }

  drawPixels(cells: Uint8Array, colour: string, pixelType: Pixel) {
    this.ctx.fillStyle = colour;

    for (let row = 0; row < this.height; row += 1) {
      for (let col = 0; col < this.width; col += 1) {
        const idx = this.getIndex(row, col);
        if (cells[idx] === pixelType) {
          this.ctx.fillRect(
            col * (CELL_SIZE + 1) + 1,
            row * (CELL_SIZE + 1) + 1,
            CELL_SIZE,
            CELL_SIZE,
          );
        }
      }
    }
  }

  getIndex(row: number, column: number): number {
    return row * this.width + column;
  }
}
