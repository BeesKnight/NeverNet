use std::{fs::File, io::BufWriter, path::PathBuf};

use axum::{
    body::Body,
    http::{
        StatusCode,
        header::{CONTENT_DISPOSITION, CONTENT_TYPE, HeaderValue},
    },
    response::Response,
};
use printpdf::{BuiltinFont, Mm, PdfDocument};
use rust_xlsxwriter::Workbook;
use tokio::fs;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    error::AppError,
    events::models::EventFilters,
    exports::{
        models::{CreateExportRequest, ExportJob},
        repository,
    },
    reports::{models::ReportSummary, service as reports_service},
};

const ALLOWED_FORMATS: [&str; 2] = ["pdf", "xlsx"];
const ALLOWED_REPORT_TYPES: [&str; 1] = ["summary"];

pub async fn list(state: &AppState, user_id: Uuid) -> Result<Vec<ExportJob>, AppError> {
    Ok(repository::list(&state.pool, user_id).await?)
}

pub async fn get(state: &AppState, user_id: Uuid, export_id: Uuid) -> Result<ExportJob, AppError> {
    repository::find_by_id(&state.pool, user_id, export_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Export job not found".to_string()))
}

pub async fn create(
    state: &AppState,
    user_id: Uuid,
    payload: CreateExportRequest,
) -> Result<ExportJob, AppError> {
    let report_type = payload.report_type.trim().to_lowercase();
    let format = payload.format.trim().to_lowercase();

    if !ALLOWED_REPORT_TYPES.contains(&report_type.as_str()) {
        return Err(AppError::BadRequest(
            "Report type must be summary".to_string(),
        ));
    }

    if !ALLOWED_FORMATS.contains(&format.as_str()) {
        return Err(AppError::BadRequest(
            "Export format must be either pdf or xlsx".to_string(),
        ));
    }

    let filters = serde_json::to_value(&payload.filters)
        .map_err(|error| AppError::Internal(error.to_string()))?;
    let job = repository::create(&state.pool, user_id, &report_type, &format, filters).await?;

    let state_clone = state.clone();
    tokio::spawn(async move {
        if let Err(error) = process_export_job(state_clone, job.id).await {
            tracing::error!("export job {} failed: {}", job.id, error);
        }
    });

    Ok(job)
}

pub async fn download(
    state: &AppState,
    user_id: Uuid,
    export_id: Uuid,
) -> Result<Response, AppError> {
    let job = repository::find_by_id(&state.pool, user_id, export_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Export job not found".to_string()))?;

    if job.status != "completed" {
        return Err(AppError::BadRequest(
            "Export file is not ready yet".to_string(),
        ));
    }

    let file_path = job
        .file_path
        .clone()
        .ok_or_else(|| AppError::Internal("Completed export job is missing a file".to_string()))?;
    let bytes = fs::read(&file_path).await?;
    let content_type = if job.format == "pdf" {
        "application/pdf"
    } else {
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
    };
    let file_name = format!("event-report-{}.{}", job.id, job.format);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, HeaderValue::from_static(content_type))
        .header(
            CONTENT_DISPOSITION,
            HeaderValue::from_str(&format!("attachment; filename=\"{file_name}\""))
                .map_err(|error| AppError::Internal(error.to_string()))?,
        )
        .body(Body::from(bytes))
        .map_err(AppError::from)?)
}

pub async fn resume_pending_jobs(state: AppState) {
    match repository::pending(&state.pool).await {
        Ok(jobs) => {
            for job in jobs {
                let state_clone = state.clone();
                tokio::spawn(async move {
                    if let Err(error) = process_export_job(state_clone, job.id).await {
                        tracing::error!("failed to resume export {}: {}", job.id, error);
                    }
                });
            }
        }
        Err(error) => tracing::error!("could not inspect pending export jobs: {}", error),
    }
}

