import { useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'

import { ApiError } from '../api/client'
import { useAuth } from '../features/auth/auth-context'

export function RegisterPage() {
  const navigate = useNavigate()
  const auth = useAuth()
  const [fullName, setFullName] = useState('')
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [isSubmitting, setIsSubmitting] = useState(false)

  return (
    <div className="auth-shell">
      <section className="auth-panel">
        <p className="eyebrow">EventDesign</p>
        <h1>Create an event workspace that stays practical.</h1>
        <p className="panel-copy">
          Registration creates your account and your default theme settings in one step.
        </p>
      </section>

      <section className="auth-card">
        <div className="section-header">
          <div>
            <p className="eyebrow">New account</p>
            <h2>Register</h2>
          </div>
        </div>

        <form
          className="form-grid"
          onSubmit={async (event) => {
            event.preventDefault()
            setError(null)
            setIsSubmitting(true)

            try {
              await auth.register({
                full_name: fullName,
                email,
                password,
              })
              navigate('/dashboard')
            } catch (submissionError) {
              setError(
                submissionError instanceof ApiError
                  ? submissionError.message
                  : 'Could not create the account',
              )
            } finally {
              setIsSubmitting(false)
            }
          }}
        >
          <label>
            <span>Full name</span>
            <input value={fullName} onChange={(event) => setFullName(event.target.value)} required />
          </label>

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
            {isSubmitting ? 'Creating account...' : 'Create account'}
          </button>
        </form>

        <p className="muted">
          Already registered? <Link to="/login">Sign in</Link>
        </p>
      </section>
    </div>
  )
}
