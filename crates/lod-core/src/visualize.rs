use crate::{parser, LodError, Node, RdfFormat, VisualizationRequest};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;

#[derive(Debug, Clone, Serialize)]
struct NodeSpec {
    id: String,
    label: String,
    node_type: String,
    color: String,
    shape: String,
}

#[derive(Debug, Clone, Serialize)]
struct EdgeSpec {
    id: String,
    source: String,
    target: String,
    label: String,
}

#[derive(Debug, Clone, Serialize)]
struct GraphSpec {
    nodes: Vec<NodeSpec>,
    edges: Vec<EdgeSpec>,
}

/// Visualizes an RDF graph as a self-contained HTML page with an inline SVG graph.
///
/// The layout mirrors the browser app:
/// - connected components are grouped together
/// - nodes are arranged on concentric rings by graph distance
/// - edges are curved so labels and arrowheads stay visible
pub struct VisualizationService;

impl VisualizationService {
    pub fn visualize(&self, req: VisualizationRequest) -> Result<(), LodError> {
        let fmt = match req.input_format.as_deref() {
            Some(s) => Some(RdfFormat::parse(s)?),
            None => None,
        };
        let graph = parser::read_graph(&req.input_path, fmt)?;

        let subjects: BTreeSet<String> = graph.all_triples().map(|t| node_label(&t.subject)).collect();
        let mut nodes: BTreeMap<String, NodeSpec> = BTreeMap::new();
        let mut edges = Vec::new();

        for (i, t) in graph.all_triples().enumerate() {
            let s_label = node_label(&t.subject);
            let o_label = node_label(&t.object);

            nodes
                .entry(s_label.clone())
                .or_insert_with(|| make_node(&s_label, &t.subject, true));
            nodes
                .entry(o_label.clone())
                .or_insert_with(|| make_node(&o_label, &t.object, subjects.contains(&o_label)));

            edges.push(EdgeSpec {
                id: format!("e{i}"),
                source: s_label,
                target: o_label,
                label: short(&t.predicate),
            });
        }

        let spec = GraphSpec {
            nodes: nodes.into_values().collect(),
            edges,
        };

        fs::write(req.output_path, html(&serde_json::to_string(&spec)?))?;
        Ok(())
    }
}

fn node_label(n: &Node) -> String {
    match n {
        Node::Iri(i) => i.clone(),
        Node::Blank(b) => b.clone(),
        Node::Literal { value, .. } => format!("literal:{value}"),
    }
}

fn short(s: &str) -> String {
    s.rsplit(['#', '/']).next().unwrap_or(s).chars().take(42).collect()
}

fn make_node(id: &str, node: &Node, has_outgoing: bool) -> NodeSpec {
    let (node_type, color, shape) = match node {
        Node::Iri(_) => ("iri", "#4f46e5", "ellipse"),
        Node::Blank(_) => ("blank", "#d97706", "diamond"),
        Node::Literal { .. } => {
            if has_outgoing {
                ("literal-hub", "#059669", "round-rectangle")
            } else {
                ("literal-leaf", "#059669", "round-rectangle")
            }
        }
    };

    NodeSpec {
        id: id.to_string(),
        label: short(id),
        node_type: node_type.to_string(),
        color: color.to_string(),
        shape: shape.to_string(),
    }
}

