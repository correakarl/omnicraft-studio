import './app.css';
import App from './App.svelte';
import { mount } from 'svelte';

const target = document.getElementById('app');
if (!target) {
  console.error('Target #app not found in document!');
}

let app: any;
try {
  app = mount(App, {
    target: target!,
  });
} catch (err) {
  console.error('CRITICAL: Error mounting Svelte 5 app:', err);
  if (target) {
    target.innerHTML = `
      <div style="padding: 2rem; color: #ef4444; font-family: monospace; background: #0f111a; height: 100vh;">
        <h2 style="font-size: 1.25rem; font-weight: bold; margin-bottom: 1rem;">Omnicraft Launcher Studio — Error de Renderizado</h2>
        <pre style="white-space: pre-wrap; background: #181a27; padding: 1rem; border-radius: 8px;">${err instanceof Error ? err.stack || err.message : String(err)}</pre>
      </div>
    `;
  }
}

export default app;
