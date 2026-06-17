import React, { useEffect, useRef, useState } from 'react';

function shortLabel(value, limit = 28) {
  if (value.startsWith('literal:')) return value.slice(8).slice(0, limit);
  if (value.startsWith('lit:')) return value.slice(4).slice(0, limit);
  if (value.startsWith('_:')) return value.slice(2).slice(0, limit);
  return value.split('/').pop()?.split('#').pop()?.slice(0, limit) || value.slice(0, limit);
}

function labelForNode(id, nodeType, fallback = '') {
  const raw = fallback || id;
  if (nodeType === 'literal' || String(nodeType || '').startsWith('literal')) {
    const stripped = String(raw).replace(/^literal:/, '').replace(/^lit:/, '');
    return `"${stripped.replace(/^"|"$/g, '').slice(0, 18)}"`;
  }
  return shortLabel(raw, 18);
}

function normalizeGraphData(graphData) {
  if (!graphData?.nodes || !graphData?.edges) return graphData;
  return {
    ...graphData,
    nodes: graphData.nodes.map(node => ({
      ...node,
      label: labelForNode(node.id, node.nodeType, node.label),
    })),
  };
}

function buildGraphModel(jsonld) {
  const items = jsonld?.['@graph'] || (Array.isArray(jsonld) ? jsonld : [jsonld]);
  const nodes = new Map();
  const edges = [];

  // Convert JSON-LD into a small internal graph model that the SVG renderer
  // can lay out and drag without needing a heavier graph library.
  const ensureNode = (id, type) => {
    if (nodes.has(id)) return nodes.get(id);
    const isBlank = id.startsWith('_:');
    const node = {
      id,
      label: labelForNode(id, type || (isBlank ? 'blank' : 'iri'), id),
      nodeType: type || (isBlank ? 'blank' : 'iri'),
      color: isBlank ? '#d97706' : '#4f46e5',
      shape: isBlank ? 'diamond' : 'ellipse',
      x: 0,
      y: 0,
      vx: 0,
      vy: 0,
    };
    nodes.set(id, node);
    return node;
  };

  items.forEach((item, i) => {
    if (!item || typeof item !== 'object') return;
    const sid = item['@id'] || `_:b${i}`;
    ensureNode(sid);
    Object.entries(item).forEach(([k, v]) => {
      if (k === '@id') return;
      const label = shortLabel(k, 18);
      [].concat(v).forEach((val, vi) => {
        if (val && typeof val === 'object' && val['@id']) {
          const oid = val['@id'];
          ensureNode(oid);
          edges.push({ id: `e${i}-${vi}-${edges.length}`, source: sid, target: oid, label });
          return;
        }
        const lit = (val && typeof val === 'object') ? (val['@value'] ?? JSON.stringify(val)) : val;
        const lid = `lit:${String(lit)}`;
        ensureNode(lid, 'literal');
        const node = nodes.get(lid);
        node.label = labelForNode(lid, 'literal', String(lit));
        node.color = '#059669';
        node.shape = 'round-rectangle';
        edges.push({ id: `e${i}-${vi}-${edges.length}`, source: sid, target: lid, label });
      });
    });
  });

  return { nodes: [...nodes.values()], edges };
}

