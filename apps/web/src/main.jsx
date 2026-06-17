import React, { useEffect, useRef, useState } from 'react';
import { createRoot } from 'react-dom/client';
import { GraphViewer } from './components/GraphViewer';
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
  const cards = [
    { l: 'Triples', v: d.triples }, { l: 'Subjects', v: d.subjects }, { l: 'Predicates', v: d.predicates },
    { l: 'Objects', v: d.objects }, { l: 'IRIs', v: d.iris }, { l: 'Literals', v: d.literals },
    { l: 'Blank', v: d.blank_nodes }, { l: 'Classes', v: d.classes }, { l: 'Properties', v: d.properties },
  ];
  return (
    <div className="panel">
      <div className="card-grid">
        {cards.map(c => (
          <div key={c.l} className="stat-card">
            <span className="stat-val">{c.v}</span>
            <span className="stat-lbl">{c.l}</span>
          </div>
        ))}
      </div>
      {d.class_distribution && <MiniTable title="Class distribution" data={d.class_distribution} />}
      {d.property_distribution && <MiniTable title="Property distribution (top 6)" data={d.property_distribution} />}
      {d.prefixes && <div className="section"><h4>Prefixes</h4><pre className="pref">{JSON.stringify(d.prefixes, null, 2)}</pre></div>}
    </div>
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

function ValidatePanel({ d }) {
  if (!d) return null;
  return (
    <div className="panel">
      <div className={`banner ${d.conforms ? 'ok' : 'fail'}`}>{d.conforms ? '✓ Conforms' : '✗ Does not conform'}</div>
      {d.issues?.length > 0 ? (
        <table className="tbl">
          <thead><tr><th>Sev</th><th>Line</th><th>Message</th></tr></thead>
          <tbody>
            {d.issues.map((x, i) => (
              <tr key={i}><td><span className={`sev ${x.severity.toLowerCase()}`}>{x.severity}</span></td><td>{x.line ?? '—'}</td><td>{x.message}</td></tr>
            ))}
          </tbody>
        </table>
      ) : <p className="muted">No validation issues found.</p>}
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
  const [inFmt, setInFmt] = useState('turtle');
  const [outFmt, setOutFmt] = useState('json-ld');
  const [tab, setTab] = useState('inspect');
  const [busy, setBusy] = useState('');
  const [err, setErr] = useState('');
  const [results, setResults] = useState({});
  const [fname, setFname] = useState('');
  const [visualizeData, setVisualizeData] = useState(null);
  const [dirty, setDirty] = useState(false);
  const syncSeq = useRef(0);

  const codeRef = useRef(code);
  const inRef = useRef(inFmt);
  const outRef = useRef(outFmt);
  useEffect(() => { codeRef.current = code; }, [code]);
  useEffect(() => { inRef.current = inFmt; }, [inFmt]);
  useEffect(() => { outRef.current = outFmt; }, [outFmt]);

  const requestPayload = async (endpoint, body, raw, signal) => {
    const r = await apiFetch(endpoint, body, { signal });
    if (!r.ok) {
      throw new Error(r.error);
    }
    return raw ? { ...r.data, _raw: r.data.output } : r.data;
  };

  const runAction = async (tid, endpoint, buildBody, raw) => {
    setErr('');
    setBusy(endpoint);
    try {
      const payload = await requestPayload(endpoint, buildBody(codeRef.current, inRef.current, outRef.current), raw);
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
        ['inspect', '/api/inspect-text', { content: code, format: inFmt }, false],
        ['validate', '/api/validate-text', { content: code, format: inFmt }, false],
        ['convert', '/api/convert-text', { content: code, from: inFmt, to: outFmt }, true],
        ['visualize', '/api/visualize-text', { content: code, format: inFmt }, false],
      ];

      Promise.allSettled(tasks.map(async ([tid, endpoint, body, raw]) => {
        const payload = await requestPayload(endpoint, body, raw, controller.signal);
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
  }, [code, inFmt, outFmt]);

  const loadFile = (file) => {
    setFname(file.name);
    const reader = new FileReader();
    reader.onload = (e) => {
      setCode(e.target.result);
      setDirty(false);
      setErr('');
    };
    reader.readAsText(file);
    const n = file.name.toLowerCase();
    setInFmt(n.endsWith('.nt') ? 'n-triples' : (n.endsWith('.jsonld') || n.endsWith('.json')) ? 'json-ld' : 'turtle');
  };

  const cur = results[tab];
  const apiLabel = API_URL.replace(/^https?:\/\//, '');

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
        <aside className="sidebar">
          <div className="sidebar-tools">
            <div className="tool-group">
              <label className="lbl">Input
                <select value={inFmt} onChange={e => setInFmt(e.target.value)} aria-label="Input format">
                  {FORMATS.map(x => <option key={x.value} value={x.value}>{x.label}</option>)}
                </select>
              </label>
            </div>
            <div className="tool-group">
              <button className="btn-ghost" onClick={() => { setCode(SAMPLE); setFname(''); setDirty(false); setErr(''); }} aria-label="Reset sample">
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
          <textarea
            value={code}
            onChange={e => {
              setCode(e.target.value);
              setDirty(true);
            }}
            spellCheck={false}
            aria-label="RDF input"
            placeholder="Paste RDF data here…"
          />
        </aside>

        <main className="main">
          <nav className="tnav" aria-label="Workbench views">
            {['inspect', 'validate', 'convert', 'visualize'].map(t => (
              <button
                key={t}
                className={`tn ${tab === t ? 'sel' : ''}`}
                onClick={() => setTab(t)}
                disabled={!!busy}
                aria-pressed={tab === t}
              >
                {{ inspect: '🔍 Inspect', validate: '✅ Validate', convert: '🔄 Convert', visualize: '📊 Visualize' }[t]}
              </button>
            ))}
          </nav>

          <div className="content">
            {err && <div className="err-msg" role="alert" aria-live="assertive">⚠️ {err}</div>}
            {busy && <div className="busy-msg" role="status" aria-live="polite"><span className="spin" /> {busy === 'sync' ? 'Syncing live results…' : 'Processing…'}</div>}
            {!busy && !err && tab === 'inspect' && <InspectPanel d={cur} />}
            {!busy && !err && tab === 'validate' && <ValidatePanel d={cur} />}
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
              <button className="act-btn" onClick={() => runAction('validate', '/api/validate-text', (c, f) => ({ content: c, format: f }))} disabled={!!busy}>
                {busy === '/api/validate-text' ? '⋯' : '✅'} Validate
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
