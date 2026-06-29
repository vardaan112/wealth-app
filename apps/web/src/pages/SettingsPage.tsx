import { PlaceholderCard } from '../components/PlaceholderCard'

export function SettingsPage() {
  return (
    <div className="mx-auto flex max-w-4xl flex-col gap-6">
      <header>
        <h1 className="text-2xl font-semibold tracking-tight">Settings</h1>
        <p className="mt-1 text-sm text-muted">
          Preferences and account configuration.
        </p>
      </header>
      <div className="grid gap-4 sm:grid-cols-2">
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
