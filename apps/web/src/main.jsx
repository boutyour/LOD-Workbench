import React, { useEffect, useRef, useState } from 'react';
import { createRoot } from 'react-dom/client';
import { GraphViewer } from './components/GraphViewer';
import CodeEditor from './components/CodeEditor';
import { API_URL, apiFetch } from './lib/api';
import './style.css';

// Default example exercises the features we want the UI to showcase: lists,
// bags, blank nodes, typed literals, and multi-valued properties.
const SAMPLE = `@prefix ex: <https://example.org/transport/> .
@prefix schema: <https://schema.org/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

ex:route-42 a schema:BusTrip ;
  schema:name "Airport Express"@en ;
  schema:departureTime "2026-06-17T08:30:00Z"^^xsd:dateTime ;
  schema:distance "12.4"^^xsd:decimal ;
  schema:amenityFeature (
    "Wi-Fi"
    "USB charging"
    "Wheelchair access"
  ) ;
  schema:provider ex:metro-transit ;
  schema:contactPoint [
    a schema:ContactPoint ;
    schema:telephone "+212-555-0142" ;
    schema:contactType "customer support"
  ] .

ex:metro-transit a schema:Organization ;
  schema:name "Metro Transit" ;
  schema:member [
    a schema:Person ;
    schema:name "Amina El Idrissi" ;
    schema:jobTitle "Operations lead" ;
    schema:birthDate "1990-04-15"^^xsd:date
  ] .

ex:route-42 schema:keywords [ rdf:type rdf:Bag ;
  rdf:_1 "airport" ;
  rdf:_2 "bus" ;
  rdf:_3 "rail"
] .

ex:route-42 schema:operator ex:metro-transit, ex:city-shuttle .
ex:city-shuttle a schema:Organization ;
  schema:name "City Shuttle" .`;

const SHAPES_SAMPLE = `@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix ex: <https://example.org/transport/> .
@prefix schema: <https://schema.org/> .

ex:BusTripShape a sh:NodeShape ;
  sh:targetClass schema:BusTrip ;
  sh:property [
    sh:path schema:name ;
    sh:minCount 1
  ] ;
  sh:property [
    sh:path schema:departureTime ;
    sh:minCount 1
  ] ;
  sh:property [
    sh:path schema:provider ;
    sh:minCount 1 ;
    sh:nodeKind sh:IRI
  ] ;
  sh:property [
    sh:path schema:contactPoint ;
    sh:minCount 1
  ] ;
  sh:property [
    sh:path schema:amenityFeature ;
    sh:minCount 1
  ] .

ex:OrganizationShape a sh:NodeShape ;
  sh:targetClass schema:Organization ;
  sh:property [
    sh:path schema:name ;
    sh:minCount 1
  ] .

ex:PersonShape a sh:NodeShape ;
  sh:targetClass schema:Person ;
  sh:property [
    sh:path schema:name ;
    sh:minCount 1
  ] .

ex:ContactPointShape a sh:NodeShape ;
  sh:targetClass schema:ContactPoint ;
  sh:property [
    sh:path schema:telephone ;
    sh:minCount 1
  ] ;
  sh:property [
    sh:path schema:contactType ;
    sh:minCount 1
  ] .

ex:AmenityListShape a sh:NodeShape ;
  sh:targetObjectsOf schema:amenityFeature ;
  sh:nodeKind sh:BlankNode ;
  sh:property [
    sh:path rdf:first ;
    sh:minCount 1
  ] ;
  sh:property [
    sh:path rdf:rest ;
    sh:minCount 1
  ] .`;

const FORMATS = [
  { value: 'turtle', label: 'Turtle' },
  { value: 'n-triples', label: 'N-Triples' },
  { value: 'json-ld', label: 'JSON-LD' },
];

const FORMAT_EXTENSIONS = {
  turtle: 'ttl',
  'n-triples': 'nt',
  'json-ld': 'jsonld',
};

