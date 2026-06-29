import { PlaceholderCard } from '../components/PlaceholderCard'

export function SettingsPage() {
  return (
    <div className="animate-rise mx-auto flex max-w-5xl flex-col gap-8 pt-6">
      <header>
        <h1 className="text-3xl font-semibold tracking-[-0.04em]">Settings</h1>
        <p className="mt-2 text-sm text-muted">
          Preferences and account configuration.
        </p>
      </header>
      <div className="grid gap-3 sm:grid-cols-2">
        <PlaceholderCard
          title="Profile"
          description="Name, email, and display preferences."
        />
        <PlaceholderCard
          title="Connected accounts"
          description="Link banks and brokerages."
        />
        <PlaceholderCard
          title="Currency & locale"
          description="USD · United States."
        />
        <PlaceholderCard
          title="Notifications"
          description="Alerts for bills and large transactions."
        />
      </div>
    </div>
  )
}
