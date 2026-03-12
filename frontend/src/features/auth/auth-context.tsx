import {
  createContext,
  useContext,
  useEffect,
  useMemo,
  useState,
  type PropsWithChildren,
} from 'react'

import { apiRequest } from '../../api/client'
import type { AuthResponse, User } from '../../api/types'
import { clearSession, getStoredSession, saveSession } from './auth-storage'

type LoginPayload = {
  email: string
  password: string
}

type RegisterPayload = LoginPayload & {
  full_name: string
}

export type AuthContextValue = {
  session: AuthResponse | null
  isInitializing: boolean
  login: (payload: LoginPayload) => Promise<void>
  register: (payload: RegisterPayload) => Promise<void>
  logout: () => void
  updateUser: (user: User) => void
}

const AuthContext = createContext<AuthContextValue | null>(null)

export function AuthProvider({ children }: PropsWithChildren) {
  const [session, setSession] = useState<AuthResponse | null>(() => getStoredSession())
  const [isInitializing, setIsInitializing] = useState(() => Boolean(getStoredSession()?.token))

  useEffect(() => {
    if (!session?.token) {
      return
    }

    apiRequest<User>('/auth/me', { token: session.token })
      .then((user) => {
        const nextSession = {
          ...session,
          user,
        }
        setSession(nextSession)
        saveSession(nextSession)
      })
      .catch(() => {
        setSession(null)
        clearSession()
      })
      .finally(() => {
        setIsInitializing(false)
      })
  }, [session])

  const value = useMemo<AuthContextValue>(
    () => ({
      session,
      isInitializing,
      async login(payload) {
        const nextSession = await apiRequest<AuthResponse>('/auth/login', {
          method: 'POST',
          body: JSON.stringify(payload),
        })
        setSession(nextSession)
        saveSession(nextSession)
      },
      async register(payload) {
        const nextSession = await apiRequest<AuthResponse>('/auth/register', {
          method: 'POST',
          body: JSON.stringify(payload),
        })
        setSession(nextSession)
        saveSession(nextSession)
      },
      logout() {
        clearSession()
        setSession(null)
      },
      updateUser(user) {
        setSession((current) => {
          if (!current) {
            return current
          }

          const nextSession = { ...current, user }
          saveSession(nextSession)
          return nextSession
        })
      },
    }),
    [isInitializing, session],
  )

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>
}

export function useAuth() {
  const context = useContext(AuthContext)

  if (!context) {
    throw new Error('useAuth must be used inside AuthProvider')
  }

  return context
}