function layoutGraph(model, width = 1000, height = 420) {
  const nodes = model.nodes.map(node => ({ ...node, x: 0, y: 0, vx: 0, vy: 0 }));
  if (!nodes.length) {
    return { nodes, edges: model.edges, viewBox: `0 0 ${width} ${height}` };
  }

  // First cluster connected nodes into components so each cluster can be
  // centered independently and the graph does not collapse into one pile.
  const nodeById = new Map(nodes.map(node => [node.id, node]));
  const outgoing = new Map(nodes.map(node => [node.id, []]));
  const incoming = new Map(nodes.map(node => [node.id, []]));
  const neighbors = new Map(nodes.map(node => [node.id, new Set()]));

  model.edges.forEach(edge => {
    if (!nodeById.has(edge.source) || !nodeById.has(edge.target)) return;
    outgoing.get(edge.source)?.push(edge);
    incoming.get(edge.target)?.push(edge);
    neighbors.get(edge.source)?.add(edge.target);
    neighbors.get(edge.target)?.add(edge.source);
  });

  const components = [];
  const seen = new Set();
  for (const node of nodes) {
    if (seen.has(node.id)) continue;
    const queue = [node.id];
    const component = [];
    seen.add(node.id);
    while (queue.length) {
      const id = queue.shift();
      const cur = nodeById.get(id);
      if (!cur) continue;
      component.push(cur);
      neighbors.get(id)?.forEach(next => {
        if (!seen.has(next)) {
          seen.add(next);
          queue.push(next);
        }
      });
    }
    components.push(component);
  }

  const degree = id => (neighbors.get(id)?.size || 0) + (outgoing.get(id)?.length || 0) * 0.9 + (incoming.get(id)?.length || 0) * 0.3;
  const resourceRank = node => {
    if (node?.nodeType === 'literal' || String(node?.nodeType || '').startsWith('literal')) return 2;
    if (String(node?.nodeType || '').startsWith('blank') || String(node?.id || '').startsWith('_:')) return 1;
    return 0;
  };

  const rootFor = component => [...component].sort((a, b) => {
    const typeDiff = resourceRank(a) - resourceRank(b);
    if (typeDiff !== 0) return typeDiff;
    const diff = degree(b.id) - degree(a.id);
    return diff !== 0 ? diff : a.id.localeCompare(b.id);
  })[0];

  const placements = new Map();
  const place = (id, x, y) => {
    placements.set(id, { x, y });
  };

  const layoutComponent = (component, centerX, centerY, rotation = -Math.PI / 2) => {
    const componentSet = new Set(component.map(node => node.id));
    const root = rootFor(component);
    // Seed nodes on concentric rings before the force pass to reduce overlap.
    const initialRadius = Math.max(100, Math.min(width, height) * 0.18);
    const positions = new Map(component.map((node, index) => {
      const angle = rotation + (index / Math.max(component.length, 1)) * Math.PI * 2;
      const ring = Math.floor(index / 8);
      const radius = index === 0 ? 0 : initialRadius + ring * 54;
      return [node.id, {
        x: centerX + Math.cos(angle) * radius,
        y: centerY + Math.sin(angle) * radius,
        vx: 0,
        vy: 0,
      }];
    }));
    positions.set(root.id, { x: centerX, y: centerY, vx: 0, vy: 0 });

    const links = model.edges
      .filter(edge => componentSet.has(edge.source) && componentSet.has(edge.target))
      .map(edge => ({ source: edge.source, target: edge.target }));

    const repulsion = 90000;
    const spring = 0.016;
    const desiredBase = Math.max(160, Math.min(width, height) * 0.22);
    const centerPull = 0.0012;

    for (let iter = 0; iter < 260; iter += 1) {
      const forces = new Map(component.map(node => [node.id, { fx: 0, fy: 0 }]));

      for (let i = 0; i < component.length; i += 1) {
        const a = component[i];
        const pa = positions.get(a.id);
        for (let j = i + 1; j < component.length; j += 1) {
          const b = component[j];
          const pb = positions.get(b.id);
          let dx = pb.x - pa.x;
          let dy = pb.y - pa.y;
          let dist = Math.hypot(dx, dy) || 0.001;
          const minDist = nodeRadiusFor(a) + nodeRadiusFor(b) + 60;
          if (dist < minDist) dist = minDist;
          const force = repulsion / (dist * dist);
          dx /= dist;
          dy /= dist;
          forces.get(a.id).fx -= dx * force;
          forces.get(a.id).fy -= dy * force;
          forces.get(b.id).fx += dx * force;
          forces.get(b.id).fy += dy * force;
        }
      }

      links.forEach(({ source, target }) => {
        const a = positions.get(source);
        const b = positions.get(target);
        if (!a || !b) return;
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        const dist = Math.hypot(dx, dy) || 0.001;
        const desired = desiredBase;
        const delta = dist - desired;
        const force = delta * spring;
        dx /= dist;
        dy /= dist;
        forces.get(source).fx += dx * force;
        forces.get(source).fy += dy * force;
        forces.get(target).fx -= dx * force;
        forces.get(target).fy -= dy * force;
      });

      component.forEach(node => {
        const p = positions.get(node.id);
        forces.get(node.id).fx += (centerX - p.x) * centerPull;
        forces.get(node.id).fy += (centerY - p.y) * centerPull;
      });

      component.forEach(node => {
        const p = positions.get(node.id);
        const f = forces.get(node.id);
        p.vx = (p.vx + f.fx) * 0.88;
        p.vy = (p.vy + f.fy) * 0.88;
        const maxStep = iter < 120 ? 20 : 9;
        p.vx = Math.max(-maxStep, Math.min(maxStep, p.vx));
        p.vy = Math.max(-maxStep, Math.min(maxStep, p.vy));
        p.x += p.vx;
        p.y += p.vy;
      });
    }

    component.forEach(node => {
      const p = positions.get(node.id);
      place(node.id, p.x, p.y);
    });
  };

  const orbit = components.length > 1 ? Math.max(70, Math.min(width, height) * 0.06) : 0;
  components.forEach((component, compIndex) => {
    const angle = components.length > 1
      ? (compIndex / components.length) * Math.PI * 2 - Math.PI / 2
      : 0;
    const centerX = width / 2 + Math.cos(angle) * orbit;
    const centerY = height / 2 + Math.sin(angle) * orbit * 0.4;
    layoutComponent(component, centerX, centerY, angle);
  });

  nodes.forEach(node => {
    const p = placements.get(node.id);
    if (!p) return;
    node.x = p.x;
    node.y = p.y;
  });

  const padding = Math.max(92, Math.min(width, height) * 0.14);
  let minX = Infinity;
  let maxX = -Infinity;
  let minY = Infinity;
  let maxY = -Infinity;
  nodes.forEach(node => {
    minX = Math.min(minX, node.x);
    maxX = Math.max(maxX, node.x);
    minY = Math.min(minY, node.y);
    maxY = Math.max(maxY, node.y);
  });
  const usedW = Math.max(1, maxX - minX);
  const usedH = Math.max(1, maxY - minY);
  const fit = Math.max(0.92, Math.min(2.0, Math.min((width - padding * 2) / usedW, (height - padding * 2) / usedH)));
  const cx = (minX + maxX) / 2;
  const cy = (minY + maxY) / 2;
  nodes.forEach(node => {
    node.x = width / 2 + (node.x - cx) * fit;
    node.y = height / 2 + (node.y - cy) * fit;
  });

  return {
    nodes,
    edges: model.edges,
    viewBox: `0 0 ${width} ${height}`,
    bounds: { minX, maxX, minY, maxY },
  };
}

