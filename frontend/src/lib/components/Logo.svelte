<script lang="ts">
  export let size: number | string = 28;
  export let mode: 'light' | 'dark' | 'auto' | 'brand' | 'white' = 'brand';
  export let className: string = '';
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 512 512"
  class="shrink-0 {className}"
  xmlns="http://www.w3.org/2000/svg"
>
  <defs>
    <linearGradient id="catermLogoGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#F87171" />
      <stop offset="100%" stop-color="#EF4444" />
    </linearGradient>
  </defs>

  <!-- Terminal prompt chevrons >_ bold & iconic -->
  <path
    d="M140 152 L260 256 L140 360"
    fill="none"
    stroke={mode === 'white' ? '#FFFFFF' : mode === 'light' ? '#0A0A0C' : 'url(#catermLogoGrad)'}
    stroke-width="38"
    stroke-linecap="round"
    stroke-linejoin="round"
  />
  <line
    x1="288"
    y1="360"
    x2="384"
    y2="360"
    stroke={mode === 'white' ? '#FFFFFF' : mode === 'light' ? '#0A0A0C' : '#EF4444'}
    stroke-width="38"
    stroke-linecap="round"
  />
</svg>
