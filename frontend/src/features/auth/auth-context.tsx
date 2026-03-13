import {
  createContext,
  useContext,
  useEffect,
  useMemo,
  useState,
  type PropsWithChildren,
} from 'react'
import { useQueryClient } from '@tanstack/react-query'

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
  const queryClient = useQueryClient()
  const [session, setSession] = useState<AuthResponse | null>(() => getStoredSession())
  const [isInitializing, setIsInitializing] = useState(() => Boolean(getStoredSession()?.token))

  useEffect(() => {
    const token = session?.token

    if (!token) {
      return
    }

    let isCancelled = false

    apiRequest<User>('/auth/me', { token })
      .then((user) => {
        if (isCancelled) {
          return
        }

        setSession((current) => {
          if (!current || current.token !== token) {
            return current
          }

          const nextSession = {
            ...current,
            user,
          }
          saveSession(nextSession)
          return nextSession
        })
      })
      .catch(() => {
        if (isCancelled) {
          return
        }

        queryClient.clear()
        setSession(null)
        clearSession()
      })
      .finally(() => {
        if (!isCancelled) {
          setIsInitializing(false)
        }
      })

    return () => {
      isCancelled = true
    }
  }, [queryClient, session?.token])

  const value = useMemo<AuthContextValue>(
    () => ({
      session,
      isInitializing,
      async login(payload) {
        const nextSession = await apiRequest<AuthResponse>('/auth/login', {
          method: 'POST',
          body: JSON.stringify(payload),
        })
        queryClient.clear()
        setSession(nextSession)
        saveSession(nextSession)
      },
      async register(payload) {
        const nextSession = await apiRequest<AuthResponse>('/auth/register', {
          method: 'POST',
          body: JSON.stringify(payload),
        })
        queryClient.clear()
        setSession(nextSession)
        saveSession(nextSession)
      },
      logout() {
        const token = session?.token
        queryClient.clear()
        clearSession()
        setSession(null)
        setIsInitializing(false)

        if (token) {
          void apiRequest('/auth/logout', {
            method: 'POST',
            token,
          }).catch(() => undefined)
        }
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
    [isInitializing, queryClient, session],
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
