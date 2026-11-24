import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Archive, ArrowDownToLine, Check, ChevronDown, ChevronRight, FileImage, FolderOpen, Grid3X3, Layers3, Maximize, PackageOpen, Plus, Settings2, Sparkles, Trash2, UploadCloud } from "lucide-react";
import { loadSprite, saveAtlas, saveBlob, type Sprite, type Frame } from "./atlas";
import { createAtlas, optimizeAtlas, type Recommendation } from "./wasm-atlas";

const DEMO = [
  {name:"fern", color:"#68bb7d", shape:"leaf"},
  {name:"mushroom", color:"#e9a66a", shape:"mushroom"},
  {name:"crystal", color:"#86c9f6", shape:"crystal"},
  {name:"stone", color:"#a5acc4", shape:"stone"},
  {name:"flower", color:"#d8a0f2", shape:"flower"},
  {name:"tree", color:"#66a986", shape:"tree"}
];
function demoSprites(): Promise<Sprite[]> {
  const files = DEMO.map(({name,color,shape}) => {
    const c = document.createElement("canvas"); c.width = 48; c.height = 48;
    const ctx = c.getContext("2d")!;
    ctx.fillStyle = color;
    if (shape === "leaf" || shape === "tree") {
      ctx.beginPath(); ctx.ellipse(24, 20, shape === "tree" ? 17 : 9, 18, Math.PI/5, 0, Math.PI*2); ctx.fill();
      ctx.fillStyle = "#795842"; ctx.fillRect(22, 29, 5, 12);
    } else if (shape === "mushroom") {
      ctx.fillRect(20, 24, 9, 18); ctx.beginPath();ctx.ellipse(24, 23, 16, 10, 0, Math.PI, 0);ctx.fill();
    } else if (shape === "crystal") {
      ctx.beginPath();ctx.moveTo(24,5);ctx.lineTo(40,23);ctx.lineTo(26,43);ctx.lineTo(9,23);ctx.closePath();ctx.fill();
    } else if (shape === "stone") {
      ctx.beginPath();ctx.moveTo(10,34);ctx.lineTo(15,15);ctx.lineTo(34,13);ctx.lineTo(41,34);ctx.closePath();ctx.fill();
    } else {
      for(let i=0;i<5;i++){ctx.beginPath();const a=i*Math.PI*2/5;ctx.ellipse(24+Math.cos(a)*10,22+Math.sin(a)*10,8,6,a,0,Math.PI*2);ctx.fill();}
      ctx.fillStyle="#f7cf7f";ctx.beginPath();ctx.arc(24,22,6,0,Math.PI*2);ctx.fill();
    }
    return new Promise<File>(resolve => c.toBlob(b => resolve(new File([b!], `${name}.png`, {type:"image/png"})), "image/png"));
  });
  return Promise.all(files).then(items => Promise.all(items.map(loadSprite)));
}

function FileDrop({ onFiles, children }: { onFiles: (files: FileList | File[]) => void; children: React.ReactNode }) {
  const [dragging, setDragging] = useState(false);
  return <div className={`work-area ${dragging ? "dragging" : ""}`}
    onDragOver={e => {e.preventDefault();setDragging(true);}}
    onDragLeave={e => {e.preventDefault();setDragging(false);}}
    onDrop={e => {e.preventDefault();setDragging(false);onFiles(e.dataTransfer.files);}}>{children}</div>;
}

