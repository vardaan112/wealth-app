function App() {
  return (
    <div className="min-h-screen bg-slate-950 text-slate-100">
      <main className="mx-auto flex max-w-3xl flex-col gap-6 px-6 py-16">
        <p className="text-sm font-medium uppercase tracking-widest text-emerald-400">
          wealth-app
        </p>
        <h1 className="text-4xl font-semibold tracking-tight">
          Personal finance, scaffolded
        </h1>
        <p className="text-lg text-slate-400">
          React + Vite frontend, Rust + Axum + GraphQL API, and Postgres.
        </p>
        <div className="rounded-lg border border-slate-800 bg-slate-900 p-4 text-left text-sm text-slate-300">
          <p>API health: <code className="text-emerald-300">GET /health</code></p>
          <p>GraphQL: <code className="text-emerald-300">POST /graphql</code></p>
        </div>
      </main>
    </div>
  )
}

export default App