pub async fn process_export_job(state: AppState, job_id: Uuid) -> Result<(), AppError> {
    let job = repository::find_by_job_id(&state.pool, job_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Export job not found".to_string()))?;

    repository::set_processing(&state.pool, job.id).await?;
    let result = async {
        let filters: EventFilters = serde_json::from_value(job.filters.clone())
            .map_err(|error| AppError::Internal(error.to_string()))?;
        let report = reports_service::generate_summary(&state, job.user_id, filters).await?;
        let file_path = build_export_file(&state, &job, &report).await?;
        repository::complete(&state.pool, job.id, &file_path.to_string_lossy()).await?;
        Ok::<(), AppError>(())
    }
    .await;

    if let Err(error) = result {
        repository::fail(&state.pool, job.id, &error.to_string()).await?;
        return Err(error);
    }

    Ok(())
}

async fn build_export_file(
    state: &AppState,
    job: &ExportJob,
    report: &ReportSummary,
) -> Result<PathBuf, AppError> {
    let user_dir = state.config.export_dir.join(job.user_id.to_string());
    fs::create_dir_all(&user_dir).await?;

    let extension = if job.format == "pdf" { "pdf" } else { "xlsx" };
    let path = user_dir.join(format!("{}.{}", job.id, extension));

    if job.format == "pdf" {
        build_pdf(&path, report)?;
    } else {
        build_xlsx(&path, report)?;
    }

    Ok(path)
}

fn build_pdf(path: &PathBuf, report: &ReportSummary) -> Result<(), AppError> {
    let (document, first_page, first_layer) =
        PdfDocument::new("NeverNet event report", Mm(210.0), Mm(297.0), "Layer 1");
    let font = document
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|error| AppError::Internal(error.to_string()))?;

    let mut y = 285.0;
    let mut page = first_page;
    let mut layer = first_layer;
    let mut current_layer = document.get_page(page).get_layer(layer);
    let mut page_number = 1;

    for line in pdf_lines(report) {
        if y < 14.0 {
            page_number += 1;
            let (next_page, next_layer) =
                document.add_page(Mm(210.0), Mm(297.0), format!("Layer {page_number}"));
            page = next_page;
            layer = next_layer;
            current_layer = document.get_page(page).get_layer(layer);
            y = 285.0;
        }

        current_layer.use_text(line, 11.0, Mm(12.0), Mm(y), &font);
        y -= 7.0;
    }

    let mut writer = BufWriter::new(File::create(path)?);
    document
        .save(&mut writer)
        .map_err(|error| AppError::Internal(error.to_string()))?;
    Ok(())
}

fn pdf_lines(report: &ReportSummary) -> Vec<String> {
    let mut lines = vec![
        "NeverNet Event Report".to_string(),
        format!("Total events: {}", report.total_events),
        format!("Total budget: {:.2}", report.total_budget),
        format!(
            "Period: {} - {}",
            report
                .period_start
                .map(|value| value.to_string())
                .unwrap_or_else(|| "Any".to_string()),
            report
                .period_end
                .map(|value| value.to_string())
                .unwrap_or_else(|| "Any".to_string())
        ),
        String::new(),
        "Events".to_string(),
    ];

    for event in &report.events {
        lines.push(format!(
            "{} | {} | {} | {}",
            event.starts_at.format("%Y-%m-%d %H:%M"),
            event.title,
            event.category_name,
            event.status
        ));
        lines.push(format!(
            "Location: {} | Ends: {} | Budget: {:.2}",
            if event.location.trim().is_empty() {
                "Not specified"
            } else {
                event.location.as_str()
            },
            event.ends_at.format("%Y-%m-%d %H:%M"),
            event.budget
        ));
        lines.push(String::new());
    }

    lines
}

fn build_xlsx(path: &PathBuf, report: &ReportSummary) -> Result<(), AppError> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet
        .write_string(0, 0, "Title")
        .map_err(map_xlsx_error)?;
    worksheet
        .write_string(0, 1, "Category")
        .map_err(map_xlsx_error)?;
    worksheet
        .write_string(0, 2, "Location")
        .map_err(map_xlsx_error)?;
    worksheet
        .write_string(0, 3, "Status")
        .map_err(map_xlsx_error)?;
    worksheet
        .write_string(0, 4, "Starts At")
        .map_err(map_xlsx_error)?;
    worksheet
        .write_string(0, 5, "Ends At")
        .map_err(map_xlsx_error)?;
    worksheet
        .write_string(0, 6, "Budget")
        .map_err(map_xlsx_error)?;

    for (index, event) in report.events.iter().enumerate() {
        let row = (index + 1) as u32;
        worksheet
            .write_string(row, 0, &event.title)
            .map_err(map_xlsx_error)?;
        worksheet
            .write_string(row, 1, &event.category_name)
            .map_err(map_xlsx_error)?;
        worksheet
            .write_string(row, 2, &event.location)
            .map_err(map_xlsx_error)?;
        worksheet
            .write_string(row, 3, &event.status)
            .map_err(map_xlsx_error)?;
        worksheet
            .write_string(row, 4, event.starts_at.format("%Y-%m-%d %H:%M").to_string())
            .map_err(map_xlsx_error)?;
        worksheet
            .write_string(row, 5, event.ends_at.format("%Y-%m-%d %H:%M").to_string())
            .map_err(map_xlsx_error)?;
        worksheet
            .write_number(row, 6, event.budget)
            .map_err(map_xlsx_error)?;
    }

    workbook.save(path).map_err(map_xlsx_error)?;
    Ok(())
}

fn map_xlsx_error(error: rust_xlsxwriter::XlsxError) -> AppError {
    AppError::Internal(error.to_string())
}
