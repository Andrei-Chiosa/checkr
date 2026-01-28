<script lang="ts">
  import Env from '$lib/components/Env.svelte';
  import StandardInput from '$lib/components/StandardInput.svelte';
  import { Io } from '$lib/io.svelte';

  const io = new Io('Minimizer', { raw_input: 'skip' });
</script>

<Env {io}>
  {#snippet inputView()}
    <StandardInput analysis="Minimizer" code="raw_input" {io} />
  {/snippet}
  {#snippet outputView({ output, referenceOutput })}
    <h1>{output.test_output}</h1>
  {/snippet}
</Env>
