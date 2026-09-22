<script lang="ts">
  import { onMount, onDestroy } from 'svelte';

  export let targetX = 72;
  export let targetY = 72;
  export let count = 16;

  let canvas: HTMLCanvasElement;
  let animId: number;
  let ctx: CanvasRenderingContext2D | null = null;
  let width = 0;
  let height = 0;
  let isVisible = true;

  interface Snake {
    id: number;
    x: number;
    y: number;
    history: { x: number; y: number }[];
    length: number;
    speed: number;
    waveFreq: number;
    waveAmp: number;
    phase: number;
    baseAlpha: number;
    lineWidth: number;
    curlOffset: number;
    curlSpeed: number;
    fade: number; // 0 to 1
  }

  let snakes: Snake[] = [];

  function createSnake(id: number, randomStart = false): Snake {
    // Spawn mostly from bottom, right edge, and bottom-right corner
    let x: number, y: number;
    const edge = Math.random();

    if (randomStart) {
      // Scatter initially across canvas for instant presence
      x = Math.random() * (width || 600);
      y = Math.random() * (height || 800);
      // Avoid starting too close to the target logo
      if (Math.hypot(x - targetX, y - targetY) < 120) {
        x += 200;
        y += 200;
      }
    } else if (edge < 0.45) {
      // Bottom edge
      x = Math.random() * width;
      y = height + 20 + Math.random() * 40;
    } else if (edge < 0.85) {
      // Right edge
      x = width + 20 + Math.random() * 40;
      y = Math.random() * height;
    } else {
      // Lower middle / bottom left
      x = Math.random() * (width * 0.7);
      y = height + 10;
    }

    const speed = 0.9 + Math.random() * 2.2; // Different speeds (0.9 to 3.1 px/frame)
    const length = 22 + Math.floor(Math.random() * 24); // 22 to 46 segments
    const waveFreq = 0.06 + Math.random() * 0.08;
    const waveAmp = 10 + Math.random() * 18;
    const baseAlpha = 0.25 + Math.random() * 0.45; // Subtle, elegant glow
    const lineWidth = 1.0 + Math.random() * 1.5;

    return {
      id,
      x,
      y,
      history: [{ x, y }],
      length,
      speed,
      waveFreq,
      waveAmp,
      phase: Math.random() * Math.PI * 2,
      baseAlpha,
      lineWidth,
      curlOffset: Math.random() * Math.PI * 2,
      curlSpeed: (Math.random() - 0.5) * 0.015,
      fade: 0,
    };
  }

  function initSnakes() {
    snakes = [];
    for (let i = 0; i < count; i++) {
      snakes.push(createSnake(i, true));
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
      ctx.scale(dpr, dpr);
    }
  }

  function updateAndDraw() {
    if (!ctx || !width || !height) {
      if (isVisible) animId = requestAnimationFrame(updateAndDraw);
      return;
    }

    ctx.clearRect(0, 0, width, height);

    for (let i = 0; i < snakes.length; i++) {
      const s = snakes[i];

      // Fade in smoothly when spawning
      if (s.fade < 1) {
        s.fade = Math.min(1, s.fade + 0.02);
      }

      const dx = targetX - s.x;
      const dy = targetY - s.y;
      const dist = Math.hypot(dx, dy);

      // Check if snake has reached near the CATerm logo
      if (dist < 38) {
        s.fade -= 0.06;
        if (s.fade <= 0) {
          snakes[i] = createSnake(s.id, false);
          continue;
        }
      }

      // Base heading towards target logo
      const baseAngle = Math.atan2(dy, dx);

      // Organic gentle curl
      s.curlOffset += s.curlSpeed;
      const curl = Math.sin(s.curlOffset) * 0.25;

      // Slithering sinusoidal wave perpendicular to movement
      s.phase += s.waveFreq * (s.speed * 0.8);
      const wave = Math.sin(s.phase) * s.waveAmp;
      const perpAngle = baseAngle + Math.PI / 2;

      // Forward step with slither offset
      const stepX = Math.cos(baseAngle + curl) * s.speed;
      const stepY = Math.sin(baseAngle + curl) * s.speed;
      const slitherX = Math.cos(perpAngle) * (wave * 0.08);
      const slitherY = Math.sin(perpAngle) * (wave * 0.08);

      s.x += stepX + slitherX;
      s.y += stepY + slitherY;

      s.history.unshift({ x: s.x, y: s.y });
      if (s.history.length > s.length) {
        s.history.pop();
      }

      // Draw the snake trail
      if (s.history.length < 2) continue;

      ctx.save();
      ctx.beginPath();
      ctx.moveTo(s.history[0].x, s.history[0].y);

      // Draw smooth curve along segments
      for (let j = 1; j < s.history.length - 1; j++) {
        const xc = (s.history[j].x + s.history[j + 1].x) / 2;
        const yc = (s.history[j].y + s.history[j + 1].y) / 2;
        ctx.quadraticCurveTo(s.history[j].x, s.history[j].y, xc, yc);
      }
      ctx.lineTo(s.history[s.history.length - 1].x, s.history[s.history.length - 1].y);

      // Create gradient along the snake body (head glowing bright, tail tapering)
      const head = s.history[0];
      const tail = s.history[s.history.length - 1];
      const grad = ctx.createLinearGradient(head.x, head.y, tail.x, tail.y);

      const effectiveAlpha = s.baseAlpha * s.fade;
      grad.addColorStop(0, `rgba(255, 255, 255, ${effectiveAlpha.toFixed(3)})`);
      grad.addColorStop(0.3, `rgba(240, 248, 255, ${(effectiveAlpha * 0.75).toFixed(3)})`);
      grad.addColorStop(0.7, `rgba(220, 235, 255, ${(effectiveAlpha * 0.35).toFixed(3)})`);
      grad.addColorStop(1, 'rgba(255, 255, 255, 0)');

      ctx.strokeStyle = grad;
      ctx.lineWidth = s.lineWidth;
      ctx.lineCap = 'round';
      ctx.lineJoin = 'round';
      ctx.shadowColor = 'rgba(255, 255, 255, 0.4)';
      ctx.shadowBlur = 6;
      ctx.stroke();

      // Subtle glowing dot at the head of the snake
      const headAlpha = effectiveAlpha * 0.9;
      ctx.beginPath();
      ctx.arc(head.x, head.y, s.lineWidth * 0.8, 0, Math.PI * 2);
      ctx.fillStyle = `rgba(255, 255, 255, ${headAlpha.toFixed(3)})`;
      ctx.shadowBlur = 8;
      ctx.shadowColor = 'rgba(255, 255, 255, 0.8)';
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
    initSnakes();
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
  class="absolute inset-0 pointer-events-none w-full h-full z-0 opacity-85"
  aria-hidden="true"
></canvas>
