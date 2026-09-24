<script lang="ts">
  import { onMount, onDestroy } from 'svelte';

  interface Props {
    targetX?: number;
    targetY?: number;
    count?: number;
  }

  let { targetX = 64, targetY = 64, count = 17 }: Props = $props();

  const CELL_SIZE = 40;

  let canvas: HTMLCanvasElement;
  let animId = 0;
  let ctx: CanvasRenderingContext2D | null = null;
  let width = 0;
  let height = 0;
  let isVisible = true;

  interface Streamer {
    id: number;
    x: number;
    y: number;
    dirX: number; // -1, 0, 1
    dirY: number; // -1, 0, 1
    nextX: number;
    nextY: number;
    speed: number;
    length: number;
    history: { x: number; y: number }[];
    baseAlpha: number;
    lineWidth: number;
    fade: number; // 0 to 1
    reachedTarget: boolean;
  }

  let streamers: Streamer[] = [];

  function getGridOffset() {
    const startX = ((targetX % CELL_SIZE) + CELL_SIZE) % CELL_SIZE;
    const startY = ((targetY % CELL_SIZE) + CELL_SIZE) % CELL_SIZE;
    return { startX, startY };
  }

  function chooseNextDirection(s: Streamer) {
    const { startX, startY } = getGridOffset();
    const currDist = Math.hypot(targetX - s.x, targetY - s.y);

    const candidates = [
      { dirX: s.dirX, dirY: s.dirY, straight: true },
      { dirX: s.dirY, dirY: -s.dirX, straight: false },
      { dirX: -s.dirY, dirY: s.dirX, straight: false },
    ];

    const valid = candidates.filter((c) => {
      const nx = s.x + c.dirX * CELL_SIZE;
      const ny = s.y + c.dirY * CELL_SIZE;
      return (
        nx >= startX - CELL_SIZE &&
        ny >= startY - CELL_SIZE &&
        nx <= width + CELL_SIZE * 2 &&
        ny <= height + CELL_SIZE * 2
      );
    });

    const closer = valid.filter((c) => {
      const nx = s.x + c.dirX * CELL_SIZE;
      const ny = s.y + c.dirY * CELL_SIZE;
      return Math.hypot(targetX - nx, targetY - ny) < currDist;
    });

    let chosen: { dirX: number; dirY: number };

    if (closer.length > 0) {
      const straightCloser = closer.find((c) => c.straight);
      if (straightCloser && Math.random() < 0.65) {
        chosen = straightCloser;
      } else {
        chosen = closer[Math.floor(Math.random() * closer.length)];
      }
    } else if (valid.length > 0) {
      chosen = valid[Math.floor(Math.random() * valid.length)];
    } else {
      chosen = { dirX: -s.dirX, dirY: -s.dirY };
    }

    s.dirX = chosen.dirX;
    s.dirY = chosen.dirY;
  }

  function stepStreamer(s: Streamer) {
    if (!s.reachedTarget && s.fade < 1) {
      s.fade = Math.min(1, s.fade + 0.04);
    }

    if (s.reachedTarget) {
      s.fade -= 0.08;
    }

    const distToTarget = Math.hypot(s.x - targetX, s.y - targetY);
    if (distToTarget <= 32) {
      s.reachedTarget = true;
    }

    let remaining = s.speed;

    while (remaining > 0) {
      const distToNext =
        s.dirX !== 0 ? Math.abs(s.nextX - s.x) : Math.abs(s.nextY - s.y);

      if (remaining < distToNext) {
        s.x += s.dirX * remaining;
        s.y += s.dirY * remaining;
        remaining = 0;
      } else {
        s.x = s.nextX;
        s.y = s.nextY;
        remaining -= distToNext;

        s.history.unshift({ x: s.x, y: s.y });

        if (Math.hypot(s.x - targetX, s.y - targetY) <= 32) {
          s.reachedTarget = true;
        }

        chooseNextDirection(s);
        s.nextX = s.x + s.dirX * CELL_SIZE;
        s.nextY = s.y + s.dirY * CELL_SIZE;
      }
    }

    s.history.unshift({ x: s.x, y: s.y });
    while (s.history.length > s.length) {
      s.history.pop();
    }
  }

  function createStreamer(id: number, randomStart = false): Streamer {
    const { startX, startY } = getGridOffset();
    const w = width || 600;
    const h = height || 800;
    const cols = Math.max(1, Math.floor((w - startX) / CELL_SIZE));
    const rows = Math.max(1, Math.floor((h - startY) / CELL_SIZE));

    let x = 0;
    let y = 0;
    let dirX = 0;
    let dirY = 0;

    if (randomStart) {
      const col = 1 + Math.floor(Math.random() * cols);
      const row = 1 + Math.floor(Math.random() * rows);
      x = startX + col * CELL_SIZE;
      y = startY + row * CELL_SIZE;

      if (Math.hypot(x - targetX, y - targetY) < 100) {
        x += 2 * CELL_SIZE;
        y += 2 * CELL_SIZE;
      }

      if (Math.random() < 0.5) {
        dirX = x > targetX ? -1 : 1;
        dirY = 0;
      } else {
        dirX = 0;
        dirY = y > targetY ? -1 : 1;
      }
    } else {
      const edge = Math.random();
      if (edge < 0.5) {
        const col = Math.floor(Math.random() * (cols + 1));
        x = startX + col * CELL_SIZE;
        y = startY + (rows + 1) * CELL_SIZE;
        dirX = 0;
        dirY = -1;
      } else {
        const row = Math.floor(Math.random() * (rows + 1));
        x = startX + (cols + 1) * CELL_SIZE;
        y = startY + row * CELL_SIZE;
        dirX = -1;
        dirY = 0;
      }
    }

    const speed = 1.5 + Math.random() * 3.0; // 1.5 to 4.5 px/frame
    const length = 32 + Math.floor(Math.random() * 28);
    const baseAlpha = 0.35 + Math.random() * 0.45;
    const lineWidth = 1.2 + Math.random() * 1.0;

    const streamer: Streamer = {
      id,
      x,
      y,
      dirX,
      dirY,
      nextX: x + dirX * CELL_SIZE,
      nextY: y + dirY * CELL_SIZE,
      speed,
      length,
      history: [{ x, y }],
      baseAlpha,
      lineWidth,
      fade: randomStart ? 0.6 + Math.random() * 0.4 : 0,
      reachedTarget: false,
    };

    if (randomStart) {
      const preWarmSteps = Math.floor(length * 0.7);
      for (let step = 0; step < preWarmSteps; step++) {
        if (Math.hypot(streamer.x - targetX, streamer.y - targetY) < 32) break;
        stepStreamer(streamer);
      }
    }

    return streamer;
  }

  function initStreamers() {
    streamers = [];
    for (let i = 0; i < count; i++) {
      streamers.push(createStreamer(i, true));
    }
  }

  function resize() {
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    width = rect.width;
    height = rect.height;

    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    canvas.width = Math.floor(width * dpr);
    canvas.height = Math.floor(height * dpr);

    if (ctx) {
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    }
  }

  function drawGrid() {
    if (!ctx || !width || !height) return;
    const { startX, startY } = getGridOffset();

    ctx.save();
    ctx.beginPath();
    ctx.strokeStyle = 'rgba(56, 189, 248, 0.08)';
    ctx.lineWidth = 1;

    for (let x = startX; x <= width; x += CELL_SIZE) {
      const px = Math.floor(x) + 0.5;
      ctx.moveTo(px, 0);
      ctx.lineTo(px, height);
    }
    for (let y = startY; y <= height; y += CELL_SIZE) {
      const py = Math.floor(y) + 0.5;
      ctx.moveTo(0, py);
      ctx.lineTo(width, py);
    }
    ctx.stroke();

    ctx.fillStyle = 'rgba(56, 189, 248, 0.14)';
    for (let x = startX; x <= width; x += CELL_SIZE * 2) {
      for (let y = startY; y <= height; y += CELL_SIZE * 2) {
        ctx.fillRect(Math.floor(x) - 1, Math.floor(y) - 1, 2, 2);
      }
    }

    const glow = ctx.createRadialGradient(targetX, targetY, 0, targetX, targetY, 56);
    glow.addColorStop(0, 'rgba(56, 189, 248, 0.12)');
    glow.addColorStop(1, 'rgba(56, 189, 248, 0)');
    ctx.fillStyle = glow;
    ctx.beginPath();
    ctx.arc(targetX, targetY, 56, 0, Math.PI * 2);
    ctx.fill();

    ctx.restore();
  }

  function updateAndDraw() {
    if (!ctx || !width || !height) {
      if (isVisible) animId = requestAnimationFrame(updateAndDraw);
      return;
    }

    ctx.clearRect(0, 0, width, height);
    drawGrid();

    for (let i = 0; i < streamers.length; i++) {
      const s = streamers[i];

      if (
        (s.reachedTarget && s.fade <= 0) ||
        s.x < -80 ||
        s.y < -80 ||
        s.x > width + 80 ||
        s.y > height + 80
      ) {
        streamers[i] = createStreamer(s.id, false);
        continue;
      }

      stepStreamer(s);

      if (s.history.length < 2) continue;

      const head = s.history[0];
      const tail = s.history[s.history.length - 1];
      const gDist = Math.hypot(head.x - tail.x, head.y - tail.y);

      let grad: CanvasGradient;
      if (gDist > 4) {
        grad = ctx.createLinearGradient(head.x, head.y, tail.x, tail.y);
      } else {
        grad = ctx.createLinearGradient(
          head.x,
          head.y,
          head.x + (s.dirX !== 0 ? s.dirX : 1) * 30,
          head.y + (s.dirY !== 0 ? s.dirY : 1) * 30
        );
      }

      const alpha = s.baseAlpha * Math.max(0, s.fade);
      grad.addColorStop(0, `rgba(224, 242, 254, ${alpha.toFixed(3)})`);
      grad.addColorStop(0.25, `rgba(56, 189, 248, ${(alpha * 0.9).toFixed(3)})`);
      grad.addColorStop(0.7, `rgba(14, 165, 233, ${(alpha * 0.4).toFixed(3)})`);
      grad.addColorStop(1, 'rgba(56, 189, 248, 0)');

      ctx.save();
      ctx.beginPath();
      ctx.moveTo(head.x, head.y);
      for (let j = 1; j < s.history.length; j++) {
        ctx.lineTo(s.history[j].x, s.history[j].y);
      }

      ctx.strokeStyle = grad;
      ctx.lineWidth = s.lineWidth;
      ctx.lineCap = 'round';
      ctx.lineJoin = 'round';
      ctx.shadowColor = '#38bdf8';
      ctx.shadowBlur = 6;
      ctx.stroke();

      const headAlpha = alpha * 0.95;
      ctx.beginPath();
      ctx.arc(head.x, head.y, Math.max(s.lineWidth * 0.95, 1.8), 0, Math.PI * 2);
      ctx.fillStyle = `rgba(224, 242, 254, ${headAlpha.toFixed(3)})`;
      ctx.shadowColor = '#38bdf8';
      ctx.shadowBlur = 8;
      ctx.fill();

      ctx.restore();
    }

    if (isVisible) {
      animId = requestAnimationFrame(updateAndDraw);
    }
  }

  function handleVisibilityChange() {
    if (document.hidden) {
      isVisible = false;
      cancelAnimationFrame(animId);
    } else {
      if (!isVisible) {
        isVisible = true;
        animId = requestAnimationFrame(updateAndDraw);
      }
    }
  }

  onMount(() => {
    ctx = canvas.getContext('2d');
    resize();
    initStreamers();
    animId = requestAnimationFrame(updateAndDraw);

    const ro = new ResizeObserver(() => {
      resize();
    });
    ro.observe(canvas);
    document.addEventListener('visibilitychange', handleVisibilityChange);

    return () => {
      cancelAnimationFrame(animId);
      ro.disconnect();
      document.removeEventListener('visibilitychange', handleVisibilityChange);
    };
  });

  onDestroy(() => {
    if (typeof cancelAnimationFrame !== 'undefined' && animId) {
      cancelAnimationFrame(animId);
    }
  });
</script>

<canvas
  bind:this={canvas}
  class="absolute inset-0 pointer-events-none w-full h-full z-0"
  aria-hidden="true"
></canvas>
