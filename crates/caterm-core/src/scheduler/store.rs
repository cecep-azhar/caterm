//! SQLite store for Scheduled Tasks and execution logs (REQ-38).

use crate::db;
use crate::error::{CatermError, DbError};
use crate::scheduler::task::{
    ScheduledTaskRecord, TaskExecutionLog, TaskInput, TaskStatus, TaskType,
};
use rusqlite::{Connection, params};

pub fn list_tasks() -> Result<Vec<ScheduledTaskRecord>, CatermError> {
    let conn = db::open()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, task_type, host_id, schedule_expr, is_enabled,
                    command_script, timeout_seconds, remote_src_path, local_dest_dir,
                    remote_pre_cmd, remote_post_cmd, retention_count, notify_on_success,
                    notify_on_failure, last_run_at, last_status, next_run_at, created_at, updated_at
             FROM scheduled_tasks
             ORDER BY created_at DESC",
        )
        .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;

    let rows = stmt
        .query_map([], |row| {
            let task_type_str: String = row.get(3)?;
            let is_enabled_int: i64 = row.get(6)?;
            let notify_success_int: i64 = row.get(14)?;
            let notify_failure_int: i64 = row.get(15)?;
            let last_status_str: Option<String> = row.get(17)?;

            Ok(ScheduledTaskRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                task_type: TaskType::from_str(&task_type_str),
                host_id: row.get(4)?,
                schedule_expr: row.get(5)?,
                is_enabled: is_enabled_int != 0,
                command_script: row.get(7)?,
                timeout_seconds: row.get(8)?,
                remote_src_path: row.get(9)?,
                local_dest_dir: row.get(10)?,
                remote_pre_cmd: row.get(11)?,
                remote_post_cmd: row.get(12)?,
                retention_count: row.get(13)?,
                notify_on_success: notify_success_int != 0,
                notify_on_failure: notify_failure_int != 0,
                last_run_at: row.get(16)?,
                last_status: last_status_str.as_deref().map(TaskStatus::from_str),
                next_run_at: row.get(18)?,
                created_at: row.get(19)?,
                updated_at: row.get(20)?,
            })
        })
        .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;

    let mut result = Vec::new();
    for r in rows {
        result.push(r.map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?);
    }
    Ok(result)
}

pub fn get_task_by_id(id: &str) -> Result<Option<ScheduledTaskRecord>, CatermError> {
    let conn = db::open()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, task_type, host_id, schedule_expr, is_enabled,
                    command_script, timeout_seconds, remote_src_path, local_dest_dir,
                    remote_pre_cmd, remote_post_cmd, retention_count, notify_on_success,
                    notify_on_failure, last_run_at, last_status, next_run_at, created_at, updated_at
             FROM scheduled_tasks
             WHERE id = ?1",
        )
        .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;

    let mut rows = stmt
        .query_map(params![id], |row| {
            let task_type_str: String = row.get(3)?;
            let is_enabled_int: i64 = row.get(6)?;
            let notify_success_int: i64 = row.get(14)?;
            let notify_failure_int: i64 = row.get(15)?;
            let last_status_str: Option<String> = row.get(17)?;

            Ok(ScheduledTaskRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                task_type: TaskType::from_str(&task_type_str),
                host_id: row.get(4)?,
                schedule_expr: row.get(5)?,
                is_enabled: is_enabled_int != 0,
                command_script: row.get(7)?,
                timeout_seconds: row.get(8)?,
                remote_src_path: row.get(9)?,
                local_dest_dir: row.get(10)?,
                remote_pre_cmd: row.get(11)?,
                remote_post_cmd: row.get(12)?,
                retention_count: row.get(13)?,
                notify_on_success: notify_success_int != 0,
                notify_on_failure: notify_failure_int != 0,
                last_run_at: row.get(16)?,
                last_status: last_status_str.as_deref().map(TaskStatus::from_str),
                next_run_at: row.get(18)?,
                created_at: row.get(19)?,
                updated_at: row.get(20)?,
            })
        })
        .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;

    if let Some(res) = rows.next() {
        let task = res.map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;
        Ok(Some(task))
    } else {
        Ok(None)
    }
}

