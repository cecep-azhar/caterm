<script lang="ts">
  import { Button, Input, Alert, Icon } from '@cecepazhar/caui';

  interface Props {
    title?: string;
    subtitle?: string;
    showSso?: boolean;
    loading?: boolean;
    onsubmit?: (data: { email: string; password?: string; remember: boolean }) => void;
    onsso?: (provider: 'gcc' | 'github' | 'google') => void;
    class?: string;
  }

  let {
    title = 'Sign In to CATerm Account',
    subtitle = 'Connect to CA Global Cloud & sync your encrypted servers',
    showSso = true,
    loading = false,
    onsubmit,
    onsso,
    class: customClass = '',
  }: Props = $props();

  let email = $state('developer@fathforce.com');
  let password = $state('••••••••••••');
  let showPassword = $state(false);
  let rememberMe = $state(true);
  let statusMessage = $state<{ type: 'info' | 'error' | 'success'; text: string } | null>(null);

  function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    if (!email) {
      statusMessage = { type: 'error', text: 'Please enter a valid email address.' };
      return;
    }
    statusMessage = null;
    onsubmit?.({ email, password, remember: rememberMe });
  }
</script>

<div class="w-full max-w-md mx-auto p-8 rounded-2xl bg-[#121217] border border-[#272732] text-[#EDEDED] font-sans shadow-2xl {customClass}">
  <!-- Brand Header -->
  <div class="text-center mb-6 space-y-2">
    <div class="w-12 h-12 rounded-xl bg-[#18181F] border border-[#272732] flex items-center justify-center mx-auto text-[var(--ca-brand)] shadow-inner">
      <Icon name="lock" size={22} />
    </div>
    <h2 class="text-xl font-bold text-white tracking-tight">{title}</h2>
    <p class="text-xs text-neutral-400 max-w-xs mx-auto leading-relaxed">{subtitle}</p>
  </div>

  {#if statusMessage}
    <div class="mb-4">
      <Alert variant={statusMessage.type === 'error' ? 'error' : statusMessage.type === 'success' ? 'success' : 'info'} description={statusMessage.text} dismissible />
    </div>
  {/if}

  <!-- SSO Quick Buttons -->
  {#if showSso}
    <div class="space-y-2 mb-6">
      <Button
        variant="outline"
        class="w-full justify-center text-xs py-2 bg-[#18181F]/80 hover:bg-[#20202A] border-[#272732]"
        onclick={() => onsso?.('gcc')}
      >
        <span class="w-2 h-2 rounded-full bg-emerald-500 mr-1.5"></span>
        Continue with GCC Global SSO
      </Button>
      <div class="grid grid-cols-2 gap-2">
        <Button
          variant="outline"
          class="w-full justify-center text-xs py-2 bg-[#18181F]/40 border-[#272732]"
          onclick={() => onsso?.('github')}
        >
          <Icon name="code" size={14} />
          GitHub
        </Button>
        <Button
          variant="outline"
          class="w-full justify-center text-xs py-2 bg-[#18181F]/40 border-[#272732]"
          onclick={() => onsso?.('google')}
        >
          <Icon name="globe" size={14} />
          Google Cloud
        </Button>
      </div>

      <div class="relative flex py-2 items-center">
        <div class="flex-grow border-t border-[#272732]"></div>
        <span class="shrink-0 mx-3 text-[10px] uppercase font-mono text-neutral-400">or email auth</span>
        <div class="flex-grow border-t border-[#272732]"></div>
      </div>
    </div>
  {/if}

  <!-- Form Login -->
  <form onsubmit={handleSubmit} class="space-y-4">
    <div>
      <label for="cads-login-email" class="block text-xs font-semibold uppercase tracking-wider text-neutral-400 mb-1.5">Work Email</label>
      <Input
        id="cads-login-email"
        type="email"
        bind:value={email}
        placeholder="name@company.com"
        required
      />
    </div>

    <div>
      <div class="flex items-center justify-between mb-1.5">
        <label for="cads-login-pass" class="block text-xs font-semibold uppercase tracking-wider text-neutral-400">Password / Token</label>
        <button type="button" class="text-[11px] text-[var(--ca-brand)] hover:underline cursor-pointer">
          Forgot?
        </button>
      </div>
      <div class="relative">
        <Input
          id="cads-login-pass"
          type={showPassword ? 'text' : 'password'}
          bind:value={password}
          placeholder="••••••••••••"
          required
        />
        <button
          type="button"
          onclick={() => (showPassword = !showPassword)}
          class="absolute right-3 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-white p-1 cursor-pointer"
          aria-label={showPassword ? 'Hide password' : 'Show password'}
        >
          <Icon name={showPassword ? 'x' : 'user'} size={14} />
        </button>
      </div>
    </div>

    <div class="flex items-center justify-between text-xs pt-1">
      <label class="flex items-center gap-2 cursor-pointer text-neutral-400 hover:text-neutral-300 select-none">
        <input
          type="checkbox"
          bind:checked={rememberMe}
          class="rounded border-neutral-700 bg-[#18181F] text-[var(--ca-brand)] focus:ring-0 focus:ring-offset-0 cursor-pointer"
        />
        <span>Remember this workstation</span>
      </label>
    </div>

    <Button
      type="submit"
      variant="brand"
      {loading}
      class="w-full py-2.5 font-bold text-xs justify-center shadow-lg cursor-pointer"
    >
      {loading ? 'Authenticating...' : 'Sign In'}
    </Button>
  </form>

  <div class="mt-6 pt-4 border-t border-[#272732] text-center text-xs text-neutral-400">
    Don't have a team vault?
    <button type="button" class="text-[var(--ca-brand)] font-medium hover:underline ml-1 cursor-pointer">
      Create Organization
    </button>
  </div>
</div>