export function App() {
  const [sprites, setSprites] = useState<Sprite[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [width, setWidth] = useState(512);
  const [height, setHeight] = useState(512);
  const [padding, setPadding] = useState(2);
  const [trim, setTrim] = useState(true);
  const [showGrid, setShowGrid] = useState(true);
  const [zoom, setZoom] = useState(100);
  const [error, setError] = useState("");
  const [section, setSection] = useState<"assets" | "atlas">("atlas");
  const input = useRef<HTMLInputElement>(null);
  const importFiles = useCallback(async (files: FileList | File[]) => {
    setError("");
    const incoming = Array.from(files).filter(f => f.type === "image/png" || f.name.toLowerCase().endsWith(".png"));
    if (!incoming.length) {setError("Select PNG images to import.");return;}
    const results = await Promise.allSettled(incoming.map(loadSprite));
    const loaded = results.filter((r): r is PromiseFulfilledResult<Sprite> => r.status === "fulfilled").map(r => r.value);
    const failures = results.length - loaded.length;
    setSprites(old => {
      const names = new Set(old.map(s => s.name));
      const additions = loaded.filter(s => !names.has(s.name));
      return [...old, ...additions];
    });
    if (failures) setError(`${failures} image(s) could not be loaded.`);
  }, []);
  const [recommendation, setRecommendation] = useState<Recommendation | null>(null);
  const [optimizing, setOptimizing] = useState(false);
  useEffect(() => { setRecommendation(null); }, [sprites, padding, trim, width, height]);
  const autoOptimize = async () => {
    setOptimizing(true);
    setError("");
    try {
      const best = await optimizeAtlas(sprites, padding, trim);
      setRecommendation(best);
    } catch (reason) {
      setError(String(reason));
    } finally {
      setOptimizing(false);
    }
  };
  const [page, setPage] = useState(0);
  const [result, setResult] = useState<{ canvases: HTMLCanvasElement[]; frames: Frame[]; error: string } | null>(null);
  useEffect(() => {
    let cancelled = false;
    if (!sprites.length) { setResult(null); return; }
    setResult(null);
    setPage(0);
    void createAtlas(sprites, width, height, padding, trim).then(
      atlas => { if (!cancelled) setResult({ ...atlas, error: "" }); },
      reason => { if (!cancelled) setResult({ canvases: [], frames: [], error: String(reason) }); }
    );
    return () => { cancelled = true; };
  }, [sprites, width, height, padding, trim]);
  const preview = useMemo(() => result?.canvases[page]?.toDataURL("image/png") ?? null, [result, page]);
  const pageCount = result?.canvases.length ?? 0;
  const pageName = (index: number) => pageCount === 1 ? "hewn-atlas.png" : `hewn-atlas-${index + 1}.png`;
  const utilization = pageCount ? (100 * (result?.frames.reduce((sum, frame) => sum + frame.width * frame.height, 0) ?? 0) / (pageCount * width * height)).toFixed(1) : "0";
  const active = sprites.find(s => s.name === selected);
  const exportMetadata = () => {
    if (!result || result.error) return;
    saveBlob(new Blob([JSON.stringify({
      generator: "Sindri: Hewn", version: 2,
      pages: result.canvases.map((_, index) => ({ index, image: pageName(index) })), width, height, padding, trim, frames: result.frames
    }, null, 2)], {type:"application/json"}), "hewn-atlas.json");
  };

  return <div className="app">
    <aside className="rail">
      <div className="brand-mark"><Layers3 size={24}/></div>
      <button title="Atlas workbench" className={section==="atlas"?"active":""} onClick={()=>setSection("atlas")}><Grid3X3 size={20}/></button>
      <button title="Imported assets" className={section==="assets"?"active":""} onClick={()=>setSection("assets")}><FolderOpen size={20}/></button>
      <div className="rail-bottom"><div className="avatar">S</div></div>
    </aside>
    <aside className="sidebar">
      <div className="sidebar-top">
        <div className="product"><span>SINDRI <span className="slash">/</span> </span><strong>HEWN</strong></div>
        <div className="project-name"><span className="dot"/> Untitled collection <ChevronDown size={14}/></div>
      </div>
      <div className="panel-label"><span>ASSET LIBRARY</span><button title="Import PNG images" onClick={()=>input.current?.click()}><Plus size={17}/></button></div>
      <div className="asset-group"><ChevronDown size={14}/> <FolderOpen size={15}/> Imported assets <span>{sprites.length}</span></div>
      <div className="asset-list">
        {sprites.map(sprite=><button key={sprite.name} className={selected===sprite.name?"selected":""} onClick={()=>setSelected(sprite.name)}>
          <img src={sprite.preview} alt=""/><span>{sprite.name}</span><small>{sprite.width}×{sprite.height}</small>
        </button>)}
        {!sprites.length && <p className="empty-note">Your imported PNGs will appear here.</p>}
      </div>
      <div className="sidebar-footer"><span><Check size={14}/> Local workspace</span><small>No uploads. No AI.</small></div>
    </aside>
    <main className="main">
      <header className="toolbar"><div className="breadcrumbs">Workspace <ChevronRight size={14}/> <strong>{section==="atlas"?"Atlas builder":"Asset library"}</strong></div>
        <div className="toolbar-actions">
          <button className="ghost" onClick={()=>{setSprites([]);setSelected(null);setError("");}} title="Clear workspace"><Trash2 size={16}/><span>Clear</span></button>
          <button className="ghost" onClick={()=>input.current?.click()}><Plus size={16}/> Import</button>
          <button className="primary" disabled={!pageCount} onClick={()=>result?.canvases.forEach((canvas, index)=>saveAtlas(canvas, pageName(index)))}><ArrowDownToLine size={16}/> Export PNGs</button>
        </div>
      </header>
      <FileDrop onFiles={importFiles}><div className="canvas-header"><div><span className="eyebrow">WORKBENCH / {section.toUpperCase()}</span><h1>{section==="atlas"?"Texture atlas":"Your assets"}</h1><p>{section==="atlas"?"Arrange, inspect, and export your sprites in one place.":"All imported sprites, ready for your next creation."}</p></div>
        <div className="canvas-status"><span className="status-dot"/> {sprites.length} assets loaded</div></div>
        {error && <div className="error" role="alert">{error}</div>}
        {result?.error && <div className="error" role="alert">{result.error}</div>}
        {section === "assets" && sprites.length > 0 ? <div className="asset-cards">{sprites.map(s=><button key={s.name} onClick={()=>setSelected(s.name)} className={selected===s.name?"chosen":""}><div className="asset-thumb"><img src={s.preview} alt={s.name}/></div><strong>{s.name}</strong><small>{s.width} × {s.height} px</small></button>)}</div> :
        section === "atlas" && preview ? <div className="preview-wrap"><div className="preview-top"><span><Grid3X3 size={15}/> ATLAS PREVIEW</span><span>{width} × {height} px</span><label>Page <select value={page} onChange={e=>setPage(Number(e.target.value))}>{result?.canvases.map((_, index)=><option key={index} value={index}>{index+1} / {pageCount}</option>)}</select></label></div><div className="preview-canvas"><div className={showGrid?"atlas-preview checker":"atlas-preview"} style={{width:Math.min(100,zoom)+"%",aspectRatio:`${width}/${height}`}}><img src={preview} alt="Generated texture atlas" /></div></div></div> :
        <div className="drop-zone"><div className="drop-art"><div className="drop-art-secondary"><FileImage size={30}/></div><div className="drop-art-primary"><PackageOpen size={36}/></div></div><h2>Every asset in its place.</h2><p>Drop your PNG sprites here, or import a few files to start building your first atlas.</p><button className="primary large" onClick={()=>input.current?.click()}><UploadCloud size={17}/> Import PNG files</button><button className="text-button" onClick={async()=>{setError("");setSprites(await demoSprites());setSelected(null);}}>Or try with sample assets <ChevronRight size={15}/></button></div>}
        <div className="bottom-bar"><span><span className="status-dot"/> Ready</span><span>{sprites.length} sprites <span className="sep">·</span> {result?.frames.length??0} packed <span className="sep">·</span> PNG</span></div>
      </FileDrop>
    </main>
    <aside className="inspector">
      <div className="inspector-title"><Settings2 size={17}/> Inspector</div>
      {active ? <div className="inspector-section"><div className="label">SELECTED SPRITE</div><div className="selected-sprite"><img src={active.preview} alt={active.name}/></div><strong>{active.name}</strong><div className="stat"><span>Original size</span><strong>{active.width} × {active.height}</strong></div><div className="stat"><span>Trimmed size</span><strong>{active.trimWidth} × {active.trimHeight}</strong></div><div className="stat"><span>Offset</span><strong>{active.offsetX}, {active.offsetY}</strong></div></div> : null}
      <div className="inspector-section"><div className="label">ATLAS SETTINGS</div>
      <button className="secondary-button" disabled={!sprites.length || optimizing} onClick={()=>void autoOptimize()}><Sparkles size={15}/> {optimizing ? "Optimizing…" : "Auto Optimize"}</button>
      {recommendation && <div role="status"><p>Recommended: {recommendation.width} × {recommendation.height}, {recommendation.pages} page(s), {(recommendation.bytes / 1024).toFixed(0)} KiB RGBA8. Lowest memory across 36 tested sizes; fewer pages break ties.</p><button className="secondary-button" onClick={()=>{setWidth(recommendation.width);setHeight(recommendation.height);setRecommendation(null);}}>Apply recommendation</button></div>}<label className="field">Width<select value={width} onChange={e=>setWidth(Number(e.target.value))}>{[128,256,512,1024,2048,4096].map(n=><option key={n} value={n}>{n} px</option>)}</select></label><label className="field">Height<select value={height} onChange={e=>setHeight(Number(e.target.value))}>{[128,256,512,1024,2048,4096].map(n=><option key={n} value={n}>{n} px</option>)}</select></label><label className="field">Padding<input type="number" min="0" max="64" value={padding} onChange={e=>setPadding(Math.min(64,Math.max(0,Number(e.target.value)||0)))}/></label>
        <label className="toggle-row"><span>Trim transparency<small>Remove unused transparent edges</small></span><input type="checkbox" checked={trim} onChange={e=>setTrim(e.target.checked)}/></label>
        <label className="toggle-row"><span>Checkerboard<small>Show transparent background</small></span><input type="checkbox" checked={showGrid} onChange={e=>setShowGrid(e.target.checked)}/></label>
      </div>
      <div className="inspector-section"><div className="label">VIEW</div><div className="stat"><span>Atlas pages</span><strong>{pageCount}</strong></div><div className="stat"><span>Frame coverage</span><strong>{utilization}%</strong></div><label className="field">Preview scale<select value={zoom} onChange={e=>setZoom(Number(e.target.value))}>{[25,50,75,100].map(n=><option key={n} value={n}>{n}%</option>)}</select></label></div>
      <div className="inspector-spacer"/>
      <div className="export-panel"><div className="export-title"><Archive size={17}/> Export resources</div><p>Your original images stay untouched.</p><button className="secondary-button" disabled={!pageCount} onClick={exportMetadata}><ArrowDownToLine size={15}/> Export JSON metadata</button></div>
    </aside>
    <input ref={input} type="file" multiple accept=".png,image/png" hidden onChange={e=>{if(e.target.files)void importFiles(e.target.files);e.target.value="";}}/>
  </div>;
}
