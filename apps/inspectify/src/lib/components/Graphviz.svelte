<script lang="ts">
  import { Graphviz } from '@hpcc-js/wasm-graphviz';
  import { parseNodesAndEdges } from '$lib/utils/dotHandler';
  import { mirage } from 'ayu';

  interface Props {
    dot: string;
  }
  let { dot }: Props = $props();

  let graphviz: Graphviz | null = $state(null);
  let svg = $state('');

  function enrichDot(dotStr: string): string {
    try {
      const { nodes, edges } = parseNodesAndEdges(dotStr);
      let initialNode: string | null = null;

      for (const node of nodes) {
        if (node.isInitial) {
          initialNode = node.id;
          break;
        }
      }

      let enriched = 'digraph DFA {\n  rankdir=LR\n';
      if (initialNode) {
        enriched += `  __start [label="", shape=none]\n  __start -> "${initialNode}"\n`;
      }

      for (const node of nodes) {
        const attrs: string[] = [];

        if (node.isAccepting) {
          attrs.push('shape=doublecircle');
          attrs.push('class="accepting"');
        } else {
          attrs.push('shape=circle');
        }
        if (node.additionalAttrs) {
          attrs.push(node.additionalAttrs);
        }

        enriched += `  "${node.id}" [${attrs.join(', ')}];\n`;
      }
      enriched += '\n';
      for (const edge of edges) {
        enriched += `  "${edge.from}" -> "${edge.to}" [label="${edge.label}"];\n`;
      }
      enriched += '}\n';
      return enriched;
    } catch (error) {
      console.error('Failed to parse DOT with dotparser:', error);
      return dotStr;
    }
  }

  $effect(() => {
    Graphviz.load().then((g) => {
      graphviz = g;
    });
  });

  $effect(() => {
    const currentDot = dot;
    if (graphviz && currentDot) {
      svg = graphviz.dot(enrichDot(currentDot));
    } else {
      svg = '';
    }
  });
</script>

<div class="flex h-full w-full items-center justify-center">{@html svg}</div>

<style>
  div :global(svg) {
    max-width: 100%;
    max-height: 100%;
    height: auto;
    width: auto;
    background: transparent;
  }

  /* Arrow heads */
  div :global(polygon[fill='black']) {
    fill: #dfbfff;
    stroke: none;
  }

  /* Node circles */
  div :global(ellipse) {
    stroke: none !important;
    stroke-width: 0 !important;
    fill: #6b7a8f !important;
  }

  /* Accepting nodes */
  div :global(g[class*='accepting'] ellipse) {
    stroke: #85cc95 !important;
    stroke-width: 1.5 !important;
    fill: #6b7a8f !important;
    filter: drop-shadow(0 0 4px #85cc95);
  }

  /* Second ellipse for accepting states */
  div :global(g[class*='accepting'] ellipse + ellipse) {
    fill: transparent !important;
  }

  /* Edges */
  div :global(path[stroke]) {
    stroke: #dfbfff !important;
    stroke-width: 1.5 !important;
  }

  /* Edge labels */
  div :global(text) {
    fill: white !important;
    font-family: 'Menlo', 'Monaco', 'Courier New', monospace;
  }

  /* Background */
  div :global(polygon[fill='white']),
  div :global(circle[fill='white']) {
    fill: transparent !important;
  }
</style>
