import DotParser from 'dotparser';

export type NodeData = {
  id: string;
  isInitial: boolean;
  isAccepting: boolean;
  additionalAttrs: string; // Store any other attributes as a string
};

export type EdgeData = {
  from: string;
  to: string;
  label: string;
};

export type ParsedAutomaton = {
  nodes: NodeData[];
  edges: EdgeData[];
};

export function parseNodesAndEdges(dotString: string): ParsedAutomaton {
  try {
    const ast = DotParser(dotString);
    const nodesMap = new Map<string, NodeData>();
    const edges: EdgeData[] = [];
    if (!Array.isArray(ast) || ast.length === 0) {
      throw new Error('Invalid DOT format');
    }
    const graph = ast[0];
    if (!Array.isArray(graph.children)) {
      throw new Error('Invalid graph structure');
    }
    for (const stmt of graph.children) {
      if (stmt.type === 'node_stmt') {
        const nodeId = String(stmt.node_id.id);
        let isInitial = false;
        let isAccepting = false;
        const otherAttrs: string[] = [];
        if (Array.isArray(stmt.attr_list)) {
          for (const attr of stmt.attr_list) {
            if (attr.id === 'isInitial' && attr.eq === 'true') {
              isInitial = true;
            } else if (attr.id === 'isAccepting' && attr.eq === 'true') {
              isAccepting = true;
            } else {
              otherAttrs.push(`${attr.id}="${attr.eq}"`);
            }
          }
        }

        nodesMap.set(nodeId, {
          id: nodeId,
          isInitial,
          isAccepting,
          additionalAttrs: otherAttrs.join(', '),
        });
      } else if (stmt.type === 'edge_stmt') {
        if (!Array.isArray(stmt.edge_list) || stmt.edge_list.length < 2) {
          continue;
        }
        let label = '';
        if (Array.isArray(stmt.attr_list)) {
          for (const attr of stmt.attr_list) {
            if (attr.id === 'label') {
              label = String(attr.eq);
              break;
            }
          }
        }
        for (let i = 0; i < stmt.edge_list.length - 1; i++) {
          const fromNode = stmt.edge_list[i];
          const toNode = stmt.edge_list[i + 1];
          if (fromNode.type !== 'node_id' || toNode.type !== 'node_id') {
            continue;
          }

          const from = String(fromNode.id);
          const to = String(toNode.id);
          if (!nodesMap.has(from)) {
            nodesMap.set(from, { id: from, isInitial: false, isAccepting: false, additionalAttrs: '' });
          }
          if (!nodesMap.has(to)) {
            nodesMap.set(to, { id: to, isInitial: false, isAccepting: false, additionalAttrs: '' });
          }

          edges.push({ from, to, label });
        }
      }
    }

    return {
      nodes: Array.from(nodesMap.values()),
      edges,
    };
  } catch (error) {
    throw new Error(`Failed to parse DOT format: ${error instanceof Error ? error.message : String(error)}`);
  }
}
