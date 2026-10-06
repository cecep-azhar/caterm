//! Integration and unit tests for Scheduler engine, retention policies, and SQLCipher CRUD.

use caterm_core::scheduler::{
    TaskExecutionLog, TaskInput, TaskStatus, TaskType, apply_retention_policy, calculate_next_run,
    delete_task, get_task_by_id, get_task_logs, list_tasks, log_execution, save_task,
};
use chrono::{Duration, TimeZone, Utc};
use std::fs::File;
use std::io::Write;

#[test]
fn test_scheduler_interval_calculations() {
    let base_time = Utc.with_ymd_and_hms(2026, 10, 6, 12, 0, 0).unwrap();

    // 1. Every Nh
    let next_6h = calculate_next_run("every 6h", Some(base_time)).expect("should parse 6h");
    assert_eq!(next_6h, base_time + Duration::hours(6));

    // 2. Every Nm
    let next_30m = calculate_next_run("every 30m", Some(base_time)).expect("should parse 30m");
    assert_eq!(next_30m, base_time + Duration::minutes(30));

    // 3. Every Nd
    let next_1d = calculate_next_run("every 1d", Some(base_time)).expect("should parse 1d");
    assert_eq!(next_1d, base_time + Duration::days(1));

    // 4. Invalid expressions
    assert!(calculate_next_run("", Some(base_time)).is_err());
    assert!(calculate_next_run("every 0h", Some(base_time)).is_err());
    assert!(calculate_next_run("every invalid", Some(base_time)).is_err());
}

#[test]
fn test_scheduler_cron_parsing() {
    let base_time = Utc.with_ymd_and_hms(2026, 10, 6, 1, 30, 0).unwrap();

    // "0 2 * * *" -> 02:00 UTC today
    let next_2am = calculate_next_run("0 2 * * *", Some(base_time)).expect("cron 0 2 * * *");
    let expected = Utc.with_ymd_and_hms(2026, 10, 6, 2, 0, 0).unwrap();
    assert_eq!(next_2am, expected);

    // If current time is 03:00, "0 2 * * *" should be 02:00 tomorrow
    let late_time = Utc.with_ymd_and_hms(2026, 10, 6, 3, 0, 0).unwrap();
    let next_day_2am =
        calculate_next_run("0 2 * * *", Some(late_time)).expect("cron next day 0 2 * * *");
    let expected_next_day = Utc.with_ymd_and_hms(2026, 10, 7, 2, 0, 0).unwrap();
    assert_eq!(next_day_2am, expected_next_day);
}

#[test]
fn test_retention_policy_file_rotation() {
    let temp_dir =
        std::env::temp_dir().join(format!("caterm_retention_test_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).expect("create test temp dir");

    // Create 5 backup files with varying timestamps
    for i in 1..=5 {
        let file_path = temp_dir.join(format!("backup_{i}.sql.gz"));
        let mut f = File::create(&file_path).expect("create dummy backup");
        writeln!(f, "dummy content {i}").expect("write dummy");
        // Sleep briefly to ensure different modified times
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    // Also create a temporary .part file (should NOT be removed or counted)
    let part_path = temp_dir.join("in_progress.sql.gz.part");
    {
        let mut f = File::create(&part_path).expect("create part file");
        writeln!(f, "partial content").expect("write part");
    }

    // Apply retention policy: keep only 3 newest files
    let deleted = apply_retention_policy(&temp_dir, 3).expect("apply retention policy");
    assert_eq!(deleted, 2, "Should delete exactly 2 oldest files");

    // Check .part file still exists
    assert!(part_path.exists(), ".part file must not be deleted");

    // Check remaining regular files count is 3
    let remaining_files: Vec<_> = std::fs::read_dir(&temp_dir)
        .expect("read dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) != Some("part"))
        .collect();
    assert_eq!(
        remaining_files.len(),
        3,
        "Only 3 backup files should remain"
    );

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_sqlcipher_scheduled_tasks_and_logs_crud() {
    let _data_dir = caterm_core::test_support::isolated_data_dir("scheduler_crud");
    caterm_core::vault::unlock_vault("testpass123456").expect("unlock test vault");

    // 1. Initial list should be empty
    let initial_tasks = list_tasks().expect("list tasks");
    assert!(initial_tasks.is_empty());

    // 2. Create a new task (SFTP Backup)
    let input = TaskInput {
        id: None,
        name: "Nightly VPS Backup".to_string(),
        description: Some("Backup MySQL DB & /var/www".to_string()),
        task_type: TaskType::SftpBackup,
        host_id: None,
        schedule_expr: "0 2 * * *".to_string(),
        is_enabled: true,
        command_script: None,
        timeout_seconds: 300,
        remote_src_path: Some("/tmp/backup.tar.gz".to_string()),
        local_dest_dir: Some("/home/cecepazhar/Backups/vps".to_string()),
        remote_pre_cmd: Some("tar -czf /tmp/backup.tar.gz /var/www".to_string()),
        remote_post_cmd: Some("rm -f /tmp/backup.tar.gz".to_string()),
        retention_count: 5,
        notify_on_success: false,
        notify_on_failure: true,
    };

    let created = save_task(input).expect("save task");
    assert_eq!(created.name, "Nightly VPS Backup");
    assert_eq!(created.retention_count, 5);
    assert_eq!(created.task_type, TaskType::SftpBackup);

    // 3. Fetch by ID
    let fetched = get_task_by_id(&created.id)
        .expect("get task")
        .expect("task must exist");
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.name, "Nightly VPS Backup");

    // 4. Log execution
    let log = TaskExecutionLog {
        id: uuid::Uuid::new_v4().to_string(),
        task_id: created.id.clone(),
        started_at: Utc::now().to_rfc3339(),
        finished_at: Some(Utc::now().to_rfc3339()),
        duration_ms: Some(1500),
        exit_code: Some(0),
        status: TaskStatus::Success,
        stdout: Some("Backup pull complete".to_string()),
        stderr: None,
        bytes_transferred: Some(1024 * 1024 * 15), // 15 MB
        error_message: None,
    };

    log_execution(&log).expect("log execution");

    // 5. Query execution logs
    let logs = get_task_logs(&created.id).expect("get logs");
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].status, TaskStatus::Success);
    assert_eq!(logs[0].bytes_transferred, Some(1024 * 1024 * 15));

    // 6. Delete task
    delete_task(&created.id).expect("delete task");
    let after_delete = get_task_by_id(&created.id).expect("get deleted task");
    assert!(after_delete.is_none());
}
