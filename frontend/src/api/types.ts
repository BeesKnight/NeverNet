export type ApiResponse<T> = {
  data: T
}

export type ApiErrorResponse = {
  error: {
    code: string
    message: string
    request_id?: string | null
  }
}

export type User = {
  id: string
  email: string
  full_name: string
  created_at: string
}

export type AuthResponse = {
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

export type SortDirection = 'asc' | 'desc'

export type EventSortField =
  | 'starts_at'
  | 'ends_at'
  | 'budget'
  | 'title'
  | 'status'
  | 'updated_at'
  | 'category_name'

export type EventFilters = {
  search?: string
  status?: string
  category_id?: string
  start_date?: string
  end_date?: string
  sort_by?: EventSortField
  sort_dir?: SortDirection
}

export type CalendarItem = {
  event_id: string
  title: string
  date: string
  starts_at: string
  ends_at: string
  status: EventStatus
  category_color: string
}

export type DashboardCards = {
  total_events: number
  upcoming_events: number
  completed_events: number
  cancelled_events: number
  total_budget: number
}

export type RecentActivityItem = {
  id: string
  entity_type: string
  entity_id: string
  action: string
  title: string
  occurred_at: string
}

export type DashboardResponse = {
  cards: DashboardCards
  upcoming: Event[]
  recent_activity: RecentActivityItem[]
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
  accent_color: string
  default_view: 'dashboard' | 'events' | 'calendar' | 'reports'
  created_at: string
  updated_at: string
}

export type ExportJob = {
  id: string
  user_id: string
  report_type: 'summary'
  format: 'pdf' | 'xlsx'
  status: 'queued' | 'processing' | 'completed' | 'failed'
  filters: EventFilters
  object_key: string | null
  content_type: string | null
  error_message: string | null
  created_at: string
  started_at: string | null
  updated_at: string
  finished_at: string | null
}

export type CsrfTokenResponse = {
  csrf_token: string
}
