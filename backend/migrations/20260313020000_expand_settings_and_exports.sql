ALTER TABLE ui_settings
    ADD COLUMN accent_color TEXT NOT NULL DEFAULT '#b6532f',
    ADD COLUMN default_view TEXT NOT NULL DEFAULT 'dashboard',
    ADD CONSTRAINT ui_settings_accent_color_check CHECK (accent_color ~ '^#[0-9A-Fa-f]{6}$'),
    ADD CONSTRAINT ui_settings_default_view_check CHECK (
        default_view IN ('dashboard', 'events', 'calendar', 'reports')
    );

ALTER TABLE export_jobs
    ADD COLUMN report_type TEXT NOT NULL DEFAULT 'summary';

ALTER TABLE export_jobs
    ADD CONSTRAINT export_jobs_report_type_check CHECK (report_type IN ('summary'));

ALTER TABLE export_jobs
    RENAME COLUMN completed_at TO finished_at;