fn html(elements_json: &str) -> String {
    let template = r##"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>LOD Graph</title>
<style>
* { box-sizing: border-box; }
:root {
  --bg: #f8fafc;
  --panel: rgba(255,255,255,0.94);
  --panel-border: #e2e8f0;
  --text: #1e293b;
  --muted: #64748b;
  --accent: #4f46e5;
}
html, body { margin: 0; height: 100%; }
body {
  font-family: system-ui, -apple-system, "Segoe UI", sans-serif;
  background: radial-gradient(circle at top, #ffffff 0%, var(--bg) 70%);
  color: var(--text);
  overflow: hidden;
}
#app { width: 100vw; height: 100vh; position: relative; }
#cy {
  width: 100%;
  height: 100%;
  display: block;
}
.graph-node { cursor: grab; }
.graph-node:active { cursor: grabbing; }
.panel {
  position: fixed;
  z-index: 100;
  background: var(--panel);
  border: 1px solid var(--panel-border);
  border-radius: 12px;
  box-shadow: 0 6px 20px rgba(15, 23, 42, 0.08);
  backdrop-filter: blur(6px);
}
#controls-panel {
  top: 1rem;
  left: 1rem;
  padding: 0.75rem 1rem;
  max-width: 19rem;
}
#controls-panel h2 {
  margin: 0 0 0.3rem;
  font-size: 1rem;
}
#controls-panel small {
  color: var(--muted);
  font-size: 0.8rem;
  line-height: 1.4;
}
#legend-panel {
  bottom: 1rem;
  left: 1rem;
  padding: 0.7rem 0.9rem;
  font-size: 0.8rem;
  line-height: 1.6;
}
#zoom-panel {
  bottom: 1rem;
  right: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  padding: 0.5rem;
}
#zoom-panel button {
  width: 38px;
  height: 38px;
  border: 1px solid #cbd5e1;
  border-radius: 9px;
  background: #fff;
  color: var(--text);
  font-size: 1rem;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform 0.12s, border-color 0.12s, background 0.12s, color 0.12s;
}
#zoom-panel button:hover {
  background: #eef2ff;
  border-color: var(--accent);
  color: var(--accent);
  transform: translateY(-1px);
}
#zoom-panel button:active {
  transform: translateY(0);
}
.legend-row {
  display: flex;
  align-items: center;
  gap: 0.55rem;
  margin: 0.22rem 0;
}
.legend-swatch {
  display: inline-block;
  width: 14px;
  height: 14px;
  border: 1px solid rgba(0,0,0,0.1);
  flex-shrink: 0;
}
.legend-swatch.circle { border-radius: 50%; }
.legend-swatch.diamond { border-radius: 2px; transform: rotate(45deg); margin: 4px; }
.legend-swatch.rect { border-radius: 3px; }
.badge {
  display: inline-block;
  font-size: 0.64rem;
  font-weight: 700;
  letter-spacing: 0.5px;
  color: var(--accent);
  background: #eef2ff;
  border-radius: 4px;
  padding: 0.12rem 0.45rem;
  vertical-align: middle;
}
.status {
  position: fixed;
  right: 1rem;
  top: 1rem;
  z-index: 100;
  padding: 0.45rem 0.75rem;
  font-size: 0.78rem;
  color: var(--muted);
}
.status strong { color: var(--accent); }
</style>
</head>
<body>
<div id="app">
  <div id="controls-panel" class="panel">
    <h2>LOD Graph Visualizer <span class="badge">RDF</span></h2>
    <small>Drag the graph to pan. Use the controls to zoom, fit, relayout, or export SVG.</small>
  </div>
  <div id="legend-panel" class="panel">
    <strong>Legend</strong>
    <div class="legend-row"><span class="legend-swatch circle" style="background:#4f46e5"></span> IRI</div>
    <div class="legend-row"><span class="legend-swatch diamond" style="background:#d97706"></span> Blank node</div>
    <div class="legend-row"><span class="legend-swatch rect" style="background:#059669"></span> Literal</div>
  </div>
  <div id="zoom-panel" class="panel">
    <button id="btn-zoom-in" title="Zoom in">+</button>
    <button id="btn-zoom-out" title="Zoom out">−</button>
    <button id="btn-fit" title="Fit graph to view">⊞</button>
    <button id="btn-reset-layout" title="Relayout graph">⟳</button>
    <button id="btn-export" title="Export as SVG">⬇</button>
  </div>
  <div id="status" class="status"><strong>Ready</strong></div>
  <svg id="cy" aria-label="LOD graph"></svg>
</div>
<script>
const GRAPH = __GRAPH_DATA__;

function shortLabel(value, limit = 28) {
  return value.split('/').pop()?.split('#').pop()?.slice(0, limit) || value.slice(0, limit);
}

function esc(value) {
  return String(value)
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;');
}