function formatFromFilename(name) {
  const n = (name || '').toLowerCase();
  if (n.endsWith('.nt')) return 'n-triples';
  if (n.endsWith('.jsonld') || n.endsWith('.json')) return 'json-ld';
  return 'turtle';
}

function formatFilename(name, fmt) {
  const ext = FORMAT_EXTENSIONS[fmt] || 'rdf';
  const stem = (name || 'lod-workbench-output').replace(/\.[^.]+$/, '') || 'lod-workbench-output';
  return `${stem}.${ext}`;
}

function downloadText(content, filename, mimeType = 'text/plain;charset=utf-8') {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}

function InspectPanel({ d }) {
  if (!d) return null;
  return (
    <div className="panel">
      <div className="inspect-panel-head">
        <div>
          <h3>Graph inspection</h3>
          <p>Overview for the data graph and any loaded SHACL shapes.</p>
        </div>
      </div>
      <div className="inspect-grid">
        <SourceInspectSection title="Input RDF" report={d.data} kind="data" />
        {d.shapes ? <SourceInspectSection title="SHACL shapes" report={d.shapes} kind="shapes" /> : null}
      </div>
    </div>
  );
}

function countBySuffix(entries, suffixes) {
  if (!entries) return 0;
  return Object.entries(entries)
    .filter(([key]) => suffixes.some(suffix => key.endsWith(suffix)))
    .reduce((sum, [, value]) => sum + Number(value || 0), 0);
}

function countByPredicateSuffix(entries, suffixes) {
  if (!entries) return 0;
  return Object.entries(entries)
    .filter(([key]) => suffixes.some(suffix => key.endsWith(suffix)))
    .reduce((sum, [, value]) => sum + Number(value || 0), 0);
}

function SourceInspectSection({ title, report, kind }) {
  if (!report) {
    return (
      <section className={`inspect-source source-${kind}`}>
        <div className="inspect-source-head">
          <h4>{title}</h4>
          <span className="inspect-source-count">0</span>
        </div>
        <p className="muted">No {kind === 'shapes' ? 'SHACL shapes' : 'input RDF'} loaded.</p>
      </section>
    );
  }
  const isShapes = kind === 'shapes';
  const nodeShapes = isShapes ? countBySuffix(report.class_distribution, ['#NodeShape', '/NodeShape']) : null;
  const propertyShapes = isShapes ? countBySuffix(report.class_distribution, ['#PropertyShape', '/PropertyShape']) : null;
  const targetClassCount = isShapes ? countByPredicateSuffix(report.property_distribution, ['#targetClass', '/targetClass']) : null;
  const targetNodeCount = isShapes ? countByPredicateSuffix(report.property_distribution, ['#targetNode', '/targetNode']) : null;
  const targetObjectsOfCount = isShapes ? countByPredicateSuffix(report.property_distribution, ['#targetObjectsOf', '/targetObjectsOf']) : null;
  const targetSubjectsOfCount = isShapes ? countByPredicateSuffix(report.property_distribution, ['#targetSubjectsOf', '/targetSubjectsOf']) : null;
  const constraintCount = isShapes ? countByPredicateSuffix(report.property_distribution, [
    '#path', '/path',
    '#property', '/property',
    '#minCount', '/minCount',
    '#maxCount', '/maxCount',
    '#datatype', '/datatype',
    '#nodeKind', '/nodeKind',
    '#hasValue', '/hasValue',
    '#pattern', '/pattern',
    '#severity', '/severity',
    '#message', '/message',
  ]) : null;
  const cards = isShapes ? [
    { l: 'Triples', v: report.triples },
    { l: 'Node shapes', v: nodeShapes },
    { l: 'Property shapes', v: propertyShapes },
    { l: 'Target classes', v: targetClassCount },
    { l: 'Targets of / by', v: (targetNodeCount || 0) + (targetObjectsOfCount || 0) + (targetSubjectsOfCount || 0) },
    { l: 'Constraints', v: constraintCount },
    { l: 'IRIs', v: report.iris },
    { l: 'Blank', v: report.blank_nodes },
  ] : [
    { l: 'Triples', v: report.triples }, { l: 'Subjects', v: report.subjects }, { l: 'Predicates', v: report.predicates },
    { l: 'Objects', v: report.objects }, { l: 'IRIs', v: report.iris }, { l: 'Literals', v: report.literals },
    { l: 'Blank', v: report.blank_nodes }, { l: 'Classes', v: report.classes }, { l: 'Properties', v: report.properties },
  ];
  return (
    <section className={`inspect-source source-${kind}`}>
      <div className="inspect-source-head">
        <h4>{title}</h4>
        <span className="inspect-source-count">{report.triples ?? 0}</span>
      </div>
      <div className="card-grid inspect-cards">
        {cards.map(c => (
          <div key={c.l} className="stat-card">
            <span className="stat-val">{c.v}</span>
            <span className="stat-lbl">{c.l}</span>
          </div>
        ))}
      </div>
      {isShapes && (
        <>
          <div className="inspect-cluster">
            <h4>Shape declarations</h4>
            <MiniTable
              title="Declared shapes"
              data={{
                'sh:NodeShape': nodeShapes ?? 0,
                'sh:PropertyShape': propertyShapes ?? 0,
              }}
            />
          </div>
          <div className="inspect-cluster">
            <h4>Targets</h4>
            <MiniTable
              title="Target predicates"
              data={{
                'sh:targetClass': targetClassCount ?? 0,
                'sh:targetNode': targetNodeCount ?? 0,
                'sh:targetObjectsOf': targetObjectsOfCount ?? 0,
                'sh:targetSubjectsOf': targetSubjectsOfCount ?? 0,
              }}
            />
          </div>
          <div className="inspect-cluster">
            <h4>Constraint vocabulary</h4>
            {report.property_distribution && <MiniTable title="Top predicates" data={report.property_distribution} />}
          </div>
        </>
      )}
      {!isShapes && report.class_distribution && <MiniTable title="Class distribution" data={report.class_distribution} />}
      {!isShapes && report.property_distribution && <MiniTable title="Predicate distribution (top 6)" data={report.property_distribution} />}
      {report.prefixes && <div className="section"><h4>Prefixes</h4><pre className="pref">{JSON.stringify(report.prefixes, null, 2)}</pre></div>}
    </section>
  );
}

