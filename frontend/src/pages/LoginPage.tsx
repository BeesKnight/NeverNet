import { useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'

import { ApiError } from '../api/client'
import { useAuth } from '../features/auth/auth-context'

export function LoginPage() {
  const navigate = useNavigate()
  const auth = useAuth()
  const [email, setEmail] = useState('demo@eventdesign.local')
  const [password, setPassword] = useState('password123')
  const [error, setError] = useState<string | null>(null)
  const [isSubmitting, setIsSubmitting] = useState(false)

  return (
    <div className="auth-shell">
      <section className="auth-panel">
        <p className="eyebrow">EventDesign</p>
        <h1>Run planning, reporting, and exports from one workspace.</h1>
        <p className="panel-copy">
          Sign in to manage categories, track event execution, and export reports.
        </p>
      </section>

      <section className="auth-card">
        <div className="section-header">
          <div>
            <p className="eyebrow">Welcome back</p>
            <h2>Sign in</h2>
          </div>
        </div>

        <form
          className="form-grid"
          onSubmit={async (event) => {
            event.preventDefault()
            setError(null)
            setIsSubmitting(true)

            try {
              await auth.login({ email, password })
              navigate('/dashboard')
            } catch (submissionError) {
              setError(
                submissionError instanceof ApiError
                  ? submissionError.message
                  : 'Could not sign in',
              )
            } finally {
              setIsSubmitting(false)
            }
          }}
        >
          <label>
            <span>Email</span>
            <input value={email} onChange={(event) => setEmail(event.target.value)} required />
          </label>

          <label>
            <span>Password</span>
            <input
              type="password"
              value={password}
              onChange={(event) => setPassword(event.target.value)}
              required
            />
          </label>

          {error ? <p className="error-text">{error}</p> : null}

          <button className="primary-button" disabled={isSubmitting} type="submit">
            {isSubmitting ? 'Signing in...' : 'Sign in'}
          </button>
        </form>

        <p className="muted">
          No account yet? <Link to="/register">Create one</Link>
        </p>
      </section>
    </div>
  )
}