pub fn save_task(input: TaskInput) -> Result<ScheduledTaskRecord, CatermError> {
    let conn = db::open()?;
    let now = chrono::Utc::now().to_rfc3339();

    let id = input
        .id
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let is_enabled_int = if input.is_enabled { 1 } else { 0 };
    let notify_success_int = if input.notify_on_success { 1 } else { 0 };
    let notify_failure_int = if input.notify_on_failure { 1 } else { 0 };

    conn.execute(
        "INSERT INTO scheduled_tasks (
            id, name, description, task_type, host_id, schedule_expr, is_enabled,
            command_script, timeout_seconds, remote_src_path, local_dest_dir,
            remote_pre_cmd, remote_post_cmd, retention_count, notify_on_success,
            notify_on_failure, created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            description = excluded.description,
            task_type = excluded.task_type,
            host_id = excluded.host_id,
            schedule_expr = excluded.schedule_expr,
            is_enabled = excluded.is_enabled,
            command_script = excluded.command_script,
            timeout_seconds = excluded.timeout_seconds,
            remote_src_path = excluded.remote_src_path,
            local_dest_dir = excluded.local_dest_dir,
            remote_pre_cmd = excluded.remote_pre_cmd,
            remote_post_cmd = excluded.remote_post_cmd,
            retention_count = excluded.retention_count,
            notify_on_success = excluded.notify_on_success,
            notify_on_failure = excluded.notify_on_failure,
            updated_at = excluded.updated_at",
        params![
            id,
            input.name,
            input.description,
            input.task_type.as_str(),
            input.host_id,
            input.schedule_expr,
            is_enabled_int,
            input.command_script,
            input.timeout_seconds,
            input.remote_src_path,
            input.local_dest_dir,
            input.remote_pre_cmd,
            input.remote_post_cmd,
            input.retention_count,
            notify_success_int,
            notify_failure_int,
            now,
            now
        ],
    )
    .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;

    get_task_by_id(&id)?.ok_or_else(|| {
        CatermError::Db(DbError::Generic(
            "Tugas tidak ditemukan setelah disimpan".into(),
        ))
    })
}

pub fn delete_task(id: &str) -> Result<(), CatermError> {
    let conn = db::open()?;
    conn.execute("DELETE FROM scheduled_tasks WHERE id = ?1", params![id])
        .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;
    Ok(())
}

pub fn update_task_status(
    task_id: &str,
    last_run_at: &str,
    last_status: TaskStatus,
    next_run_at: Option<&str>,
) -> Result<(), CatermError> {
    let conn = db::open()?;
    update_task_status_in(&conn, task_id, last_run_at, last_status, next_run_at)
}

pub(crate) fn update_task_status_in(
    conn: &Connection,
    task_id: &str,
    last_run_at: &str,
    last_status: TaskStatus,
    next_run_at: Option<&str>,
) -> Result<(), CatermError> {
    conn.execute(
        "UPDATE scheduled_tasks
         SET last_run_at = ?1, last_status = ?2, next_run_at = ?3, updated_at = ?4
         WHERE id = ?5",
        params![
            last_run_at,
            last_status.as_str(),
            next_run_at,
            chrono::Utc::now().to_rfc3339(),
            task_id
        ],
    )
    .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;
    Ok(())
}

pub fn log_execution(log: &TaskExecutionLog) -> Result<(), CatermError> {
    let conn = db::open()?;
    conn.execute(
        "INSERT INTO task_execution_logs (
            id, task_id, started_at, finished_at, duration_ms, exit_code,
            status, stdout, stderr, bytes_transferred, error_message
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            log.id,
            log.task_id,
            log.started_at,
            log.finished_at,
            log.duration_ms,
            log.exit_code,
            log.status.as_str(),
            log.stdout,
            log.stderr,
            log.bytes_transferred,
            log.error_message
        ],
    )
    .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;

    update_task_status_in(
        &conn,
        &log.task_id,
        &log.started_at,
        log.status.clone(),
        None,
    )?;

    Ok(())
}

pub fn get_task_logs(task_id: &str) -> Result<Vec<TaskExecutionLog>, CatermError> {
    let conn = db::open()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, task_id, started_at, finished_at, duration_ms, exit_code,
                    status, stdout, stderr, bytes_transferred, error_message
             FROM task_execution_logs
             WHERE task_id = ?1
             ORDER BY started_at DESC
             LIMIT 100",
        )
        .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;

    let rows = stmt
        .query_map(params![task_id], |row| {
            let status_str: String = row.get(6)?;
            Ok(TaskExecutionLog {
                id: row.get(0)?,
                task_id: row.get(1)?,
                started_at: row.get(2)?,
                finished_at: row.get(3)?,
                duration_ms: row.get(4)?,
                exit_code: row.get(5)?,
                status: TaskStatus::from_str(&status_str),
                stdout: row.get(7)?,
                stderr: row.get(8)?,
                bytes_transferred: row.get(9)?,
                error_message: row.get(10)?,
            })
        })
        .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;

    let mut result = Vec::new();
    for r in rows {
        result.push(r.map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?);
    }
    Ok(result)
}
