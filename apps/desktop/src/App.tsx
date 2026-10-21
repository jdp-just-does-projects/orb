import { createResource, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

interface AppInfo {
  version: string;
  data_dir: string;
  cache_dir: string;
}

export default function App() {
  const [info] = createResource(() => invoke<AppInfo>("app_info"));

  return (
    <main class="shell">
      <h1>orb</h1>
      <Show when={info()} fallback={<p>Loading…</p>}>
        {(i) => (
          <dl>
            <dt>Version</dt>
            <dd>{i().version}</dd>
            <dt>Data</dt>
            <dd>{i().data_dir}</dd>
            <dt>Cache</dt>
            <dd>{i().cache_dir}</dd>
          </dl>
        )}
      </Show>
    </main>
  );
}
