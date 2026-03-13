use std::error::Error;

use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString},
};
use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Utc};
use persistence::connect_pool;
use printpdf::{BuiltinFont, Mm, PdfDocument};
use rand_core::OsRng;
use rust_xlsxwriter::Workbook;
use s3::{Bucket, Region, creds::Credentials};
use sqlx::{PgPool, Row};
use uuid::Uuid;

const DEMO_EMAIL: &str = "demo@eventdesign.local";
const DEMO_PASSWORD: &str = "DemoPass123!";
const DEMO_NAME: &str = "Defense Demo";

#[derive(Clone)]
struct Config {
    database_url: String,
    minio_endpoint: String,
    minio_bucket: String,
    minio_access_key: String,
    minio_secret_key: String,
    minio_region: String,
}

#[derive(Clone)]
struct CategorySeed {
    id: Uuid,
    name: &'static str,
    color: &'static str,
}

#[derive(Clone)]
struct EventSeed {
    id: Uuid,
    category_id: Uuid,
    title: &'static str,
    location: &'static str,
    status: &'static str,
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
    budget: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

    let config = Config::from_env()?;
    let pool = connect_pool(&config.database_url, 5).await?;
    sqlx::migrate!("../../migrations").run(&pool).await?;

    let storage = build_storage_client(&config)?;
    ensure_bucket(&storage, &config.minio_bucket).await;

    let user_id = recreate_demo_user(&pool).await?;
    let categories = build_categories();
    let events = build_events(&categories);

    insert_categories(&pool, user_id, &categories).await?;
    insert_events(&pool, user_id, &events).await?;
    refresh_projections(&pool, user_id).await?;
    seed_exports(&pool, &storage, user_id, &events).await?;

    println!("Demo seed completed.");
    println!("Email: {DEMO_EMAIL}");
    println!("Password: {DEMO_PASSWORD}");
    println!("Categories: {}", categories.len());
    println!("Events: {}", events.len());

    Ok(())
}

impl Config {
    fn from_env() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")?,
            minio_endpoint: std::env::var("MINIO_ENDPOINT")
                .unwrap_or_else(|_| "http://127.0.0.1:9000".to_string()),
            minio_bucket: std::env::var("MINIO_BUCKET")
                .unwrap_or_else(|_| "eventdesign-exports".to_string()),
            minio_access_key: std::env::var("MINIO_ACCESS_KEY")
                .unwrap_or_else(|_| "eventdesign".to_string()),
            minio_secret_key: std::env::var("MINIO_SECRET_KEY")
                .unwrap_or_else(|_| "eventdesign123".to_string()),
            minio_region: std::env::var("MINIO_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
        })
    }
}

async fn recreate_demo_user(pool: &PgPool) -> Result<Uuid, Box<dyn Error>> {
    let existing_user_id = sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE email = $1")
        .bind(DEMO_EMAIL)
        .fetch_optional(pool)
        .await?;

    if let Some(existing_user_id) = existing_user_id {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(existing_user_id)
            .execute(pool)
            .await?;
    }

    let password_hash = Argon2::default()
        .hash_password(DEMO_PASSWORD.as_bytes(), &SaltString::generate(&mut OsRng))
        .map_err(|error| error.to_string())?
        .to_string();
    let user_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, full_name)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(user_id)
    .bind(DEMO_EMAIL)
    .bind(password_hash)
    .bind(DEMO_NAME)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO ui_settings (user_id, theme, updated_at)
        VALUES ($1, 'system', NOW())
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(user_id)
}

fn build_categories() -> Vec<CategorySeed> {
    vec![
        CategorySeed {
            id: Uuid::new_v4(),
            name: "Conference",
            color: "#0f766e",
        },
        CategorySeed {
            id: Uuid::new_v4(),
            name: "Meetup",
            color: "#2563eb",
        },
        CategorySeed {
            id: Uuid::new_v4(),
            name: "Workshop",
            color: "#b45309",
        },
        CategorySeed {
            id: Uuid::new_v4(),
            name: "Launch",
            color: "#be123c",
        },
        CategorySeed {
            id: Uuid::new_v4(),
            name: "Expo",
            color: "#7c3aed",
        },
    ]
}

