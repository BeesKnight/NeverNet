import {
  createContext,
  useContext,
  useEffect,
  useMemo,
  useState,
  type PropsWithChildren,
} from 'react'
import { useQueryClient } from '@tanstack/react-query'

import { ApiError, apiRequest, clearCsrfToken, refreshCsrfToken } from '../../api/client'
import type { AuthResponse, User } from '../../api/types'

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
  const [session, setSession] = useState<AuthResponse | null>(null)
  const [isInitializing, setIsInitializing] = useState(true)

  useEffect(() => {
    let isCancelled = false

    void refreshCsrfToken().catch((error) => {
      if (!isCancelled) {
        console.error(error)
      }
    })

    apiRequest<AuthResponse>('/auth/me')
      .then((nextSession) => {
        if (!isCancelled) {
          setSession(nextSession)
        }
      })
      .catch((error) => {
        if (isCancelled) {
          return
        }

        if (!(error instanceof ApiError && error.status === 401)) {
          console.error(error)
        }

        queryClient.clear()
        setSession(null)
      })
      .finally(() => {
        if (!isCancelled) {
          setIsInitializing(false)
        }
      })

    return () => {
      isCancelled = true
    }
  }, [queryClient])

  const value = useMemo<AuthContextValue>(
    () => ({
      session,
      isInitializing,
      async login(payload) {
        const nextSession = await apiRequest<AuthResponse>('/auth/login', {
          method: 'POST',
          body: JSON.stringify(payload),
        })
        await refreshCsrfToken(true)
        queryClient.clear()
        setSession(nextSession)
      },
      async register(payload) {
        const nextSession = await apiRequest<AuthResponse>('/auth/register', {
          method: 'POST',
          body: JSON.stringify(payload),
        })
        await refreshCsrfToken(true)
        queryClient.clear()
        setSession(nextSession)
      },
      logout() {
        queryClient.clear()
        setSession(null)
        setIsInitializing(false)

        void apiRequest('/auth/logout', {
          method: 'POST',
        })
          .catch(() => undefined)
          .finally(() => {
            clearCsrfToken()
            void refreshCsrfToken(true).catch(() => undefined)
          })
      },
      updateUser(user) {
        setSession((current) => {
          if (!current) {
            return current
          }

          return { ...current, user }
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
