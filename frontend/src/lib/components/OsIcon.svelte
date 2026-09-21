<script lang="ts">
  let { os = '', name = '', tags = [], address = '', size = 16, class: className = '' }: {
    os?: string;
    name?: string;
    tags?: string[];
    address?: string;
    size?: number;
    class?: string;
  } = $props();

  export function detectOsKey(explicitOs?: string, hostName?: string, hostTags?: string[], hostAddress?: string): string {
    const raw = (explicitOs || '').trim().toLowerCase();
    if (raw) {
      if (raw.includes('ubuntu')) return 'ubuntu';
      if (raw.includes('debian')) return 'debian';
      if (raw.includes('fedora')) return 'fedora';
      if (raw.includes('red hat') || raw.includes('redhat') || raw.includes('rhel')) return 'redhat';
      if (raw.includes('centos')) return 'centos';
      if (raw.includes('rocky')) return 'rocky';
      if (raw.includes('almalinux') || raw.includes('alma')) return 'almalinux';
      if (raw.includes('arch')) return 'arch';
      if (raw.includes('manjaro')) return 'manjaro';
      if (raw.includes('alpine')) return 'alpine';
      if (raw.includes('opensuse') || raw.includes('suse')) return 'opensuse';
      if (raw.includes('mint')) return 'mint';
      if (raw.includes('kali')) return 'kali';
      if (raw.includes('pop') || raw.includes('pop!_os') || raw.includes('popos')) return 'popos';
      if (raw.includes('gentoo')) return 'gentoo';
      if (raw.includes('void')) return 'void';
      if (raw.includes('nixos') || raw.includes('nix')) return 'nixos';
      if (raw.includes('endeavour')) return 'endeavour';
      if (raw.includes('elementary')) return 'elementary';
      if (raw.includes('zorin')) return 'zorin';
      if (raw.includes('raspbian') || raw.includes('raspberry') || raw.includes('rpi')) return 'raspberry';
      if (raw.includes('amazon') || raw.includes('amzn')) return 'amazon';
      if (raw.includes('oracle')) return 'oracle';
      if (raw.includes('slackware')) return 'slackware';
      if (raw.includes('mageia')) return 'mageia';
      if (raw.includes('solus')) return 'solus';
      if (raw.includes('tails')) return 'tails';
      if (raw.includes('deepin')) return 'deepin';
      if (raw.includes('clear')) return 'clear';
      if (raw.includes('garuda')) return 'garuda';
      if (raw.includes('steamos') || raw.includes('steam')) return 'steam';
      if (raw.includes('coreos')) return 'coreos';
      if (raw.includes('flatcar')) return 'flatcar';
      if (raw.includes('devuan')) return 'devuan';
      if (raw.includes('parrot')) return 'parrot';
      if (raw.includes('mx')) return 'mx';
      if (raw.includes('lubuntu')) return 'lubuntu';
      if (raw.includes('xubuntu')) return 'xubuntu';
      if (raw.includes('kubuntu')) return 'kubuntu';
      if (raw.includes('freebsd')) return 'freebsd';
      if (raw.includes('openbsd')) return 'openbsd';
      if (raw.includes('netbsd')) return 'netbsd';
      if (raw.includes('mikrotik') || raw.includes('routeros')) return 'mikrotik';
      if (raw.includes('cisco')) return 'cisco';
      if (raw.includes('windows') || raw.includes('win')) return 'windows';
      if (raw.includes('macos') || raw.includes('apple') || raw.includes('mac') || raw.includes('darwin')) return 'macos';
      if (raw.includes('android')) return 'android';
      if (raw.includes('ios')) return 'ios';
      if (raw.includes('server')) return 'server';
      if (raw.includes('linux')) return 'linux';
      return raw;
    }

    const corpus = `${hostName || ''} ${(hostTags || []).join(' ')} ${hostAddress || ''}`.toLowerCase();
    if (corpus.includes('ubuntu')) return 'ubuntu';
    if (corpus.includes('debian')) return 'debian';
    if (corpus.includes('fedora') || corpus.includes('fc4')) return 'fedora';
    if (corpus.includes('rhel') || corpus.includes('redhat') || corpus.includes('red hat')) return 'redhat';
    if (corpus.includes('centos')) return 'centos';
    if (corpus.includes('rocky')) return 'rocky';
    if (corpus.includes('alma') || corpus.includes('almalinux')) return 'almalinux';
    if (corpus.includes('arch')) return 'arch';
    if (corpus.includes('manjaro')) return 'manjaro';
    if (corpus.includes('alpine')) return 'alpine';
    if (corpus.includes('opensuse') || corpus.includes('suse')) return 'opensuse';
    if (corpus.includes('mint')) return 'mint';
    if (corpus.includes('kali')) return 'kali';
    if (corpus.includes('pop-os') || corpus.includes('popos') || corpus.includes('pop!_os')) return 'popos';
    if (corpus.includes('gentoo')) return 'gentoo';
    if (corpus.includes('void')) return 'void';
    if (corpus.includes('nixos') || corpus.includes('nix')) return 'nixos';
    if (corpus.includes('endeavour')) return 'endeavour';
    if (corpus.includes('elementary')) return 'elementary';
    if (corpus.includes('zorin')) return 'zorin';
    if (corpus.includes('raspbian') || corpus.includes('raspberry') || corpus.includes('rpi')) return 'raspberry';
    if (corpus.includes('amazon') || corpus.includes('amzn') || corpus.includes('ec2') || corpus.includes('aws')) return 'amazon';
    if (corpus.includes('oracle') || corpus.includes('oci')) return 'oracle';
    if (corpus.includes('slackware')) return 'slackware';
    if (corpus.includes('mageia')) return 'mageia';
    if (corpus.includes('solus')) return 'solus';
    if (corpus.includes('tails')) return 'tails';
    if (corpus.includes('deepin')) return 'deepin';
    if (corpus.includes('clear linux') || corpus.includes('clearlinux')) return 'clear';
    if (corpus.includes('garuda')) return 'garuda';
    if (corpus.includes('steam')) return 'steam';
    if (corpus.includes('flatcar')) return 'flatcar';
    if (corpus.includes('coreos')) return 'coreos';
    if (corpus.includes('devuan')) return 'devuan';
    if (corpus.includes('parrot')) return 'parrot';
    if (corpus.includes('mx linux') || corpus.includes('mxlinux')) return 'mx';
    if (corpus.includes('lubuntu')) return 'lubuntu';
    if (corpus.includes('xubuntu')) return 'xubuntu';
    if (corpus.includes('kubuntu')) return 'kubuntu';
    if (corpus.includes('freebsd') || corpus.includes('bsd')) return 'freebsd';
    if (corpus.includes('openbsd')) return 'openbsd';
    if (corpus.includes('netbsd')) return 'netbsd';
    if (corpus.includes('mikrotik') || corpus.includes('routeros')) return 'mikrotik';
    if (corpus.includes('cisco') || corpus.includes('ios-xe')) return 'cisco';
    if (corpus.includes('windows') || corpus.includes('win10') || corpus.includes('win11') || corpus.includes('winserver') || corpus.includes('rdp')) return 'windows';
    if (corpus.includes('mac') || corpus.includes('osx') || corpus.includes('apple') || corpus.includes('darwin')) return 'macos';
    if (corpus.includes('android')) return 'android';
    if (corpus.includes('ios') || corpus.includes('iphone') || corpus.includes('ipad')) return 'ios';
    if (corpus.includes('vps') || corpus.includes('srv') || corpus.includes('node') || corpus.includes('host') || corpus.includes('server')) return 'server';

    return 'linux';
  }

  let osKey = $derived(detectOsKey(os, name, tags, address));