function layoutGraph(model, width = 1000, height = 700) {
  const nodes = model.nodes.map(node => ({ ...node, x: 0, y: 0, vx: 0, vy: 0 }));
  if (!nodes.length) return { nodes, edges: model.edges, viewBox: `0 0 ${width} ${height}` };

  const indexById = new Map(nodes.map(node => [node.id, node]));
  const neighbors = new Map(nodes.map(node => [node.id, new Set()]));
  const inbound = new Map(nodes.map(node => [node.id, 0]));
  const outbound = new Map(nodes.map(node => [node.id, 0]));
  model.edges.forEach(edge => {
    if (neighbors.has(edge.source) && neighbors.has(edge.target)) {
      neighbors.get(edge.source).add(edge.target);
      neighbors.get(edge.target).add(edge.source);
      outbound.set(edge.source, (outbound.get(edge.source) || 0) + 1);
      inbound.set(edge.target, (inbound.get(edge.target) || 0) + 1);
    }
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
      const cur = indexById.get(id);
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

  const isLiteral = node => node?.nodeType === 'literal' || node?.nodeType?.startsWith('literal');
  const degree = id => (neighbors.get(id)?.size || 0) + (outbound.get(id) || 0) * 1.5 - (inbound.get(id) || 0) * 0.25;
  const rootFor = component => [...component].sort((a, b) => {
    const scoreA = degree(a.id) - (isLiteral(a) ? 100 : 0);
    const scoreB = degree(b.id) - (isLiteral(b) ? 100 : 0);
    const diff = scoreB - scoreA;
    return diff !== 0 ? diff : a.id.localeCompare(b.id);
  })[0];

  function buildHubAndSpokeLayout(component, centerX, centerY, rotation = -Math.PI / 2) {
    const componentSet = new Set(component.map(node => node.id));
    const root = rootFor(component);
    const parent = new Map([[root.id, null]]);
    const depth = new Map([[root.id, 0]]);
    const queue = [root.id];
    const children = new Map(component.map(node => [node.id, []]));

    while (queue.length) {
      const id = queue.shift();
      const nextDepth = depth.get(id) + 1;
      const nexts = [...(neighbors.get(id) || [])]
        .filter(next => componentSet.has(next) && !parent.has(next))
        .sort((a, b) => {
          const diff = degree(b) - degree(a);
          return diff !== 0 ? diff : a.localeCompare(b);
        });
      nexts.forEach(next => {
        parent.set(next, id);
        depth.set(next, nextDepth);
        children.get(id)?.push(next);
        queue.push(next);
      });
    }

    component.forEach(node => {
      if (!parent.has(node.id)) {
        parent.set(node.id, root.id);
        depth.set(node.id, 1);
        children.get(root.id)?.push(node.id);
      }
    });

    component.forEach(node => {
      if (isLiteral(node)) {
        const parentId = parent.get(node.id);
        const baseDepth = parentId ? (depth.get(parentId) || 1) + 1 : 2;
        depth.set(node.id, Math.max(depth.get(node.id) || 0, baseDepth));
      }
    });

    const subtree = new Map();
    function size(id) {
      const kids = (children.get(id) || []).slice().sort((a, b) => {
        const diff = degree(b) - degree(a);
        return diff !== 0 ? diff : a.localeCompare(b);
      });
      children.set(id, kids);
      const total = 1 + kids.reduce((sum, kid) => sum + size(kid), 0);
      subtree.set(id, total);
      return total;
    }
    size(root.id);

    const placements = new Map([[root.id, { x: centerX, y: centerY }]]);
    const ringStep = Math.max(58, Math.min(width, height) * 0.105);

    function place(id, startAngle, endAngle, d) {
      const span = endAngle - startAngle;
      const angle = d === 0 ? rotation : startAngle + span / 2;
      const radius = d === 0 ? 0 : Math.max(d * ringStep, 34 + d * 8);
      placements.set(id, {
        x: centerX + Math.cos(angle) * radius,
        y: centerY + Math.sin(angle) * radius,
      });
      const kids = children.get(id) || [];
      if (!kids.length) return;
      const total = kids.reduce((sum, kid) => sum + (subtree.get(kid) || 1), 0) || kids.length;
      const usable = span * 0.88;
      const gap = (span - usable) / 2;
      let cursor = startAngle + gap;
      kids.forEach((kid, index) => {
        const weight = (subtree.get(kid) || 1) / total;
        const slice = Math.max((Math.PI / 42), usable * weight);
        const nextStart = cursor;
        const nextEnd = index === kids.length - 1 ? startAngle + span - gap : cursor + slice;
        place(kid, nextStart, nextEnd, d + 1);
        cursor = nextEnd;
      });
    }

    place(root.id, rotation - Math.PI, rotation + Math.PI, 0);

    component.forEach(node => {
      const p = placements.get(node.id) || { x: centerX, y: centerY };
      node.x = p.x;
      node.y = p.y;
    });
  }

  const orbit = Math.max(120, Math.min(width, height) * 0.20);
  components.forEach((component, compIndex) => {
    const angle = components.length > 1
      ? (compIndex / components.length) * Math.PI * 2 - Math.PI / 2
      : 0;
    const centerX = components.length > 1 ? width / 2 + Math.cos(angle) * orbit : width / 2;
    const centerY = components.length > 1 ? height / 2 + Math.sin(angle) * orbit * 0.60 : height / 2;
    buildHubAndSpokeLayout(component, centerX, centerY, angle);
  });

  const padding = Math.max(60, Math.min(width, height) * 0.08);
  let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
  nodes.forEach(node => {
    minX = Math.min(minX, node.x);
    maxX = Math.max(maxX, node.x);
    minY = Math.min(minY, node.y);
    maxY = Math.max(maxY, node.y);
  });
  const usedW = Math.max(1, maxX - minX);
  const usedH = Math.max(1, maxY - minY);
  const fit = Math.min((width - padding * 2) / usedW, (height - padding * 2) / usedH, 1);
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
  };
}

function nodeRadius(node) {
  if (node.shape === 'diamond') return 14;
  if (node.shape === 'round-rectangle') return 22;
  return 14;
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
  const labelY = node.shape === 'round-rectangle' ? 26 : 22;
  const common = `transform="translate(${node.x} ${node.y})" data-node-id="${esc(node.id)}" class="graph-node"`;
  if (node.shape === 'diamond') {
    return `
      <g ${common}>
        <title>${esc(node.id)}</title>
        <polygon points="0,-14 14,0 0,14 -14,0" fill="${node.color}" stroke="${node.color}" stroke-width="1.5" opacity="0.92"></polygon>
        <text x="0" y="${labelY}" text-anchor="middle" class="glabel">${esc(node.label)}</text>
      </g>`;
  }
  if (node.shape === 'round-rectangle') {
    return `
      <g ${common}>
        <title>${esc(node.id)}</title>
        <rect x="-22" y="-13" width="44" height="26" rx="7" ry="7" fill="${node.color}" stroke="${node.color}" stroke-width="1.5" opacity="0.92"></rect>
        <text x="0" y="${labelY}" text-anchor="middle" class="glabel">${esc(node.label)}</text>
      </g>`;
  }
  return `
    <g ${common}>
      <title>${esc(node.id)}</title>
      <circle r="14" fill="${node.color}" stroke="${node.color}" stroke-width="1.5" opacity="0.92"></circle>
      <text x="0" y="${labelY}" text-anchor="middle" class="glabel">${esc(node.label)}</text>
    </g>`;
}

function drawGraph(graph) {
  const svg = document.getElementById('cy');
  const vb = graph.viewBox;

  svg.setAttribute('viewBox', vb);
  svg.setAttribute('preserveAspectRatio', 'xMidYMid meet');
  svg.innerHTML = `
    <defs>
      <marker id="g-arrow" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
        <path d="M 0 0 L 10 5 L 0 10 z" fill="#94a3b8"></path>
      </marker>
      <filter id="g-shadow" x="-20%" y="-20%" width="140%" height="140%">
        <feDropShadow dx="0" dy="2" stdDeviation="2" flood-color="#94a3b8" flood-opacity="0.18"></feDropShadow>
      </filter>
      <style>
        .glabel { fill: #1f2937; font-size: 10px; font-weight: 600; paint-order: stroke; stroke: rgba(255,255,255,0.85); stroke-width: 3px; stroke-linejoin: round; pointer-events: none; }
        .gedge { stroke: #cbd5e1; stroke-width: 1.5px; fill: none; opacity: 0.95; }
        .gedge-label { fill: #64748b; font-size: 8px; font-weight: 600; paint-order: stroke; stroke: rgba(255,255,255,0.9); stroke-width: 3px; stroke-linejoin: round; pointer-events: none; }
      </style>
    </defs>
    ${graph.edges.map(edge => {
      const source = graph.nodes.find(node => node.id === edge.source);
      const target = graph.nodes.find(node => node.id === edge.target);
      if (!source || !target) return '';
      const path = edgePath(source, target);
      return `
        <g>
          <title>${esc(`${edge.label}: ${source.id} -> ${target.id}`)}</title>
          <path d="${path.d}" class="gedge" marker-end="url(#g-arrow)"></path>
          <text x="${path.lx}" y="${path.ly}" text-anchor="middle" class="gedge-label">${esc(edge.label)}</text>
        </g>`;
    }).join('')}
    ${graph.nodes.map(svgNode).join('')}
  `;
  return graph;
}

function relayoutGraph(model) {
  const { width, height } = currentViewport();
  currentGraph = layoutGraph(model, width, height);
  drawGraph(currentGraph);
  return currentGraph;
}

function setStatus(message) {
  const status = document.getElementById('status');
  if (status) status.innerHTML = `<strong>${message}</strong>`;
}

function currentSvg() {
  return document.getElementById('cy');
}

function currentViewport() {
  const svg = currentSvg();
  return {
    width: Math.max(640, svg?.clientWidth || window.innerWidth || 1000),
    height: Math.max(360, svg?.clientHeight || window.innerHeight || 700),
  };
}

function downloadSvg(svgNode, filename = 'lod-graph.svg') {
  if (!svgNode) return;
  const clone = svgNode.cloneNode(true);
  const serializer = new XMLSerializer();
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

let zoom = 1;
let currentGraph = relayoutGraph(GRAPH);
let dragState = null;

function applyZoom() {
  const svg = currentSvg();
  if (!svg || !currentGraph?.viewBox) return;
  const [x, y, w, h] = currentGraph.viewBox.split(' ').map(Number);
  const cx = x + w / 2;
  const cy = y + h / 2;
  const zw = w / zoom;
  const zh = h / zoom;
  svg.setAttribute('viewBox', `${cx - zw / 2} ${cy - zh / 2} ${zw} ${zh}`);
}

function svgPoint(svg, clientX, clientY) {
  const pt = svg.createSVGPoint();
  pt.x = clientX;
  pt.y = clientY;
  return pt.matrixTransform(svg.getScreenCTM().inverse());
}

function beginDrag(evt) {
  const svg = currentSvg();
  const nodeEl = evt.target.closest?.('[data-node-id]');
  if (!svg || !nodeEl) return;
  const nodeId = nodeEl.getAttribute('data-node-id');
  const node = currentGraph.nodes.find(n => n.id === nodeId);
  if (!node) return;
  evt.preventDefault();
  evt.stopPropagation();
  const p = svgPoint(svg, evt.clientX, evt.clientY);
  dragState = {
    id: nodeId,
    pointerId: evt.pointerId,
    startX: p.x,
    startY: p.y,
    nodeX: node.x,
    nodeY: node.y,
  };
  svg.setPointerCapture?.(evt.pointerId);
}

function moveDrag(evt) {
  const svg = currentSvg();
  if (!svg || !dragState || evt.pointerId !== dragState.pointerId) return;
  evt.preventDefault();
  const p = svgPoint(svg, evt.clientX, evt.clientY);
  const nextX = dragState.nodeX + (p.x - dragState.startX);
  const nextY = dragState.nodeY + (p.y - dragState.startY);
  currentGraph.nodes = currentGraph.nodes.map(node => node.id === dragState.id ? { ...node, x: nextX, y: nextY } : node);
  drawGraph(currentGraph);
  applyZoom();
}

function endDrag(evt) {
  const svg = currentSvg();
  if (!svg || !dragState || evt.pointerId !== dragState.pointerId) return;
  dragState = null;
  svg.releasePointerCapture?.(evt.pointerId);
}

document.getElementById('btn-zoom-in').addEventListener('click', () => {
  zoom = Math.min(3, zoom * 1.2);
  applyZoom();
});
document.getElementById('btn-zoom-out').addEventListener('click', () => {
  zoom = Math.max(0.5, zoom / 1.2);
  applyZoom();
});
document.getElementById('btn-fit').addEventListener('click', () => {
  zoom = 1;
  applyZoom();
});
document.getElementById('btn-reset-layout').addEventListener('click', () => {
  relayoutGraph(GRAPH);
  zoom = 1;
  applyZoom();
  setStatus('Relayout complete');
});
document.getElementById('btn-export').addEventListener('click', () => {
  const svg = currentSvg();
  if (!svg) return;
  const clone = svg.cloneNode(true);
  const style = document.createElementNS('http://www.w3.org/2000/svg', 'style');
  style.textContent = `
    .glabel { fill: #1f2937; font-size: 10px; font-weight: 600; paint-order: stroke; stroke: rgba(255,255,255,0.85); stroke-width: 3px; stroke-linejoin: round; pointer-events: none; }
    .gedge { stroke: #cbd5e1; stroke-width: 1.5px; fill: none; opacity: 0.95; }
    .gedge-label { fill: #64748b; font-size: 8px; font-weight: 600; paint-order: stroke; stroke: rgba(255,255,255,0.9); stroke-width: 3px; stroke-linejoin: round; pointer-events: none; }
  `;
  clone.insertBefore(style, clone.firstChild);
  downloadSvg(clone, 'lod-graph.svg');
  setStatus('SVG exported');
});

const svg = currentSvg();
svg.addEventListener('pointerdown', beginDrag);
svg.addEventListener('pointermove', moveDrag);
svg.addEventListener('pointerup', endDrag);
svg.addEventListener('pointerleave', endDrag);
window.addEventListener('resize', () => {
  if (dragState) return;
  relayoutGraph(GRAPH);
  applyZoom();
});
</script>
</body>
</html>"##;

    template.replace("__GRAPH_DATA__", elements_json)
}
