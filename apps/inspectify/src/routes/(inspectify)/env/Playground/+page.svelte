<script lang="ts">
  import Env from '$lib/components/Env.svelte';
  import StandardInput from '$lib/components/StandardInput.svelte';
  import { Io } from '$lib/io.svelte';

  const io = new Io('Playground', { simple_string: 'skip' });
</script>

<Env {io}>
  {#snippet inputView()}
    <StandardInput analysis="Playground" code="simple_string" {io} />
  {/snippet}
  {#snippet outputView({ output, referenceOutput })}
    <div class="grid grid-cols-1">
      <div class="relative">
        <div class="absolute inset-0 flex flex-col border-r">
          <h1 class="border-t bg-slate-900 p-2 text-2xl font-light italic">Capitalised String</h1> 
          <h2 class="overflow-auto rounded-md px-2 text-base whitespace-pre-wrap">{output.capitalised_string}</h2>  
        </div>
      </div>
    </div>
  {/snippet}
</Env>
