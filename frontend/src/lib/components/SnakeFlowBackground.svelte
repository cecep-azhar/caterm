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

  interface Streamer {
    id: number;
    x: number;
    y: number;
    dirX: number; // -1, 0, 1
    dirY: number; // -1, 0, 1
    distTraveled: number;
    segmentDist: number;
    history: { x: number; y: number }[];
    length: number;
    speed: number;
    baseAlpha: number;
    lineWidth: number;
    fade: number; // 0 to 1
  }

  let streamers: Streamer[] = [];

  function stepStreamer(s: Streamer) {
    const nearTargetX = Math.abs(s.x - targetX) < Math.max(s.speed, 2.5);
    const nearTargetY = Math.abs(s.y - targetY) < Math.max(s.speed, 2.5);

    let shouldTurn = s.distTraveled >= s.segmentDist;
    if (!shouldTurn && s.distTraveled > 20) {
      if (s.dirX !== 0 && nearTargetX) {
        shouldTurn = true;
        s.x = targetX;
      } else if (s.dirY !== 0 && nearTargetY) {
        shouldTurn = true;
        s.y = targetY;
      }
    }

    if (shouldTurn) {
      if (s.dirX !== 0) {
        // Was moving horizontally, turn vertically (90 degrees)
        s.dirX = 0;
        const towards = s.y > targetY ? -1 : (s.y < targetY ? 1 : (Math.random() < 0.5 ? -1 : 1));
        // 85% towards target, 15% orthogonal branch away
        s.dirY = Math.random() < 0.85 ? towards : -towards;
      } else {
        // Was moving vertically, turn horizontally (90 degrees)
        s.dirY = 0;
        const towards = s.x > targetX ? -1 : (s.x < targetX ? 1 : (Math.random() < 0.5 ? -1 : 1));
        // 85% towards target, 15% orthogonal branch away
        s.dirX = Math.random() < 0.85 ? towards : -towards;
      }
      s.distTraveled = 0;
      s.segmentDist = 30 + Math.random() * 55;
    }

    s.x += s.dirX * s.speed;
    s.y += s.dirY * s.speed;
    s.distTraveled += s.speed;

    s.history.unshift({ x: s.x, y: s.y });
    if (s.history.length > s.length) {
      s.history.pop();
    }
  }

  function createStreamer(id: number, randomStart = false): Streamer {
    let x: number, y: number;
    let dirX = 0;
    let dirY = 0;

    if (randomStart) {
      x = Math.random() * (width || 600);
      y = Math.random() * (height || 800);
      if (Math.hypot(x - targetX, y - targetY) < 120) {
        x += 200;
        y += 200;
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
      if (edge < 0.45) {
        // Bottom edge: move up
        x = Math.random() * (width || 600);
        y = (height || 800) + 15 + Math.random() * 30;
        dirX = 0;
        dirY = -1;
      } else if (edge < 0.85) {
        // Right edge: move left
        x = (width || 600) + 15 + Math.random() * 30;
        y = Math.random() * (height || 800);
        dirX = -1;
        dirY = 0;
      } else {
        // Bottom-right corner: 50% up, 50% left
        x = (width || 600) - Math.random() * 120;
        y = (height || 800) + 15 + Math.random() * 20;
        if (Math.random() < 0.5) {
          dirX = 0;
          dirY = -1;
        } else {
          dirX = -1;
          dirY = 0;
        }
      }
    }

    const speed = 1.2 + Math.random() * 2.0; // 1.2 to 3.2 px/frame
    const length = 60 + Math.floor(Math.random() * 40); // 60 to 100 segments (2x longer)
    const segmentDist = 30 + Math.random() * 55; // 30 to 85 px
    const baseAlpha = 0.25 + Math.random() * 0.45;
    const lineWidth = 1.0 + Math.random() * 1.5;

    const streamer: Streamer = {
      id,
      x,
      y,
      dirX,
      dirY,
      distTraveled: 0,
      segmentDist,
      history: [{ x, y }],
      length,
      speed,
      baseAlpha,
      lineWidth,
      fade: randomStart ? 0.7 + Math.random() * 0.3 : 0,
    };

    if (randomStart) {
      const preWarmSteps = Math.floor(length * 0.8);
      for (let step = 0; step < preWarmSteps; step++) {
        if (Math.hypot(streamer.x - targetX, streamer.y - targetY) < 45) break;
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
      ctx.scale(dpr, dpr);
    }
  }

  function updateAndDraw() {
    if (!ctx || !width || !height) {
      if (isVisible) animId = requestAnimationFrame(updateAndDraw);
      return;
    }

    ctx.clearRect(0, 0, width, height);

    for (let i = 0; i < streamers.length; i++) {
      const s = streamers[i];

      if (s.fade < 1) {
        s.fade = Math.min(1, s.fade + 0.025);
      }

      const dx = targetX - s.x;
      const dy = targetY - s.y;
      const dist = Math.hypot(dx, dy);

      // Fade out and respawn when reaching near target logo
      if (dist < 38) {
        s.fade -= 0.06;
        if (s.fade <= 0) {
          streamers[i] = createStreamer(s.id, false);
          continue;
        }
      }

      // Respawn if streamer wandered far out of bounds
      if (s.x < -60 || s.y < -60 || s.x > width + 60 || s.y > height + 60) {
        streamers[i] = createStreamer(s.id, false);
        continue;
      }

      stepStreamer(s);

      if (s.history.length < 2) continue;

      ctx.save();
      ctx.beginPath();
      ctx.moveTo(s.history[0].x, s.history[0].y);

      for (let j = 1; j < s.history.length; j++) {
        ctx.lineTo(s.history[j].x, s.history[j].y);
      }

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

      const effectiveAlpha = s.baseAlpha * s.fade;
      grad.addColorStop(0, `rgba(255, 255, 255, ${effectiveAlpha.toFixed(3)})`);
      grad.addColorStop(0.25, `rgba(230, 245, 255, ${(effectiveAlpha * 0.85).toFixed(3)})`);
      grad.addColorStop(0.65, `rgba(180, 220, 255, ${(effectiveAlpha * 0.4).toFixed(3)})`);
      grad.addColorStop(1, 'rgba(255, 255, 255, 0)');

      ctx.strokeStyle = grad;
      ctx.lineWidth = s.lineWidth;
      ctx.lineCap = 'round';
      ctx.lineJoin = 'round';
      ctx.shadowColor = 'rgba(255, 255, 255, 0.5)';
      ctx.shadowBlur = 6;
      ctx.stroke();

      // Glowing dot at the head of the streamer
      const headAlpha = effectiveAlpha * 0.95;
      ctx.beginPath();
      ctx.arc(head.x, head.y, Math.max(s.lineWidth * 0.9, 1.2), 0, Math.PI * 2);
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
  class="absolute inset-0 pointer-events-none w-full h-full z-0 opacity-85"
  aria-hidden="true"
></canvas>
