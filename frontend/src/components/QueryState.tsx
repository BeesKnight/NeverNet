import type { ReactNode } from 'react'

type StateCardProps = {
  title: string
  detail: string
  action?: ReactNode
}

function StateCard({ title, detail, action }: StateCardProps) {
  return (
    <div className="page-shell">
      <section className="section-card state-card">
        <p className="eyebrow">{title}</p>
        <h2>{detail}</h2>
        {action ? <div className="form-actions">{action}</div> : null}
      </section>
    </div>
  )
}

export function LoadingState({ title = 'Loading', detail = 'Fetching the latest workspace view.' }: Partial<StateCardProps>) {
  return <StateCard title={title} detail={detail} />
}

export function ErrorState({
  title = 'Something failed',
  detail = 'Refresh the page or retry the request.',
  action,
}: StateCardProps) {
  return <StateCard title={title} detail={detail} action={action} />
}

export function InlineNotice({
  tone = 'neutral',
  children,
}: {
  tone?: 'neutral' | 'error' | 'success'
  children: ReactNode
}) {
  return <div className={`inline-notice ${tone}`}>{children}</div>
}