function downloadSvg(svgNode, graph, filename = 'lod-graph.svg') {
  if (!svgNode) return;
  const serializer = new XMLSerializer();
  const clone = svgNode.cloneNode(true);
  // Recompute the exported bounds so the downloaded SVG captures the whole
  // graph instead of only the current viewport.
  const contentGroup = clone.querySelector('g');
  if (contentGroup) {
    contentGroup.removeAttribute('transform');
  }
  if (graph?.bounds) {
    const pad = 80;
    const minX = graph.bounds.minX - pad;
    const minY = graph.bounds.minY - pad;
    const maxX = graph.bounds.maxX + pad;
    const maxY = graph.bounds.maxY + pad;
    const width = Math.max(1, maxX - minX);
    const height = Math.max(1, maxY - minY);
    clone.setAttribute('viewBox', `${minX} ${minY} ${width} ${height}`);
    clone.setAttribute('width', `${width}`);
    clone.setAttribute('height', `${height}`);
    clone.setAttribute('preserveAspectRatio', 'xMidYMid meet');
  }
  const style = document.createElementNS('http://www.w3.org/2000/svg', 'style');
  style.textContent = `
    .glabel { fill: #1f2937; font-size: 11px; font-weight: 700; paint-order: stroke; stroke: rgba(255, 255, 255, 0.95); stroke-width: 4px; stroke-linejoin: round; pointer-events: none; }
    .gedge { stroke: #cbd5e1; stroke-width: 1.5px; fill: none; opacity: 0.95; }
    .gedge-label { fill: #475569; font-size: 9px; font-weight: 700; paint-order: stroke; stroke: rgba(255, 255, 255, 0.95); stroke-width: 4px; stroke-linejoin: round; pointer-events: none; }
  `;
  clone.insertBefore(style, clone.firstChild);
  const source = serializer.serializeToString(clone);
  const blob = new Blob([source], { type: 'image/svg+xml;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

function svgPoint(svg, clientX, clientY) {
  const pt = svg.createSVGPoint();
  pt.x = clientX;
  pt.y = clientY;
  return pt.matrixTransform(svg.getScreenCTM().inverse());
}

function graphPoint(svg, pan, zoom, clientX, clientY) {
  const p = svgPoint(svg, clientX, clientY);
  return {
    x: (p.x - pan.x) / zoom,
    y: (p.y - pan.y) / zoom,
  };
}

function nodeRadius(node) {
  if (node.shape === 'diamond') return 16;
  if (node.shape === 'round-rectangle') return 24;
  return 16;
}

function nodeRadiusFor(node) {
  if (node?.shape === 'round-rectangle') return 24;
  if (node?.shape === 'diamond') return 16;
  return 16;
}

function edgePath(source, target) {
  const dx = target.x - source.x;
  const dy = target.y - source.y;
  const dist = Math.sqrt(dx * dx + dy * dy) || 1;
  const sx = source.x + (dx / dist) * nodeRadius(source);
  const sy = source.y + (dy / dist) * nodeRadius(source);
  const tx = target.x - (dx / dist) * nodeRadius(target);
  const ty = target.y - (dy / dist) * nodeRadius(target);
  const midX = (sx + tx) / 2;
  const midY = (sy + ty) / 2;
  const offset = Math.max(10, Math.min(45, dist * 0.12));
  const perpX = (-dy / dist) * offset;
  const perpY = (dx / dist) * offset;
  return {
    d: `M ${sx} ${sy} Q ${midX + perpX} ${midY + perpY} ${tx} ${ty}`,
    lx: midX + perpX,
    ly: midY + perpY - 4,
  };
}

function svgNode(node) {
  const labelY = node.shape === 'round-rectangle' ? 30 : 24;
  const common = `transform="translate(${node.x} ${node.y})"`;
  if (node.shape === 'diamond') {
    return `
      <g ${common}>
        <title>${node.id}</title>
        <polygon points="0,-16 16,0 0,16 -16,0" fill="${node.color}" stroke="${node.color}" stroke-width="1.5" opacity="0.92"></polygon>
        <text x="0" y="${labelY}" text-anchor="middle" class="glabel">${node.label}</text>
      </g>`;
  }
  if (node.shape === 'round-rectangle') {
    return `
      <g ${common}>
        <title>${node.id}</title>
        <rect x="-24" y="-15" width="48" height="30" rx="8" ry="8" fill="${node.color}" stroke="${node.color}" stroke-width="1.5" opacity="0.92"></rect>
        <text x="0" y="${labelY}" text-anchor="middle" class="glabel">${node.label}</text>
      </g>`;
  }
  return `
    <g ${common}>
      <title>${node.id}</title>
      <circle r="16" fill="${node.color}" stroke="${node.color}" stroke-width="1.5" opacity="0.92"></circle>
      <text x="0" y="${labelY}" text-anchor="middle" class="glabel">${node.label}</text>
    </g>`;
}

export function GraphViewer({ graphData, jsonld }) {
  const svgRef = useRef(null);
  const boxRef = useRef(null);
  const dragRef = useRef(null);
  const [graph, setGraph] = useState({ nodes: [], edges: [], viewBox: '0 0 1000 420' });
  const [zoom, setZoom] = useState(1);
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const [draggingId, setDraggingId] = useState('');
  const [panning, setPanning] = useState(false);
  const [boxSize, setBoxSize] = useState({ width: 1000, height: 520 });

  useEffect(() => {
    const el = boxRef.current;
    if (!el) return undefined;
    const update = () => {
      const rect = el.getBoundingClientRect();
      if (!rect.width || !rect.height) return;
      setBoxSize({
        width: Math.max(640, rect.width),
        height: Math.max(360, rect.height),
      });
    };
    update();
    const ro = new ResizeObserver(update);
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  useEffect(() => {
    const model = graphData?.nodes && graphData?.edges ? normalizeGraphData(graphData) : buildGraphModel(jsonld);
    const layout = layoutGraph(model, boxSize.width, boxSize.height);
    setGraph(layout);
    fitLayout(layout);
  }, [graphData, jsonld, boxSize.width, boxSize.height]);

  const svgTransform = `translate(${pan.x} ${pan.y}) scale(${zoom})`;

  const updateNodePosition = (id, x, y) => {
    setGraph(prev => ({
      ...prev,
      nodes: prev.nodes.map(node => (node.id === id ? { ...node, x, y } : node)),
    }));
  };

  const clampZoom = (value) => Math.min(3, Math.max(0.45, value));

  const fitLayout = (layout) => {
    const bounds = layout?.bounds;
    if (!bounds) {
      setZoom(1);
      setPan({ x: 0, y: 0 });
      return;
    }
    // Center the graph in the visible SVG area and scale it to fit safely.
    const [vx, vy, vw, vh] = layout.viewBox.split(' ').map(Number);
    const contentW = Math.max(1, bounds.maxX - bounds.minX);
    const contentH = Math.max(1, bounds.maxY - bounds.minY);
    const pad = Math.max(110, Math.min(boxSize.width, boxSize.height) * 0.12);
    const fit = Math.min((vw - pad * 2) / contentW, (vh - pad * 2) / contentH);
    const nextZoom = clampZoom(fit * 0.9);
    const contentCx = (bounds.minX + bounds.maxX) / 2;
    const contentCy = (bounds.minY + bounds.maxY) / 2;
    const viewCx = vx + vw / 2;
    const viewCy = vy + vh / 2;
    setZoom(nextZoom);
    setPan({
      x: viewCx - contentCx * nextZoom,
      y: viewCy - contentCy * nextZoom,
    });
  };

  const zoomAtPoint = (nextZoom, pt) => {
    const svg = svgRef.current;
    if (!svg) return;
    const clamped = clampZoom(nextZoom);
    setPan(prev => {
      const factor = clamped / zoom;
      return {
        x: pt.x - (pt.x - prev.x) * factor,
        y: pt.y - (pt.y - prev.y) * factor,
      };
    });
    setZoom(clamped);
  };

  const beginDrag = (node, evt) => {
    const svg = svgRef.current;
    if (!svg) return;
    // Convert the pointer position into graph coordinates so the node can be
    // moved even while the graph is zoomed or panned.
    evt.preventDefault();
    evt.stopPropagation();
    const p = graphPoint(svg, pan, zoom, evt.clientX, evt.clientY);
    dragRef.current = {
      id: node.id,
      pointerId: evt.pointerId,
      startX: p.x,
      startY: p.y,
      nodeX: node.x,
      nodeY: node.y,
    };
    setDraggingId(node.id);
    svg.setPointerCapture?.(evt.pointerId);
  };

  const beginPan = (evt) => {
    const svg = svgRef.current;
    if (!svg || evt.button !== 0) return;
    if (evt.target !== svg) return;
    // Dragging the bare SVG background pans the full graph.
    evt.preventDefault();
    const p = svgPoint(svg, evt.clientX, evt.clientY);
    dragRef.current = {
      kind: 'pan',
      pointerId: evt.pointerId,
      startX: p.x,
      startY: p.y,
      panX: pan.x,
      panY: pan.y,
    };
    setPanning(true);
    svg.setPointerCapture?.(evt.pointerId);
  };

  const moveDrag = (evt) => {
    const drag = dragRef.current;
    const svg = svgRef.current;
    if (!drag || !svg) return;
    if (evt.pointerId !== drag.pointerId) return;
    evt.preventDefault();
    const p = drag.kind === 'pan'
      ? svgPoint(svg, evt.clientX, evt.clientY)
      : graphPoint(svg, pan, zoom, evt.clientX, evt.clientY);
    if (drag.kind === 'pan') {
      setPan({
        x: drag.panX + (p.x - drag.startX),
        y: drag.panY + (p.y - drag.startY),
      });
      return;
    }
    const nextX = drag.nodeX + (p.x - drag.startX);
    const nextY = drag.nodeY + (p.y - drag.startY);
    updateNodePosition(drag.id, nextX, nextY);
  };

  const endDrag = (evt) => {
    const drag = dragRef.current;
    const svg = svgRef.current;
    if (!drag || !svg) return;
    if (evt.pointerId !== drag.pointerId) return;
    dragRef.current = null;
    setDraggingId('');
    setPanning(false);
    svg.releasePointerCapture?.(evt.pointerId);
  };

  const handleWheel = (evt) => {
    evt.preventDefault();
    const svg = svgRef.current;
    if (!svg) return;
    // Zoom around the cursor to keep the graph interaction predictable.
    const pt = svgPoint(svg, evt.clientX, evt.clientY);
    const factor = evt.deltaY < 0 ? 1.12 : 1 / 1.12;
    zoomAtPoint(zoom * factor, pt);
  };

  return (
    <div className="gbox" ref={boxRef}>
      <svg
        ref={svgRef}
        className={`gcn ${draggingId ? 'is-dragging' : ''} ${panning ? 'is-panning' : ''}`}
        viewBox={graph.viewBox}
        preserveAspectRatio="xMidYMid meet"
        role="img"
        aria-label="RDF graph visualization"
        onPointerDown={beginPan}
        onPointerMove={moveDrag}
        onPointerUp={endDrag}
        onPointerLeave={endDrag}
        onWheel={handleWheel}
      >
        <defs>
          <marker id="g-arrow" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
            <path d="M 0 0 L 10 5 L 0 10 z" fill="#94a3b8" />
          </marker>
        </defs>
        <g transform={svgTransform}>
          {graph.edges.map(edge => {
            const source = graph.nodes.find(node => node.id === edge.source);
            const target = graph.nodes.find(node => node.id === edge.target);
            if (!source || !target) return null;
            const path = edgePath(source, target);
            const dist = Math.hypot(target.x - source.x, target.y - source.y);
            return (
              <g key={edge.id}>
                <title>{`${edge.label}: ${source.id} -> ${target.id}`}</title>
                <path d={path.d} className="gedge" markerEnd="url(#g-arrow)" />
                {dist > 180 && (
                  <text x={path.lx} y={path.ly} textAnchor="middle" className="gedge-label">
                    {edge.label}
                  </text>
                )}
              </g>
            );
          })}
          {graph.nodes.map(node => {
            const labelY = node.shape === 'round-rectangle' ? 30 : 24;
            const common = {
              key: node.id,
              transform: `translate(${node.x} ${node.y})`,
              className: `graph-node${draggingId === node.id ? ' dragging' : ''}`,
              onPointerDown: (evt) => beginDrag(node, evt),
            };
            if (node.shape === 'diamond') {
              return (
                <g {...common}>
                <title>{node.id}</title>
                <polygon points="0,-16 16,0 0,16 -16,0" fill={node.color} stroke={node.color} strokeWidth="1.5" opacity="0.92" />
                <text x="0" y={labelY} textAnchor="middle" className="glabel">{node.label}</text>
              </g>
            );
          }
          if (node.shape === 'round-rectangle') {
            return (
              <g {...common}>
                <title>{node.id}</title>
                <rect x="-24" y="-15" width="48" height="30" rx="8" ry="8" fill={node.color} stroke={node.color} strokeWidth="1.5" opacity="0.92" />
                <text x="0" y={labelY} textAnchor="middle" className="glabel">{node.label}</text>
              </g>
            );
          }
          return (
            <g {...common}>
              <title>{node.id}</title>
              <circle r="16" fill={node.color} stroke={node.color} strokeWidth="1.5" opacity="0.92" />
              <text x="0" y={labelY} textAnchor="middle" className="glabel">{node.label}</text>
            </g>
          );
        })}
      </g>
      </svg>
      <div className="gtools">
        <button className="gbtn" title="Fit to view" aria-label="Fit graph to view" onClick={() => { setZoom(1); setPan({ x: 0, y: 0 }); }}>
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" style={{ pointerEvents: 'none', display: 'block' }}>
            <path d="M8 3H5a2 2 0 00-2 2v3m18 0V5a2 2 0 00-2-2h-3m0 18h3a2 2 0 002-2v-3M3 16v3a2 2 0 002 2h3" />
          </svg>
        </button>
        <button className="gbtn" title="Zoom in" aria-label="Zoom graph in" onClick={() => {
          const [x, y, w, h] = graph.viewBox.split(' ').map(Number);
          zoomAtPoint(zoom * 1.2, { x: x + w / 2, y: y + h / 2 });
        }}>
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" style={{ pointerEvents: 'none', display: 'block' }}>
            <circle cx="11" cy="11" r="8" />
            <line x1="21" y1="21" x2="16.65" y2="16.65" />
            <line x1="11" y1="8" x2="11" y2="14" />
            <line x1="8" y1="11" x2="14" y2="11" />
          </svg>
        </button>
        <button className="gbtn" title="Zoom out" aria-label="Zoom graph out" onClick={() => {
          const [x, y, w, h] = graph.viewBox.split(' ').map(Number);
          zoomAtPoint(zoom / 1.2, { x: x + w / 2, y: y + h / 2 });
        }}>
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" style={{ pointerEvents: 'none', display: 'block' }}>
            <circle cx="11" cy="11" r="8" />
            <line x1="21" y1="21" x2="16.65" y2="16.65" />
            <line x1="8" y1="11" x2="14" y2="11" />
          </svg>
        </button>
        <button className="gbtn" title="Re-layout" aria-label="Re-layout graph" onClick={() => {
          const model = graphData?.nodes && graphData?.edges ? graphData : buildGraphModel(jsonld);
          const layout = layoutGraph(model, boxSize.width, boxSize.height);
          setGraph(layout);
          fitLayout(layout);
        }}>
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" style={{ pointerEvents: 'none', display: 'block' }}>
            <polyline points="1 4 1 10 7 10" />
            <path d="M3.51 15a9 9 0 102.13-9.36L1 10" />
          </svg>
        </button>
        <button className="gbtn" title="Export as SVG" aria-label="Export graph as SVG" onClick={() => downloadSvg(svgRef.current, graph, 'lod-graph.svg')}>
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" style={{ pointerEvents: 'none', display: 'block' }}>
            <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" />
            <polyline points="14 2 14 8 20 8" />
            <line x1="12" y1="18" x2="12" y2="12" />
            <polyline points="9 15 12 12 15 15" />
          </svg>
        </button>
      </div>
    </div>
  );
}
