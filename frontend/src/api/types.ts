export type ApiResponse<T> = {
  data: T
}

export type User = {
  id: string
  email: string
  full_name: string
  created_at: string
}

export type AuthResponse = {
  token: string
  user: User
}

export type Category = {
  id: string
  user_id: string
  name: string
  color: string
  created_at: string
  updated_at: string
}

export type EventStatus = 'planned' | 'in_progress' | 'completed' | 'cancelled'

export type Event = {
  id: string
  user_id: string
  category_id: string
  category_name: string
  category_color: string
  title: string
  description: string
  location: string
  starts_at: string
  ends_at: string
  budget: number
  status: EventStatus
  created_at: string
  updated_at: string
}

export type EventRecord = {
  id: string
  user_id: string
  category_id: string
  title: string
  description: string
  location: string
  starts_at: string
  ends_at: string
  budget: number
  status: EventStatus
  created_at: string
  updated_at: string
}

export type EventFilters = {
  search?: string
  status?: string
  category_id?: string
  start_date?: string
  end_date?: string
}

export type ReportCategoryRow = {
  category_id: string
  category_name: string
  category_color: string
  event_count: number
  total_budget: number
}

export type ReportStatusRow = {
  status: string
  event_count: number
  total_budget: number
}

export type ReportSummary = {
  filters: EventFilters
  period_start: string | null
  period_end: string | null
  total_events: number
  total_budget: number
  by_category: ReportCategoryRow[]
  by_status: ReportStatusRow[]
  events: Event[]
}

export type UiSettings = {
  user_id: string
  theme: 'light' | 'dark' | 'system'
  created_at: string
  updated_at: string
}

export type ExportJob = {
  id: string
  user_id: string
  format: 'pdf' | 'xlsx'
  status: 'pending' | 'processing' | 'completed' | 'failed'
  filters: EventFilters
  file_path: string | null
  error_message: string | null
  created_at: string
  updated_at: string
  completed_at: string | null
}