fn build_events(categories: &[CategorySeed]) -> Vec<EventSeed> {
    let now = Utc::now();
    let month_start = Utc
        .with_ymd_and_hms(now.year(), now.month(), 1, 9, 0, 0)
        .single()
        .unwrap_or(now);
    let offsets = [-10, -7, -5, -3, -1, 1, 3, 5, 7, 10, 13, 16, 20, 24];
    let titles = [
        ("Design sync", "Room A"),
        ("Venue walkthrough", "Main hall"),
        ("Vendor alignment", "Zoom"),
        ("Marketing review", "Studio"),
        ("Speaker prep", "Room B"),
        ("Launch standup", "Ops desk"),
        ("Volunteer briefing", "Room C"),
        ("Budget checkpoint", "Finance pod"),
        ("Schedule rehearsal", "Auditorium"),
        ("Security review", "Control room"),
        ("Expo setup", "Hall 2"),
        ("Defense rehearsal", "Room 301"),
        ("Guest arrival plan", "Lobby"),
        ("Wrap-up retro", "Room D"),
    ];

    offsets
        .iter()
        .enumerate()
        .map(|(index, offset)| {
            let starts_at = month_start + Duration::days((index as i64 * 2) + offset);
            let ends_at = starts_at + Duration::hours(2 + (index % 3) as i64);
            let status = if ends_at < now {
                if index % 5 == 0 {
                    "cancelled"
                } else {
                    "completed"
                }
            } else if starts_at < now {
                "in_progress"
            } else {
                "planned"
            };

            EventSeed {
                id: Uuid::new_v4(),
                category_id: categories[index % categories.len()].id,
                title: titles[index].0,
                location: titles[index].1,
                status,
                starts_at,
                ends_at,
                budget: 450.0 + (index as f64 * 125.0),
            }
        })
        .collect()
}