</script>

<span
  class="inline-flex items-center justify-center shrink-0 {className}"
  style="width: {size}px; height: {size}px;"
  title={osKey}
>
  {#if osKey === 'ubuntu'}
    <!-- Ubuntu circular coof logo -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#E95420]">
      <circle cx="12" cy="12" r="10" fill="#E95420" />
      <circle cx="6.5" cy="12" r="1.7" fill="#ffffff" />
      <circle cx="14.8" cy="7.2" r="1.7" fill="#ffffff" />
      <circle cx="14.8" cy="16.8" r="1.7" fill="#ffffff" />
      <path d="M12 5.2a6.8 6.8 0 0 1 5 2.2l-1.3 1.3a5 5 0 0 0-3.7-1.7 5.1 5.1 0 0 0-4.6 3h-1.9a7 7 0 0 1 6.5-4.8zm0 13.6a7 7 0 0 1-6.5-4.8h1.9a5.1 5.1 0 0 0 4.6 3 5 5 0 0 0 3.7-1.7l1.3 1.3a6.8 6.8 0 0 1-5 2.2zm4.3-8.8a5 5 0 0 0 0 4h1.9a6.9 6.9 0 0 1 0-4h-1.9z" fill="#ffffff" />
    </svg>
  {:else if osKey === 'debian'}
    <!-- Debian swirl -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#A81D33]">
      <path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10 10-4.5 10-10S17.5 2 12 2zm1.6 4.2c1.7.3 3 1.4 3.7 2.9.7 1.6.6 3.4-.2 4.9-.8 1.4-2.2 2.3-3.7 2.6-1.5.3-3.2-.2-4.2-1.3-.9-1-1.3-2.5-1-3.8.3-1.4 1.4-2.6 2.7-3.1 1.2-.5 2.7-.4 3.7.4.8.6 1.1 1.6.9 2.5-.2.9-.9 1.6-1.8 1.8-.7.2-1.6-.1-2-.7-.3-.5-.3-1.1-.1-1.6.1-.3.4-.6.7-.6.2 0 .4.2.4.4 0 .3-.2.5-.3.6.2.2.5.2.7.1.3-.1.5-.4.5-.7 0-.5-.3-.9-.7-1.1-.5-.2-1.1-.1-1.5.2-.6.5-.8 1.3-.7 2 .1.9.8 1.6 1.6 1.8 1.1.3 2.3-.2 2.9-1.2.6-1 .6-2.3 0-3.3-.6-1.1-1.7-1.8-2.9-2-1.3-.1-2.6.4-3.5 1.3-1 1-1.5 2.4-1.4 3.8.2 1.6 1.1 3.1 2.5 3.9 1.4.9 3.2 1.1 4.7.6 1.7-.6 3-1.9 3.7-3.5.7-1.8.6-3.8-.4-5.5-.8-1.5-2.2-2.7-3.9-3-1.9-.3-3.9.3-5.3 1.6" />
    </svg>
  {:else if osKey === 'fedora'}
    <!-- Fedora infinity f -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#294172]">
      <circle cx="12" cy="12" r="10" fill="#294172" />
      <path d="M12 6.5a4 4 0 0 0-4 4v5.5a2 2 0 0 0 4 0V13h2a2 2 0 1 0 0-4h-2v-1a2 2 0 0 1 2-2h1V4h-1a4 4 0 0 0-2 2.5z" fill="#3c6eb4" />
      <path d="M8 12.5h8v2H8z" fill="#ffffff" />
      <circle cx="14" cy="9.5" r="1.5" fill="#ffffff" />
    </svg>
  {:else if osKey === 'redhat'}
    <!-- Red Hat shadowman hat -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#EE0000]">
      <path d="M21.5 13.8c-.4-.4-1.2-.6-2.5-.7-.4-1.5-1.5-3.3-3.2-4.1-.3-.2-.7-.3-1.1-.3 0 0 .1-.5.1-.7 0-1.1-.8-2-1.9-2-.8 0-1.5.5-1.8 1.2-.4.9-.8 1.9-1.3 2.9-1 .3-1.9.9-2.6 1.7-.8.9-1.3 2.1-1.3 3.4 0 .3 0 .7.1 1-1.5.2-2.6.6-3.2 1-.7.5-.9 1.1-.7 1.6.3.7 1.5 1.2 3.4 1.5 2.1.3 4.8.4 7.6.4s5.5-.1 7.6-.4c1.9-.3 3.1-.8 3.4-1.5.3-.5 0-1.1-.7-1.7zm-9.7-5.8c.6 0 1 .4 1 1 0 .2 0 .4-.1.6-1-.1-1.8.2-2.4.8.2-.8.8-1.4 1.5-2.4zm-1 3.9c.7-.6 1.6-.9 2.7-.8 1.6.8 2.6 2.2 3 3.6-1.5.1-3.6.3-5.7.9-.2-.7-.3-1.4-.3-2.1 0-.6.1-1.1.3-1.6z" />
    </svg>
  {:else if osKey === 'centos'}
    <!-- CentOS 4-color symbol -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <path d="M12 3l3.5 3.5h-7L12 3z" fill="#93227F" />
      <path d="M21 12l-3.5 3.5v-7L21 12z" fill="#EFA724" />
      <path d="M12 21l-3.5-3.5h7L12 21z" fill="#8EB737" />
      <path d="M3 12l3.5-3.5v7L3 12z" fill="#262577" />
      <rect x="9.5" y="9.5" width="5" height="5" fill="#ffffff" />
    </svg>
  {:else if osKey === 'rocky'}
    <!-- Rocky Linux green mountain circle -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#10B981]">
      <path d="M12 2a10 10 0 1 0 10 10A10 10 0 0 0 12 2zm3.8 14.8l-3.8-5.7-3.8 5.7h-2.5l5.1-7.6a1.5 1.5 0 0 1 2.4 0l5.1 7.6z" />
    </svg>
  {:else if osKey === 'almalinux'}
    <!-- AlmaLinux symbol -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="8" cy="8" r="3" fill="#E43A36" />
      <circle cx="16" cy="8" r="3" fill="#F0B323" />
      <circle cx="8" cy="16" r="3" fill="#0E8A63" />
      <circle cx="16" cy="16" r="3" fill="#1C5EAE" />
    </svg>
  {:else if osKey === 'arch'}
    <!-- Arch Linux logo -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#1793D1]">
      <path d="M12 2.5c-.7 1.8-1.5 3.7-2.3 5.5l1.6 1.5c.7-.8 1.4-1.6 2-2.4.9 1.1 1.9 2.2 3 3.3.4.4.9.8 1.3 1.2-1.5 1.4-3.4 2.8-5.6 4-1.3.7-2.6 1.3-3.8 1.7 1-.7 2-1.4 3-2.3-1.1-.3-2.1-.8-3.1-1.5l-4.9 5.5C4.8 19.8 8.1 21 12 21c4.5 0 8.3-1.6 10-4.2L12 2.5z" />
    </svg>
  {:else if osKey === 'manjaro'}
    <!-- Manjaro 3 bars -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#35BF5C]">
      <path d="M4 4h4.5v16H4zm5.5 0h4.5v4.5H14v11.5H9.5zm5.5 5.5h5V20h-5z" />
    </svg>
  {:else if osKey === 'alpine'}
    <!-- Alpine Linux twin peaks -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#0D597F]">
      <path d="M12.5 4.5l-7 12.5h5.5l3.5-6.5 3.5 6.5h5.5l-7-12.5-2 3.5z" />
    </svg>
  {:else if osKey === 'opensuse'}
    <!-- openSUSE chameleon curve -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#73BA25]">
      <path d="M12 2a10 10 0 1 0 10 10A10 10 0 0 0 12 2zm1 14.5c-2.5 0-4.5-2-4.5-4.5S10.5 7.5 13 7.5c1.8 0 3.3 1 4.1 2.5l-1.8 1c-.5-.9-1.3-1.5-2.3-1.5-1.4 0-2.5 1.1-2.5 2.5s1.1 2.5 2.5 2.5c1 0 1.8-.6 2.3-1.5l1.8 1c-.8 1.5-2.3 2.5-4.1 2.5z" />
    </svg>
  {:else if osKey === 'mint'}
    <!-- Linux Mint leaf / LM symbol -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#87CF3E]">
      <circle cx="12" cy="12" r="10" fill="#87CF3E" />
      <path d="M7 8v8h3v-5a1 1 0 0 1 2 0v5h3v-5a1 1 0 0 1 2 0v5h2V11a3 3 0 0 0-3-3 3 3 0 0 0-2.5 1.3A3 3 0 0 0 11 8H7z" fill="#ffffff" />
    </svg>
  {:else if osKey === 'kali'}
    <!-- Kali Linux dragon icon -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#557C94]">
      <path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10 10-4.5 10-10S17.5 2 12 2zm4.5 12.8c-1.2 1.5-3.1 2.4-5 2.2-2.1-.2-3.9-1.7-4.5-3.8-.4-1.3-.2-2.7.5-3.9l1.8 1.1c-.4.8-.5 1.7-.2 2.5.4 1.3 1.5 2.2 2.8 2.4 1.2.1 2.4-.4 3.1-1.4l1.5.9z" />
    </svg>
  {:else if osKey === 'popos'}
    <!-- Pop!_OS exclamation emblem -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#48B9C7]">
      <circle cx="12" cy="12" r="10" fill="#48B9C7" />
      <rect x="10.5" y="6" width="3" height="7" rx="1.5" fill="#ffffff" />
      <circle cx="12" cy="16.5" r="1.5" fill="#ffffff" />
    </svg>
  {:else if osKey === 'gentoo'}
    <!-- Gentoo "g" swirl -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#9A91D3]">
      <path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10 10-4.5 10-10S17.5 2 12 2zm3 13.5c-1.5 1.2-3.5 1.5-5.2.8-1.7-.7-2.8-2.3-2.8-4.2s1.2-3.5 2.9-4.2c1.7-.7 3.7-.4 5.1.8l-1.3 1.3c-.9-.8-2.2-1-3.3-.5-1.1.5-1.8 1.6-1.8 2.8s.7 2.3 1.8 2.8c1.1.5 2.4.3 3.3-.5v-1.5h-2.5V11H15v4.5z" />
    </svg>
  {:else if osKey === 'nixos'}
    <!-- NixOS snowflake flake -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#5277C3]">
      <path d="M12 2l2.5 4.5h-5L12 2zm7.5 4.5l-1.5 5-4.5-2.5 6-2.5zm-15 0l6 2.5-4.5 2.5-1.5-5zm15 11l-6-2.5 4.5-2.5 1.5 5zm-15 0l1.5-5 4.5 2.5-6 2.5zm7.5 4.5l-2.5-4.5h5l-2.5 4.5z" />
    </svg>
  {:else if osKey === 'raspberry'}
    <!-- Raspberry Pi icon -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#C51A4A]">
      <path d="M9.5 4a2 2 0 0 1 2.5 1.8A2 2 0 0 1 14.5 4a2 2 0 0 1 2 2c0 .4-.1.7-.3 1 1 .3 1.8 1.2 1.8 2.4 0 .9-.5 1.7-1.2 2.1.8.4 1.2 1.2 1.2 2 0 1.1-.7 2-1.8 2.3.4.6.6 1.3.6 2 0 2.2-2.7 4.2-4.8 4.2s-4.8-2-4.8-4.2c0-.7.2-1.4.6-2-1.1-.3-1.8-1.2-1.8-2.3 0-.8.4-1.6 1.2-2-.7-.4-1.2-1.2-1.2-2.1 0-1.2.8-2.1 1.8-2.4-.2-.3-.3-.6-.3-1a2 2 0 0 1 2-2z" />
    </svg>
  {:else if osKey === 'amazon'}
    <!-- Amazon Linux AWS orange a -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#FF9900]">
      <path d="M18.8 17.5c-3.7 2.3-8.8 2.6-13.6.5-.6-.3-.7-.9-.2-1.3.5-.4 1-.4 1.6-.1 4.2 1.8 8.6 1.5 11.9-.4.7-.4 1.3.5.3 1.3zm1.6-1.5c-.3-.4-1.7-.2-2.4-.1-.2 0-.3-.2-.1-.3 1.1-.8 2.9-.6 3.2-.2.3.4-.1 2.3-1.2 3.1-.2.1-.3 0-.3-.2.2-.7.8-1.9.8-2.3zM12 4a5 5 0 0 0-5 5v1h3V9a2 2 0 0 1 4 0v1h1V9a5 5 0 0 0-5-5z" />
    </svg>
  {:else if osKey === 'oracle'}
    <!-- Oracle Linux red O -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#F80000]">
      <path d="M12 4C7.6 4 4 7.6 4 12s3.6 8 8 8 8-3.6 8-8-3.6-8-8-8zm0 13c-2.8 0-5-2.2-5-5s2.2-5 5-5 5 2.2 5 5-2.2 5-5 5z" />
    </svg>
  {:else if osKey === 'void'}
    <!-- Void Linux ring with slash -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#478061]">
      <circle cx="12" cy="12" r="9" fill="none" stroke="#478061" stroke-width="2.5" />
      <circle cx="12" cy="12" r="4" fill="#478061" />
    </svg>
  {:else if osKey === 'endeavour'}
    <!-- EndeavourOS spaceship arch -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#7F3FBF]">
      <path d="M3 18c3-7 8-12 15-14-2 6-5 11-10 15 3-1 7-3 10-6-4 6-10 9-15 5z" />
    </svg>
  {:else if osKey === 'elementary'}
    <!-- elementary OS 'e' loop -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#5B9BD5]">
      <circle cx="12" cy="12" r="9" fill="none" stroke="#5B9BD5" stroke-width="2" />
      <path d="M12 7c-2.8 0-5 2.2-5 5s2.2 5 5 5c1.8 0 3.3-.9 4.1-2.3l-1.6-.9c-.5.8-1.4 1.4-2.5 1.4-1.8 0-3.1-1.3-3.2-3H17c0-2.9-2.2-5.2-5-5.2zm0 1.8c1.5 0 2.8 1.1 3 2.4H9c.2-1.3 1.5-2.4 3-2.4z" />
    </svg>
  {:else if osKey === 'zorin'}
    <!-- Zorin OS 'Z' symbol -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#0CC1E8]">
      <path d="M5 6h14v2.5L9.5 16H19v2.5H5V16l9.5-7.5H5V6z" />
    </svg>
  {:else if osKey === 'slackware'}
    <!-- Slackware pipe emblem -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#24548E]">
      <circle cx="12" cy="12" r="10" fill="#24548E" />
      <path d="M8 8h3v8H8zm5 0h3v4h-3zm0 5h3v3h-3z" fill="#ffffff" />
    </svg>
  {:else if osKey === 'mageia'}
    <!-- Mageia cauldron spark -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#2671A6]">
      <circle cx="12" cy="6" r="2" fill="#59A5D8" />
      <path d="M7 11h10c0 4.5-2.2 8-5 8s-5-3.5-5-8z" />
    </svg>
  {:else if osKey === 'solus'}
    <!-- Solus sailboat -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#5294E2]">
      <path d="M12 3v13h7c0-6-3.5-11-7-13zm-1 3.5C8.5 8.5 6.5 12 6.5 16H11V6.5zM5 18c2 2 5 3 7 3s5-1 7-3H5z" />
    </svg>
  {:else if osKey === 'tails'}
    <!-- Tails anonymity camouflage -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#56347C]">
      <circle cx="12" cy="12" r="9" fill="none" stroke="#56347C" stroke-width="2" />
      <path d="M12 7v5l4 2" fill="none" stroke="#56347C" stroke-width="2" stroke-linecap="round" />
    </svg>
  {:else if osKey === 'deepin'}
    <!-- Deepin infinity swirl -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#007AFF]">
      <path d="M12 4a8 8 0 1 0 8 8 3 3 0 0 0-6 0 2 2 0 1 1-4 0 6 6 0 1 1 6 6" fill="none" stroke="#007AFF" stroke-width="2.5" stroke-linecap="round" />
    </svg>
  {:else if osKey === 'clear'}
    <!-- Clear Linux swirl -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#006CE0]">
      <circle cx="12" cy="12" r="10" fill="#006CE0" />
      <circle cx="14" cy="10" r="4" fill="#ffffff" />
    </svg>
  {:else if osKey === 'garuda'}
    <!-- Garuda Linux eagle wing -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#F75229]">
      <path d="M3 6c5 3 9 7 11 12 3-5 5-9 7-12-5 3-9 4-13 4-2 0-3-.5-5-4z" />
    </svg>
  {:else if osKey === 'steam'}
    <!-- SteamOS valve emblem -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#1A3A53]">
      <circle cx="12" cy="12" r="10" fill="#1A3A53" />
      <circle cx="9" cy="14" r="2.5" fill="#ffffff" />
      <circle cx="15" cy="9" r="3.5" fill="none" stroke="#ffffff" stroke-width="1.8" />
      <path d="M9 14l5-4" stroke="#ffffff" stroke-width="2" />
    </svg>
  {:else if osKey === 'flatcar' || osKey === 'coreos'}
    <!-- Flatcar / CoreOS container gear -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#E6522C]">
      <path d="M12 3l8 4.5v9L12 21l-8-4.5v-9L12 3zm0 3.5L7 9.5v5l5 3 5-3v-5l-5-3z" />
    </svg>
  {:else if osKey === 'devuan'}
    <!-- Devuan swirl ring -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#4F5B66]">
      <circle cx="12" cy="12" r="9" fill="none" stroke="#4F5B66" stroke-width="2.5" />
      <path d="M12 6a6 6 0 0 1 6 6h-3a3 3 0 0 0-3-3V6z" fill="#4F5B66" />
    </svg>
  {:else if osKey === 'parrot'}
    <!-- Parrot OS security symbol -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#00D0FF]">
      <circle cx="12" cy="12" r="10" fill="#00D0FF" />
      <path d="M8 8c3 0 6 2 7 5 1-2 2-3 3-4-2 6-5 8-8 8-1 0-2-.5-2-2 0-3 1-5 0-7z" fill="#0b1728" />
    </svg>
  {:else if osKey === 'mx'}
    <!-- MX Linux two crossed wings -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#232F3E]">
      <circle cx="12" cy="12" r="10" fill="#232F3E" />
      <path d="M7 8l5 5 5-5m-10 8l5-5 5 5" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" />
    </svg>
  {:else if osKey === 'lubuntu' || osKey === 'xubuntu' || osKey === 'kubuntu'}
    <!-- *buntu flavors -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#007FFF]">
      <circle cx="12" cy="12" r="10" fill="#007FFF" />
      <circle cx="7" cy="12" r="1.5" fill="#ffffff" />
      <circle cx="14.5" cy="7.5" r="1.5" fill="#ffffff" />
      <circle cx="14.5" cy="16.5" r="1.5" fill="#ffffff" />
      <circle cx="12" cy="12" r="4.5" fill="none" stroke="#ffffff" stroke-width="1.5" />
    </svg>
  {:else if osKey === 'freebsd'}
    <!-- FreeBSD red daemon horns -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#AB2B28]">
      <path d="M12 3a9 9 0 0 0-9 9c0 5 4 9 9 9s9-4 9-9a9 9 0 0 0-9-9zm4 4.5c.8 0 1.5.7 1.5 1.5s-.7 1.5-1.5 1.5-1.5-.7-1.5-1.5.7-1.5 1.5-1.5zm-8 0c.8 0 1.5.7 1.5 1.5S8.8 10.5 8 10.5 6.5 9.8 6.5 9s.7-1.5 1.5-1.5zM12 18c-3 0-5-2-5.5-4h11c-.5 2-2.5 4-5.5 4z" />
    </svg>
  {:else if osKey === 'openbsd'}
    <!-- OpenBSD blowfish puff -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#F2BE22]">
      <circle cx="12" cy="12" r="8" fill="#F2BE22" />
      <circle cx="8.5" cy="10.5" r="1.5" fill="#1f2937" />
      <circle cx="15.5" cy="10.5" r="1.5" fill="#1f2937" />
      <path d="M9 15c1.5 1.2 4.5 1.2 6 0" fill="none" stroke="#1f2937" stroke-width="1.8" stroke-linecap="round" />
    </svg>
  {:else if osKey === 'netbsd'}
    <!-- NetBSD orange flag -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#FF6600]">
      <path d="M5 4v16h2V13h10l-3-4.5L17 4H5z" />
    </svg>
  {:else if osKey === 'mikrotik'}
    <!-- MikroTik / RouterOS router symbol -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#2A5298]">
      <rect x="3" y="7" width="18" height="10" rx="2" fill="#2A5298" />
      <circle cx="7" cy="12" r="1.5" fill="#ffffff" />
      <circle cx="12" cy="12" r="1.5" fill="#ffffff" />
      <circle cx="17" cy="12" r="1.5" fill="#ffffff" />
    </svg>
  {:else if osKey === 'cisco'}
    <!-- Cisco bridge bars -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#049FD9]">
      <path d="M4 14v4M7 10v8M10 6v12M13 10v8M16 6v12M19 10v8M22 14v4" stroke="#049FD9" stroke-width="2.2" stroke-linecap="round" fill="none" />
    </svg>
  {:else if osKey === 'windows'}
    <!-- Microsoft Windows 4 panes -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#00A4EF]">
      <path d="M3 4.5l8-1.2v7.7H3V4.5zm0 8.5h8v7.7l-8-1.2V13zm9-9.8l9-1.4v9.2h-9V3.2zm0 10.8h9v9.2l-9-1.4V14z" />
    </svg>
  {:else if osKey === 'macos'}
    <!-- Apple logo -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-current text-neutral-700 dark:text-neutral-200">
      <path d="M18.7 19.5c-.8 1.2-1.7 2.4-3 2.5-1.3 0-1.7-.8-3.2-.8-1.5 0-2 .8-3.2.8-1.3 0-2.3-1.3-3.1-2.5-1.7-2.4-3-6.9-1.2-10 1-.7 2.4-1.2 3.8-1.2 1.4 0 2.4.9 3.2.9.8 0 2-.9 3.5-.9 1.4 0 2.7.5 3.5 1.5-3.1 1.7-2.6 6 0 7.2-.6 1.4-1.4 2.8-2.3 4m-3.4-14.7c.6-.8 1.1-1.9.9-3-.9.1-2 .6-2.6 1.4-.6.7-1.1 1.8-.9 2.9 1 0 2-.5 2.6-1.3z" />
    </svg>
  {:else if osKey === 'android'}
    <!-- Android bugdroid head -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-[#3DDC84]">
      <path d="M6 18c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2v-5H6v5zM16.5 6.5l1.4-1.4c.2-.2.2-.6 0-.8-.2-.2-.6-.2-.8 0l-1.6 1.6C14.4 5.3 13.2 5 12 5s-2.4.3-3.5.9L6.9 4.3c-.2-.2-.6-.2-.8 0-.2.2-.2.6 0 .8l1.4 1.4C5.9 7.7 5 9.7 5 12h14c0-2.3-.9-4.3-2.5-5.5z" />
      <circle cx="9" cy="9" r="1" fill="#ffffff" />
      <circle cx="15" cy="9" r="1" fill="#ffffff" />
    </svg>
  {:else if osKey === 'ios'}
    <!-- iOS phone emblem -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-current text-sky-500">
      <rect x="6" y="2" width="12" height="20" rx="3" fill="none" stroke="currentColor" stroke-width="2" />
      <circle cx="12" cy="18" r="1" fill="currentColor" />
      <line x1="10" y1="5" x2="14" y2="5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
    </svg>
  {:else if osKey === 'server'}
    <!-- Generic Server rack -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-none stroke-current text-sky-500">
      <rect x="3" y="4" width="18" height="6" rx="1.5" stroke-width="2" />
      <rect x="3" y="14" width="18" height="6" rx="1.5" stroke-width="2" />
      <circle cx="7" cy="7" r="1" fill="currentColor" />
      <circle cx="7" cy="17" r="1" fill="currentColor" />
      <line x1="15" y1="7" x2="17" y2="7" stroke-width="2" stroke-linecap="round" />
      <line x1="15" y1="17" x2="17" y2="17" stroke-width="2" stroke-linecap="round" />
    </svg>
  {:else}
    <!-- Generic Linux (Tux) fallback -->
    <svg viewBox="0 0 24 24" width={size} height={size} class="fill-current text-amber-500">
      <path d="M12 2c-2.8 0-4.5 2.5-4.5 5.5 0 1.2.3 2.5.8 3.5C6.8 12.3 5 14.5 5 17c0 2.8 2.5 4 7 4s7-1.2 7-4c0-2.5-1.8-4.7-3.3-6 .5-1 .8-2.3.8-3.5C16.5 4.5 14.8 2 12 2zm-1.5 5a1 1 0 1 1 0 2 1 1 0 0 1 0-2zm3 0a1 1 0 1 1 0 2 1 1 0 0 1 0-2zm-1.5 3c1 0 1.8.4 2 .8-.4.6-1.2.9-2 .9s-1.6-.3-2-.9c.2-.4 1-.8 2-.8z" />
    </svg>
  {/if}
</span>