function MiniTable({ title, data }) {
  const es = Object.entries(data).slice(0, 6);
  if (!es.length) return null;
  return (
    <div className="section">
      <h4>{title}</h4>
      <table className="tbl">
        <tbody>
          {es.map(([k, v]) => (
            <tr key={k}><td className="tbl-k">{k.split('/').pop()}</td><td className="tbl-v">{v}</td></tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function ValidatePanel({
  d,
  onLocateLine,
  onDownloadReport,
  activeLine,
  title = 'Validation report',
  variant = 'syntax',
}) {
  if (!d) return null;
  const report = d.report ?? d;
  const syntaxIssues = report.issues ?? [];
  const isShaclReport = variant === 'shacl';
  const bannerOk = title === 'SHACL report' ? '✓ Conforms' : '✓ Syntax valid';
  const bannerFail = title === 'SHACL report' ? '✗ Does not conform' : '✗ Syntax issues found';
  const headerTag = isShaclReport ? 'Constraint report' : 'Syntax only';
  const headerTitle = isShaclReport ? title : `${title} (syntax only)`;
  return (
    <div className={`panel validate-panel validate-${variant}`}>
      <div className="panel-title">
        <div>
          <h3>{headerTitle}</h3>
          <p>{isShaclReport
            ? 'SHACL constraint results with focus node, constraint component and source shape.'
            : 'Checks RDF syntax, prefixes and basic IRI hygiene for the input graph and any loaded shapes. No SHACL inference here.'}</p>
        </div>
        <div className={`panel-badge ${isShaclReport ? 'shacl' : 'syntax'}`}>
          {headerTag}
        </div>
      </div>
      <div className={`banner ${report.conforms ? 'ok' : 'fail'}`}>{report.conforms ? bannerOk : bannerFail}</div>
      {!isShaclReport && syntaxIssues.length > 0 ? (
        <table className="tbl validation-table">
          <thead><tr><th>Sev</th><th>Source</th><th>Line</th><th>Column</th><th>Token</th><th>Message</th><th>Hint</th></tr></thead>
          <tbody>
            {syntaxIssues.map((x, i) => (
              <tr key={`syntax-${i}`} className={activeLine && x.line === activeLine ? 'row-active' : ''}>
                <td><span className={`sev ${x.severity.toLowerCase()}`}>{x.severity}</span></td>
                <td><span className={`source-chip ${(x.source || 'input').toLowerCase()}`}>{x.source || 'input'}</span></td>
                <td>
                  {x.line ? (
                    <button
                      type="button"
                      className="line-link"
                      onClick={() => onLocateLine?.(x.line)}
                      title={`Jump to line ${x.line}`}
                    >
                      {x.line}
                    </button>
                  ) : '—'}
                </td>
                <td>{x.column ?? '—'}</td>
                <td>{x.token ?? '—'}</td>
                <td><div className="issue-message">{x.message}</div></td>
                <td>{x.suggestion ?? '—'}</td>
              </tr>
            ))}
          </tbody>
        </table>
      ) : isShaclReport ? (
        report.issues?.length > 0 ? (
          <table className="tbl validation-table">
            <thead><tr><th>Sev</th><th>Line</th><th>Node</th><th>Constraint</th><th>Path</th><th>Value</th><th>Source shape</th><th>Message</th><th>Hint</th></tr></thead>
            <tbody>
              {report.issues.map((x, i) => (
                <tr key={`shacl-${i}`} className={activeLine && x.line === activeLine ? 'row-active' : ''}>
                  <td><span className={`sev ${x.severity.toLowerCase()}`}>{x.severity}</span></td>
                  <td>{x.line ? (
                    <button
                      type="button"
                      className="line-link"
                      onClick={() => onLocateLine?.(x.line)}
                      title={`Jump to line ${x.line}`}
                    >
                      {x.line}
                    </button>
                  ) : '—'}</td>
                  <td>{x.focus_node ?? x.token ?? '—'}</td>
                  <td>{x.constraint_component ?? '—'}</td>
                  <td>{x.path ?? '—'}</td>
                  <td>{x.value ?? '—'}</td>
                  <td>{x.source_shape ?? '—'}</td>
                  <td><div className="issue-message">{x.message}</div></td>
                  <td>{x.suggestion ?? '—'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        ) : <p className="muted">No SHACL violations found.</p>
      ) : <p className="muted">No syntax issues found.</p>}
      {isShaclReport && (
        <>
          <div className="section report-actions">
            <button type="button" className="act-btn" onClick={onDownloadReport} disabled={!report}>
              ⬇ Download report
            </button>
          </div>
          <details className="section" open>
            <summary>Validation report JSON</summary>
            <pre className="pref">{JSON.stringify(report, null, 2)}</pre>
          </details>
        </>
      )}
    </div>
  );
}

function ConvertPanel({ d }) {
  if (!d) return <p className="muted">Convert RDF between Turtle, N-Triples and JSON-LD.</p>;
  return <pre className="code-block">{d._raw ?? d.output ?? ''}</pre>;
}

function safeParseJson(value) {
  if (!value) return null;
  try {
    return JSON.parse(value);
  } catch {
    return null;
  }
}

function VisualizePanel({ d }) {
  if (!d) {
    return <p className="muted">Click <strong>Visualize</strong> below to render the RDF graph.</p>;
  }
  return (
    <>
      <div className="viz-meta">
        <span><b>{d.triples ?? 0}</b> triples</span>
        <div className="legend" aria-hidden="true">
          <span className="leg"><span className="sw i" /> IRI</span>
          <span className="leg"><span className="sw b" /> Blank</span>
          <span className="leg"><span className="sw l" /> Literal</span>
        </div>
      </div>
      <GraphViewer graphData={d.graph} jsonld={safeParseJson(d.jsonld)} />
    </>
  );
}

function App() {
  const [code, setCode] = useState(SAMPLE);
  const [shapesCode, setShapesCode] = useState(SHAPES_SAMPLE);
  const [shapesFmt, setShapesFmt] = useState('turtle');
  const [inFmt, setInFmt] = useState('turtle');
  const [outFmt, setOutFmt] = useState('json-ld');
  const [tab, setTab] = useState('inspect');
  const [busy, setBusy] = useState('');
  const [err, setErr] = useState('');
  const [results, setResults] = useState({});
  const [fname, setFname] = useState('');
  const [visualizeData, setVisualizeData] = useState(null);
  const [dirty, setDirty] = useState(false);
  const [shapesDirty, setShapesDirty] = useState(false);
  const [sidebarWidth, setSidebarWidth] = useState(330);
  const [shapesHeight, setShapesHeight] = useState(185);
  const [shapesName, setShapesName] = useState('shapes.ttl');
  const [activeLine, setActiveLine] = useState(null);
  const syncSeq = useRef(0);

  const codeRef = useRef(code);
  const shapesRef = useRef(shapesCode);
  const shapesFmtRef = useRef(shapesFmt);
  const inRef = useRef(inFmt);
  const outRef = useRef(outFmt);
  useEffect(() => { codeRef.current = code; }, [code]);
  useEffect(() => { shapesRef.current = shapesCode; }, [shapesCode]);
  useEffect(() => { shapesFmtRef.current = shapesFmt; }, [shapesFmt]);
  useEffect(() => { inRef.current = inFmt; }, [inFmt]);
  useEffect(() => { outRef.current = outFmt; }, [outFmt]);

  const buildValidationBody = (content, format) => {
    const shapes = shapesRef.current.trim();
    return {
      content,
      format,
      shapes_content: shapes || undefined,
      shapes_format: shapes ? shapesFmtRef.current : undefined,
    };
  };

  const requestPayload = async (endpoint, body, raw, signal) => {
    const r = await apiFetch(endpoint, body, { signal });
    if (!r.ok) {
      throw new Error(r.error);
    }
    return raw ? { ...r.data, _raw: r.data.output } : r.data;
  };

  const requestInspectPayload = async (signal) => {
    const data = await requestPayload(
      '/api/inspect-text',
      { content: codeRef.current, format: inRef.current },
      false,
      signal,
    );
    const shapes = shapesRef.current.trim()
      ? await requestPayload(
        '/api/inspect-text',
        { content: shapesRef.current, format: shapesFmtRef.current },
        false,
        signal,
      )
      : null;
    return { data, shapes };
  };

  const runAction = async (tid, endpoint, buildBody, raw) => {
    setErr('');
    setBusy(endpoint);
    try {
      const payload = tid === 'inspect'
        ? await requestInspectPayload()
        : await requestPayload(endpoint, buildBody(codeRef.current, inRef.current, outRef.current), raw);
      setTab(tid);
      setResults(prev => ({ ...prev, [tid]: payload }));
      if (tid === 'visualize') {
        setVisualizeData(payload);
      }
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy('');
    }
  };

  const saveCurrent = async () => {
    if (!dirty) return;
    setErr('');
    setBusy('/api/save-text');
    try {
      const source = codeRef.current;
      const inputFormat = inRef.current;
      downloadText(source, formatFilename(fname, inputFormat));
      setDirty(false);
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy('');
    }
  };

  const saveShapes = async () => {
    if (!shapesDirty) return;
    setErr('');
    setBusy('/api/save-shapes');
    try {
      const source = shapesRef.current;
      const inputFormat = shapesFmtRef.current;
      downloadText(source, formatFilename(shapesName, inputFormat));
      setShapesDirty(false);
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy('');
    }
  };

  const downloadReport = () => {
    const report = (tab === 'shacl'
      ? results.shacl?.report ?? results.shacl
      : results.validate?.report ?? results.validate);
    if (!report) return;
    downloadText(JSON.stringify(report, null, 2), 'validation-report.json', 'application/json;charset=utf-8');
  };

  useEffect(() => {
    let controller = null;
    const timer = setTimeout(() => {
      const seq = ++syncSeq.current;
      controller = new AbortController();
      // Re-run every backend action after a small debounce so typing updates
      // inspect/validate/convert/visualize results in near real time.
      setBusy('sync');
      setErr('');

      const tasks = [
        ['inspect', () => requestInspectPayload(controller.signal)],
        ['validate', () => requestPayload('/api/validate-text', buildValidationBody(code, inFmt), false, controller.signal)],
        ['shacl', () => requestPayload('/api/validate-text-detail', buildValidationBody(code, inFmt), false, controller.signal)],
        ['convert', () => requestPayload('/api/convert-text', { content: code, from: inFmt, to: outFmt }, true, controller.signal)],
        ['visualize', () => requestPayload('/api/visualize-text', { content: code, format: inFmt }, false, controller.signal)],
      ];

      Promise.allSettled(tasks.map(async ([tid, run]) => {
        const payload = await run();
        return { tid, payload };
      })).then(settled => {
        if (seq !== syncSeq.current) return;

        const nextResults = {};
        const errors = [];
        let nextVisualize = null;

        settled.forEach((entry, idx) => {
          const [tid] = tasks[idx];
          if (entry.status === 'fulfilled') {
            nextResults[tid] = entry.value.payload;
            if (tid === 'visualize') {
              nextVisualize = entry.value.payload;
            }
          } else {
            nextResults[tid] = null;
            errors.push(entry.reason instanceof Error ? entry.reason.message : String(entry.reason));
            if (tid === 'visualize') {
              nextVisualize = null;
            }
          }
        });

        setResults(prev => ({ ...prev, ...nextResults }));
        setVisualizeData(nextVisualize);
        setErr(errors[0] || '');
        setBusy('');
      }).catch(() => {
        if (seq !== syncSeq.current) return;
        setBusy('');
      });
    }, 250);

    return () => {
      controller?.abort();
      syncSeq.current += 1;
      clearTimeout(timer);
    };
  }, [code, inFmt, outFmt, shapesCode, shapesFmt]);

  const loadFile = (file) => {
    setFname(file.name);
    const reader = new FileReader();
    reader.onload = (e) => {
      setCode(e.target.result);
      setDirty(false);
      setErr('');
    };
    reader.readAsText(file);
    setInFmt(formatFromFilename(file.name));
  };

  const loadShapesFile = (file) => {
    setShapesName(file.name);
    const reader = new FileReader();
    reader.onload = (e) => {
      setShapesCode(e.target.result);
      setShapesDirty(false);
      setErr('');
    };
    reader.readAsText(file);
    setShapesFmt(formatFromFilename(file.name));
  };

  const cur = tab === 'shacl' ? results.shacl : results[tab];
  const validateReport = results.validate?.report ?? results.validate ?? null;
  const highlightedLines = validateReport?.issues
    ?.map(issue => issue.line)
    .filter(Boolean) ?? [];
  const apiLabel = API_URL.replace(/^https?:\/\//, '');

  const beginResize = (event) => {
    event.preventDefault();
    const startX = event.clientX;
    const startWidth = sidebarWidth;
    document.body.classList.add('is-resizing-sidebar');

    const onMove = (moveEvent) => {
      const nextWidth = Math.min(680, Math.max(260, startWidth + moveEvent.clientX - startX));
      setSidebarWidth(nextWidth);
    };

    const stopResize = () => {
      document.body.classList.remove('is-resizing-sidebar');
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', stopResize);
    };

    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', stopResize);
  };

  const beginShapesResize = (event) => {
    event.preventDefault();
    const startY = event.clientY;
    const startHeight = shapesHeight;
    document.body.classList.add('is-resizing-shapes');

    const onMove = (moveEvent) => {
      const sidebarEl = document.querySelector('.sidebar');
      const sidebarBounds = sidebarEl?.getBoundingClientRect();
      const maxHeight = Math.max(140, (sidebarBounds?.height || 0) - 150);
      const nextHeight = Math.min(maxHeight, Math.max(120, startHeight + (startY - moveEvent.clientY)));
      setShapesHeight(nextHeight);
    };

    const stopResize = () => {
      document.body.classList.remove('is-resizing-shapes');
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', stopResize);
    };

    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', stopResize);
  };

  return (
    <div className="app">
      <header className="hdr">
        <div className="hdr-l">
          <span className="logo" aria-hidden="true">🕸️</span>
          <h1>LOD Workbench</h1>
          <span className="badge">RDF</span>
        </div>
        <div className="hdr-r">
          <span className={`led ${busy ? 'on' : ''}`} />
          <span>{busy ? 'Processing…' : `Connected · ${apiLabel}`}</span>
        </div>
      </header>

      <div className="layout">
        <aside className="sidebar" style={{ width: `${sidebarWidth}px` }}>
          <div className="sidebar-tools">
            <div className="tool-group">
              <label className="lbl">Input
                <select value={inFmt} onChange={e => setInFmt(e.target.value)} aria-label="Input format">
                  {FORMATS.map(x => <option key={x.value} value={x.value}>{x.label}</option>)}
                </select>
              </label>
            </div>
            <div className="tool-group">
              <button className="btn-ghost" onClick={() => {
                setCode(SAMPLE);
                setShapesCode(SHAPES_SAMPLE);
                setFname('');
                setShapesName('shapes.ttl');
                setDirty(false);
                setShapesDirty(false);
                setErr('');
              }} aria-label="Reset sample">
                <span aria-hidden="true">↺</span> Reset
              </button>
              <button className="btn-ghost" onClick={saveCurrent} disabled={!dirty || !!busy} aria-label="Save current RDF">
                <span aria-hidden="true">💾</span> Save
              </button>
              <label className="btn-ghost">
                <span aria-hidden="true">📂</span> Open
                <input type="file" accept=".ttl,.nt,.jsonld,.json,.rdf" hidden onChange={e => e.target.files[0] && loadFile(e.target.files[0])} />
              </label>
            </div>
            {fname && <div className="file-tag">{fname}</div>}
          </div>
          <div className="editor-stack">
            <CodeEditor
              value={code}
              format={inFmt}
              highlightLines={highlightedLines}
              activeLine={activeLine}
              onLineClick={setActiveLine}
              onChange={e => {
                setCode(e.target.value);
                setDirty(true);
              }}
              spellCheck={false}
              aria-label="RDF input"
              placeholder="Paste RDF data here…"
            />
            <div
              className="shapes-splitter"
              role="separator"
              aria-orientation="horizontal"
              aria-label="Resize SHACL shapes editor"
              onMouseDown={beginShapesResize}
            />
            <section className="shapes-pane" style={{ height: `${shapesHeight}px` }} aria-label="SHACL shapes editor">
              <div className="shapes-head">
                <strong>SHACL shapes</strong>
                <div className="shapes-tools">
                  <label className="lbl">Format
                    <select value={shapesFmt} onChange={e => setShapesFmt(e.target.value)} aria-label="Shapes format">
                      {FORMATS.map(x => <option key={x.value} value={x.value}>{x.label}</option>)}
                    </select>
                  </label>
                  <button className="btn-ghost" onClick={saveShapes} disabled={!shapesDirty || !!busy} aria-label="Save SHACL shapes">
                    <span aria-hidden="true">💾</span> Save
                  </button>
                  <label className="btn-ghost">
                    <span aria-hidden="true">📂</span> Open
                    <input type="file" accept=".ttl,.nt,.jsonld,.json,.rdf" hidden onChange={e => e.target.files[0] && loadShapesFile(e.target.files[0])} />
                  </label>
                </div>
              </div>
              <textarea
                className="shapes-textarea"
                value={shapesCode}
                onChange={e => {
                  setShapesCode(e.target.value);
                  setShapesDirty(true);
                }}
                spellCheck={false}
                placeholder="Paste SHACL shapes here. Leave empty for syntax-only validation."
              />
            </section>
          </div>
        </aside>
        <div
          className="splitter"
          role="separator"
          aria-orientation="vertical"
          aria-label="Resize RDF editor"
          onMouseDown={beginResize}
        />

        <main className="main">
          <nav className="tnav" aria-label="Workbench views">
            {['inspect', 'validate', 'shacl', 'convert', 'visualize'].map(t => (
              <button
                key={t}
                className={`tn ${tab === t ? 'sel' : ''}`}
                onClick={() => setTab(t)}
                disabled={!!busy}
                aria-pressed={tab === t}
              >
                {{ inspect: '🔍 Inspect', validate: '✅ Validate', shacl: '🧩 SHACL', convert: '🔄 Convert', visualize: '📊 Visualize' }[t]}
              </button>
            ))}
          </nav>

          <div className="content">
            {err && <div className="err-msg" role="alert" aria-live="assertive">⚠️ {err}</div>}
            {busy && <div className="busy-msg" role="status" aria-live="polite"><span className="spin" /> {busy === 'sync' ? 'Syncing live results…' : 'Processing…'}</div>}
            {!busy && !err && tab === 'inspect' && <InspectPanel d={cur} />}
            {!busy && !err && tab === 'validate' && (
              <ValidatePanel
                d={cur}
                activeLine={activeLine}
                onLocateLine={setActiveLine}
                onDownloadReport={downloadReport}
                title="Syntax validation"
                variant="syntax"
              />
            )}
            {!busy && !err && tab === 'shacl' && (
              <ValidatePanel
                d={cur}
                activeLine={activeLine}
                onLocateLine={setActiveLine}
                onDownloadReport={downloadReport}
                title="SHACL report"
                variant="shacl"
              />
            )}
            {!busy && !err && tab === 'convert' && <div className="panel"><ConvertPanel d={cur} /></div>}
            {!busy && !err && tab === 'visualize' && (
              <div className="panel visualize-panel">
                {visualizeData ? (
                  <VisualizePanel d={visualizeData} />
                ) : <VisualizePanel d={null} />}
              </div>
            )}
            {!busy && !err && !cur && <p className="muted">Choose an action below to process your RDF.</p>}

            <div className="act-bar">
              <button className="act-btn" onClick={() => runAction('inspect', '/api/inspect-text', (c, f) => ({ content: c, format: f }))} disabled={!!busy}>
                {busy === '/api/inspect-text' ? '⋯' : '🔍'} Inspect
              </button>
              <button className="act-btn" onClick={() => runAction('validate', '/api/validate-text', (c, f) => buildValidationBody(c, f))} disabled={!!busy}>
                {busy === '/api/validate-text' ? '⋯' : '✅'} Validate
              </button>
              <button className="act-btn" onClick={() => setTab('shacl')} disabled={!!busy}>
                🧩 SHACL
              </button>
              <span className="act-group">
                <select value={outFmt} onChange={e => setOutFmt(e.target.value)} aria-label="Output format">
                  {FORMATS.map(x => <option key={x.value} value={x.value}>{x.label}</option>)}
                </select>
                <button className="act-btn" onClick={() => runAction('convert', '/api/convert-text', (c, f, t) => ({ content: c, from: f, to: t }), true)} disabled={!!busy}>
                  {busy === '/api/convert-text' ? '⋯' : '🔄'} Convert
                </button>
              </span>
              <button className="act-btn" onClick={() => runAction('visualize', '/api/visualize-text', (c, f) => ({ content: c, format: f }))} disabled={!!busy}>
                {busy === '/api/visualize-text' ? '⋯' : '📊'} Visualize
              </button>
            </div>
          </div>
        </main>
      </div>
    </div>
  );
}

createRoot(document.getElementById('root')).render(<App />);
