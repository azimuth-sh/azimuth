'use client';

import { useEffect, useLayoutEffect, useRef, useState, useSyncExternalStore, type ReactNode } from 'react';

const compactQuery = '(max-width: 1000px)';
function subscribe(listener: () => void) {
  const media = window.matchMedia(compactQuery);
  media.addEventListener('change', listener);
  return () => media.removeEventListener('change', listener);
}
function compactSnapshot() { return window.matchMedia(compactQuery).matches; }
function serverSnapshot() { return false; }

export function ModelWorkspace({ navigation, children, breadcrumb, initialPinned, selection }: {
  navigation: ReactNode; children: ReactNode; breadcrumb: ReactNode;
  initialPinned: boolean; selection: string;
}) {
  const compact = useSyncExternalStore(subscribe, compactSnapshot, serverSnapshot);
  const [pinned, setPinned] = useState(initialPinned);
  const [overlayOpen, setOverlayOpen] = useState(false);
  const overlay = useRef<HTMLDialogElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const inline = !compact && pinned;

  useEffect(() => { setOverlayOpen(false); }, [selection]);
  useLayoutEffect(() => {
    const dialog = overlay.current;
    if (!dialog) return;
    if (!overlayOpen || inline) { if (dialog.open) dialog.close(); return; }
    function position() {
      const bounds = trigger.current?.getBoundingClientRect();
      if (!bounds || !dialog) return;
      const margin = 12;
      const width = Math.min(320, window.innerWidth - margin * 2);
      const below = window.innerHeight - bounds.bottom - margin - 8;
      const above = bounds.top - margin - 8;
      const flip = below < 280 && above > below;
      const height = Math.max(60, Math.min(480, flip ? above : below));
      dialog.style.width = `${width}px`;
      dialog.style.maxHeight = `${height}px`;
      dialog.style.left = `${Math.max(margin, Math.min(bounds.left, window.innerWidth - width - margin))}px`;
      dialog.style.top = flip ? 'auto' : `${Math.max(margin, bounds.bottom + 8)}px`;
      dialog.style.bottom = flip ? `${Math.max(margin, window.innerHeight - bounds.top + 8)}px` : 'auto';
    }
    position();
    if (!dialog.open) dialog.showModal();
    window.addEventListener('resize', position);
    window.addEventListener('scroll', position, true);
    return () => {
      window.removeEventListener('resize', position);
      window.removeEventListener('scroll', position, true);
    };
  }, [overlayOpen, inline]);

  function pin(value: boolean) {
    setPinned(value);
    document.cookie = `assurance-modules-pinned=${value ? '1' : '0'}; Path=/; Max-Age=31536000; SameSite=Lax${location.protocol === 'https:' ? '; Secure' : ''}`;
    setOverlayOpen(false);
    if (!value) trigger.current?.focus();
  }
  function toggleNavigation() {
    if (inline) pin(false);
    else setOverlayOpen(true);
  }
  const pinButton = <button type="button" className="modulePinButton" aria-pressed={pinned} aria-label={pinned ? 'Unpin modules' : 'Pin modules beside content'} onClick={() => pin(!pinned)}>{pinned ? 'Unpin' : 'Pin'}</button>;

  return <>
    <div className="modelWorkspaceToolbar"><button ref={trigger} type="button" className="modulesButton" aria-haspopup={inline ? undefined : "dialog"} aria-expanded={inline || overlayOpen} aria-controls={inline ? 'inline-modules' : 'module-overlay'} onClick={toggleNavigation}><svg viewBox="0 0 20 20" width="15" height="15" fill="none" aria-hidden="true"><rect x="2.5" y="3.5" width="15" height="13" rx="2" stroke="currentColor" strokeWidth="1.3" /><path d="M7.5 4v12" stroke="currentColor" strokeWidth="1.3" /></svg>Modules</button><div className="workspaceBreadcrumb">{breadcrumb}</div></div>
    <div className={`modelWorkspace${inline ? ' modelWorkspaceExpanded' : ' modelWorkspaceFull'}`}>
      {inline && <aside className="inlineModules" id="inline-modules"><div className="moduleNavigationActions"><span>Browse modules</span>{pinButton}<button type="button" className="moduleCloseButton" aria-label="Hide modules" onClick={toggleNavigation}>×</button></div>{navigation}</aside>}
      {children}
    </div>
    <dialog id="module-overlay" ref={overlay} className="moduleOverlay" aria-labelledby="module-overlay-title" onCancel={event => { event.preventDefault(); setOverlayOpen(false); }} onClose={() => setOverlayOpen(false)} onClick={event => {
      if (event.target !== overlay.current) return;
      const bounds = overlay.current.getBoundingClientRect();
      if (event.clientX < bounds.left || event.clientX > bounds.right || event.clientY < bounds.top || event.clientY > bounds.bottom) setOverlayOpen(false);
    }}>
      <div className="moduleOverlayHeading"><h2 id="module-overlay-title">Modules</h2>{!compact && pinButton}<button type="button" className="moduleCloseButton" aria-label="Close modules" onClick={() => setOverlayOpen(false)}>×</button></div>
      <div onClick={event => { if ((event.target as HTMLElement).closest('a')) setOverlayOpen(false); }}>{navigation}</div>
    </dialog>
  </>;
}
