import React, { useCallback, useEffect, useMemo, useRef } from 'react';

function isTurtleTerminator(ch, state) {
  return ch === '.'
    && !state.inString
    && !state.inIri
    && state.bracketDepth === 0
    && state.parenDepth === 0;
}

function buildTurtleRows(text) {
  const lines = (text || '').split('\n');
  const rows = [];
  const state = {
    inString: false,
    inIri: false,
    escaped: false,
    bracketDepth: 0,
    parenDepth: 0,
  };
  let statementIndex = 1;
  let inStatement = false;

  lines.forEach((line, idx) => {
    const trimmed = line.trim();
    const startsStatement = !inStatement && trimmed.length > 0;
    let lineComment = false;
    let sawTerminator = false;

    for (const ch of line) {
      if (lineComment) continue;
      if (ch === '#' && !state.inString && !state.inIri) {
        lineComment = true;
        continue;
      }

      switch (ch) {
        case '"':
          if (!state.inIri && !state.escaped) state.inString = !state.inString;
          break;
        case '<':
          if (!state.inString) state.inIri = true;
          break;
        case '>':
          if (state.inIri && !state.escaped) state.inIri = false;
          break;
        case '[':
          if (!state.inString && !state.inIri) state.bracketDepth += 1;
          break;
        case ']':
          if (!state.inString && !state.inIri) {
            state.bracketDepth = Math.max(0, state.bracketDepth - 1);
          }
          break;
        case '(':
          if (!state.inString && !state.inIri) state.parenDepth += 1;
          break;
        case ')':
          if (!state.inString && !state.inIri) {
            state.parenDepth = Math.max(0, state.parenDepth - 1);
          }
          break;
        case '\\':
          if (state.inString && !state.escaped) {
            state.escaped = true;
            continue;
          }
          break;
        default:
          break;
      }

      if (isTurtleTerminator(ch, state)) {
        sawTerminator = true;
      }

      if (ch !== '\\') state.escaped = false;
    }

    rows.push({
      sourceLine: idx + 1,
      label: startsStatement ? String(statementIndex) : '',
      statementIndex: trimmed ? statementIndex : 0,
    });

    if (trimmed.length > 0) {
      inStatement = !sawTerminator;
      if (sawTerminator) statementIndex += 1;
    }
  });

  return rows.length ? rows : [{ sourceLine: 1, label: '1', statementIndex: 1 }];
}

function buildRows(text, format) {
  if (format === 'turtle') return buildTurtleRows(text);
  const lines = (text || '').split('\n');
  return lines.map((_, idx) => ({
    sourceLine: idx + 1,
    label: String(idx + 1),
    statementIndex: idx + 1,
  }));
}

export default function CodeEditor({
  value,
  format,
  onChange,
  placeholder,
  spellCheck,
  'aria-label': ariaLabel,
  highlightLines = [],
  activeLine = null,
  onLineClick,
}) {
  const textareaRef = useRef(null);
  const gutterTrackRef = useRef(null);
  const zebraTrackRef = useRef(null);

  const rows = useMemo(() => buildRows(value, format), [value, format]);
  const highlightSet = new Set(highlightLines);
  const isActiveLine = (line) => activeLine === line;

  const syncScroll = useCallback(() => {
    const ta = textareaRef.current;
    if (!ta) return;
    const y = `translateY(${-ta.scrollTop}px)`;
    if (gutterTrackRef.current) gutterTrackRef.current.style.transform = y;
    if (zebraTrackRef.current) zebraTrackRef.current.style.transform = y;
  }, []);

  useEffect(() => {
    if (!activeLine) return;
    const ta = textareaRef.current;
    if (!ta) return;
    const styles = window.getComputedStyle(ta);
    const lineHeight = parseFloat(styles.lineHeight) || 16;
    const top = Math.max(0, (activeLine - 1) * lineHeight - lineHeight * 2);
    ta.scrollTop = top;
    syncScroll();
  }, [activeLine, syncScroll]);

  const handleKeyDown = useCallback((event) => {
    if (event.key !== 'Tab') return;
    event.preventDefault();
    const textarea = textareaRef.current;
    if (!textarea) return;
    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    const nextValue = `${value.slice(0, start)}  ${value.slice(end)}`;
    onChange({ target: { value: nextValue } });
    requestAnimationFrame(() => {
      textarea.selectionStart = start + 2;
      textarea.selectionEnd = start + 2;
    });
  }, [onChange, value]);

  return (
    <div className="editor-wrap">
      <div className="gutter" aria-hidden="true">
        <div className="editor-track" ref={gutterTrackRef}>
          {rows.map(row => (
            <div
              className={[
                'gutter-line',
                `row-${row.statementIndex % 2 === 0 ? 'even' : 'odd'}`,
                highlightSet.has(row.sourceLine) ? 'is-highlighted' : '',
                isActiveLine(row.sourceLine) ? 'is-active' : '',
              ].filter(Boolean).join(' ')}
              key={`gutter-${row.sourceLine}`}
              onClick={() => onLineClick?.(row.sourceLine)}
            >
              {row.label}
            </div>
          ))}
        </div>
      </div>
      <div className="textarea-wrap">
          <div className="zebra-layer" aria-hidden="true">
            <div className="editor-track" ref={zebraTrackRef}>
              {rows.map(row => (
                <div
                  className={[
                    'zebra-row',
                    `row-${row.statementIndex % 2 === 0 ? 'even' : 'odd'}`,
                    highlightSet.has(row.sourceLine) ? 'is-highlighted' : '',
                    isActiveLine(row.sourceLine) ? 'is-active' : '',
                  ].filter(Boolean).join(' ')}
                  key={`stripe-${row.sourceLine}`}
                />
              ))}
            </div>
          </div>
        <textarea
          ref={textareaRef}
          value={value}
          onChange={onChange}
          onKeyDown={handleKeyDown}
          onScroll={syncScroll}
          placeholder={placeholder}
          spellCheck={spellCheck}
          aria-label={ariaLabel}
          className="code-textarea"
          wrap="off"
        />
      </div>
    </div>
  );
}