async fn insert_categories(
    pool: &PgPool,
    user_id: Uuid,
    categories: &[CategorySeed],
) -> Result<(), Box<dyn Error>> {
    for category in categories {
        sqlx::query(
            r#"
            INSERT INTO categories (id, user_id, name, color)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(category.id)
        .bind(user_id)
        .bind(category.name)
        .bind(category.color)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn insert_events(
    pool: &PgPool,
    user_id: Uuid,
    events: &[EventSeed],
) -> Result<(), Box<dyn Error>> {
    for event in events {
        sqlx::query(
            r#"
            INSERT INTO events (
                id,
                user_id,
                category_id,
                title,
                description,
                location,
                starts_at,
                ends_at,
                budget,
                status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(event.id)
        .bind(user_id)
        .bind(event.category_id)
        .bind(event.title)
        .bind(format!("Seeded demo event for {}.", event.title))
        .bind(event.location)
        .bind(event.starts_at)
        .bind(event.ends_at)
        .bind(event.budget)
        .bind(event.status)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn refresh_projections(pool: &PgPool, user_id: Uuid) -> Result<(), Box<dyn Error>> {
    sqlx::query("DELETE FROM event_list_projection WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM calendar_projection WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM report_projection WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM dashboard_projection WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM recent_activity_projection WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        INSERT INTO event_list_projection (
            event_id,
            user_id,
            category_id,
            category_name,
            category_color,
            title,
            description,
            location,
            starts_at,
            ends_at,
            budget,
            status,
            created_at,
            updated_at
        )
        SELECT
            e.id,
            e.user_id,
            e.category_id,
            c.name,
            c.color,
            e.title,
            e.description,
            e.location,
            e.starts_at,
            e.ends_at,
            e.budget,
            e.status,
            e.created_at,
            e.updated_at
        FROM events e
        INNER JOIN categories c ON c.id = e.category_id
        WHERE e.user_id = $1
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO calendar_projection (
            event_id,
            user_id,
            date_bucket,
            title,
            starts_at,
            ends_at,
            status,
            category_color,
            updated_at
        )
        SELECT
            e.id,
            e.user_id,
            DATE(e.starts_at AT TIME ZONE 'UTC'),
            e.title,
            e.starts_at,
            e.ends_at,
            e.status,
            c.color,
            e.updated_at
        FROM events e
        INNER JOIN categories c ON c.id = e.category_id
        WHERE e.user_id = $1
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO report_projection (
            event_id,
            user_id,
            category_id,
            category_name,
            category_color,
            title,
            description,
            location,
            starts_at,
            ends_at,
            budget,
            status,
            created_at,
            updated_at
        )
        SELECT
            e.id,
            e.user_id,
            e.category_id,
            c.name,
            c.color,
            e.title,
            e.description,
            e.location,
            e.starts_at,
            e.ends_at,
            e.budget,
            e.status,
            e.created_at,
            e.updated_at
        FROM events e
        INNER JOIN categories c ON c.id = e.category_id
        WHERE e.user_id = $1
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO dashboard_projection (
            user_id,
            total_events,
            upcoming_events,
            completed_events,
            cancelled_events,
            total_budget,
            updated_at
        )
        SELECT
            u.id,
            COUNT(e.id)::BIGINT,
            COUNT(*) FILTER (WHERE e.starts_at >= NOW() AND e.status <> 'cancelled')::BIGINT,
            COUNT(*) FILTER (WHERE e.status = 'completed')::BIGINT,
            COUNT(*) FILTER (WHERE e.status = 'cancelled')::BIGINT,
            COALESCE(SUM(e.budget), 0)::DOUBLE PRECISION,
            NOW()
        FROM users u
        LEFT JOIN events e ON e.user_id = u.id
        WHERE u.id = $1
        GROUP BY u.id
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    for row in sqlx::query(
        r#"
        SELECT id, title, updated_at
        FROM events
        WHERE user_id = $1
        ORDER BY starts_at DESC
        LIMIT 20
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?
    {
        sqlx::query(
            r#"
            INSERT INTO recent_activity_projection (
                source_message_id,
                user_id,
                entity_type,
                entity_id,
                action,
                title,
                occurred_at
            )
            VALUES ($1, $2, 'event', $3, 'seeded', $4, $5)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(row.get::<Uuid, _>("id"))
        .bind(row.get::<String, _>("title"))
        .bind(row.get::<DateTime<Utc>, _>("updated_at"))
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn seed_exports(
    pool: &PgPool,
    bucket: &Bucket,
    user_id: Uuid,
    events: &[EventSeed],
) -> Result<(), Box<dyn Error>> {
    sqlx::query("DELETE FROM export_jobs WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM outbox_events WHERE aggregate_type = 'export' AND payload_json->>'user_id' = $1")
        .bind(user_id.to_string())
        .execute(pool)
        .await?;

    let pdf_id = Uuid::new_v4();
    let xlsx_id = Uuid::new_v4();
    let queued_id = Uuid::new_v4();
    let pdf_key = format!("/exports/{user_id}/{pdf_id}.pdf");
    let xlsx_key = format!("/exports/{user_id}/{xlsx_id}.xlsx");
    let created_at = Utc::now() - Duration::hours(6);

    let pdf_bytes = build_pdf(events)?;
    upload_object(bucket, &pdf_key, &pdf_bytes, "application/pdf").await?;
    sqlx::query(
        r#"
        INSERT INTO export_jobs (
            id,
            user_id,
            report_type,
            format,
            status,
            filters,
            object_key,
            content_type,
            created_at,
            started_at,
            updated_at,
            finished_at
        )
        VALUES ($1, $2, 'summary', 'pdf', 'completed', '{}'::jsonb, $3, 'application/pdf', $4, $5, $6, $6)
        "#,
    )
    .bind(pdf_id)
    .bind(user_id)
    .bind(&pdf_key)
    .bind(created_at)
    .bind(created_at + Duration::minutes(1))
    .bind(created_at + Duration::minutes(2))
    .execute(pool)
    .await?;

    let xlsx_bytes = build_xlsx(events)?;
    upload_object(
        bucket,
        &xlsx_key,
        &xlsx_bytes,
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    )
    .await?;
    sqlx::query(
        r#"
        INSERT INTO export_jobs (
            id,
            user_id,
            report_type,
            format,
            status,
            filters,
            object_key,
            content_type,
            created_at,
            started_at,
            updated_at,
            finished_at
        )
        VALUES (
            $1,
            $2,
            'summary',
            'xlsx',
            'completed',
            '{"status":"planned"}'::jsonb,
            $3,
            'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
            $4,
            $5,
            $6,
            $6
        )
        "#,
    )
    .bind(xlsx_id)
    .bind(user_id)
    .bind(&xlsx_key)
    .bind(created_at + Duration::minutes(30))
    .bind(created_at + Duration::minutes(31))
    .bind(created_at + Duration::minutes(32))
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO export_jobs (
            id,
            user_id,
            report_type,
            format,
            status,
            filters,
            created_at,
            updated_at
        )
        VALUES ($1, $2, 'summary', 'pdf', 'queued', '{"status":"planned"}'::jsonb, NOW(), NOW())
        "#,
    )
    .bind(queued_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    let occurred_at = Utc::now();
    let payload = serde_json::json!({
        "export_id": queued_id,
        "user_id": user_id,
        "report_type": "summary",
        "format": "pdf",
        "status": "queued",
        "filters": {
            "status": "planned"
        },
        "object_key": null,
        "error_message": null,
        "created_at": occurred_at,
        "started_at": null,
        "finished_at": null
    });
    sqlx::query(
        r#"
        INSERT INTO outbox_events (
            aggregate_type,
            aggregate_id,
            event_type,
            event_version,
            payload_json,
            occurred_at
        )
        VALUES ('export', $1, 'export.requested', 1, $2, $3)
        "#,
    )
    .bind(queued_id)
    .bind(payload)
    .bind(occurred_at)
    .execute(pool)
    .await?;

    Ok(())
}

fn build_pdf(events: &[EventSeed]) -> Result<Vec<u8>, Box<dyn Error>> {
    let (document, first_page, first_layer) =
        PdfDocument::new("EventDesign demo export", Mm(210.0), Mm(297.0), "Layer 1");
    let font = document.add_builtin_font(BuiltinFont::Helvetica)?;
    let layer = document.get_page(first_page).get_layer(first_layer);
    let mut y = 285.0;

    layer.use_text("EventDesign demo export", 18.0, Mm(12.0), Mm(y), &font);
    y -= 12.0;
    for event in events.iter().take(10) {
        layer.use_text(
            format!(
                "{} | {} | {} | {:.2}",
                event.starts_at.format("%Y-%m-%d %H:%M"),
                event.title,
                event.status,
                event.budget
            ),
            11.0,
            Mm(12.0),
            Mm(y),
            &font,
        );
        y -= 7.0;
    }

    Ok(document.save_to_bytes()?)
}

fn build_xlsx(events: &[EventSeed]) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.write_string(0, 0, "Title")?;
    worksheet.write_string(0, 1, "Location")?;
    worksheet.write_string(0, 2, "Status")?;
    worksheet.write_string(0, 3, "Starts At")?;
    worksheet.write_string(0, 4, "Budget")?;

    for (index, event) in events.iter().enumerate() {
        let row = (index + 1) as u32;
        worksheet.write_string(row, 0, event.title)?;
        worksheet.write_string(row, 1, event.location)?;
        worksheet.write_string(row, 2, event.status)?;
        worksheet.write_string(row, 3, event.starts_at.format("%Y-%m-%d %H:%M").to_string())?;
        worksheet.write_number(row, 4, event.budget)?;
    }

    Ok(workbook.save_to_buffer()?)
}

fn build_storage_client(config: &Config) -> Result<Box<Bucket>, Box<dyn Error>> {
    let region = Region::Custom {
        region: config.minio_region.clone(),
        endpoint: config.minio_endpoint.clone(),
    };
    let credentials = Credentials::new(
        Some(&config.minio_access_key),
        Some(&config.minio_secret_key),
        None,
        None,
        None,
    )?;
    Ok(Bucket::new(&config.minio_bucket, region, credentials)?.with_path_style())
}

async fn ensure_bucket(client: &Bucket, bucket_name: &str) {
    if client.exists().await.unwrap_or(false) {
        return;
    }

    unsafe {
        std::env::set_var("RUST_S3_SKIP_LOCATION_CONSTRAINT", "1");
    }
    let credentials = match client.credentials().await {
        Ok(credentials) => credentials,
        Err(_) => return,
    };
    let _ = Bucket::create_with_path_style(
        bucket_name,
        client.region().clone(),
        credentials,
        s3::bucket_ops::BucketConfiguration::default(),
    )
    .await;
}

async fn upload_object(
    bucket: &Bucket,
    object_key: &str,
    bytes: &[u8],
    content_type: &str,
) -> Result<(), Box<dyn Error>> {
    let response = bucket
        .put_object_with_content_type(object_key, bytes, content_type)
        .await?;

    if !(200..300).contains(&response.status_code()) {
        return Err(format!("MinIO upload failed with status {}", response.status_code()).into());
    }

    Ok(())
}

#[allow(dead_code)]
fn month_bounds(value: DateTime<Utc>) -> (NaiveDate, NaiveDate) {
    let start = NaiveDate::from_ymd_opt(value.year(), value.month(), 1).unwrap();
    let end = if value.month() == 12 {
        NaiveDate::from_ymd_opt(value.year() + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(value.year(), value.month() + 1, 1).unwrap()
    };
    (start, end)
}
