use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::operations::canister;
use crate::lib::root_key::fetch_root_key_if_needed;
use candid::Principal;
use clap::Parser;
use dfx_core::identity::CallSender;
use ic_utils::interfaces::management_canister::FetchCanisterLogsResponse;
use slog::info;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

/// Get the canister logs.
#[derive(Parser)]
pub struct LogsOpts {
    /// Specifies the name or id of the canister to get its canister information.
    canister: String,
}

fn format_bytes(bytes: &[u8]) -> String {
    format!("(bytes) 0x{}", hex::encode(bytes))
}

fn format_canister_logs(logs: FetchCanisterLogsResponse) -> Vec<String> {
    logs.canister_log_records
        .into_iter()
        .map(|r| {
            let time = OffsetDateTime::from_unix_timestamp_nanos(r.timestamp_nanos as i128)
                .expect("invalid timestamp");

            let message = if let Ok(s) = String::from_utf8(r.content.clone()) {
                if format!("{s:?}").contains("\\u{") {
                    format_bytes(&r.content)
                } else {
                    s
                }
            } else {
                format_bytes(&r.content)
            };

            format!(
                "[{}. {}]: {}",
                r.idx,
                time.format(&Rfc3339).unwrap(),
                message
            )
        })
        .collect()
}

#[test]
fn test_format_canister_logs() {
    use ic_utils::interfaces::management_canister::CanisterLogRecord;

    let logs = FetchCanisterLogsResponse {
        canister_log_records: vec![
            CanisterLogRecord {
                idx: 42,
                timestamp_nanos: 1_620_328_630_010_000_000,
                content: b"Some text message".to_vec(),
            },
            CanisterLogRecord {
                idx: 43,
                timestamp_nanos: 1_620_328_630_020_000_000,
                content: vec![1, 2, 3, 10, 11, 12],
            },
        ],
    };
    assert_eq!(
        format_canister_logs(logs),
        vec![
            "[42. 2021-05-06T19:17:10.010Z]: Some text message".to_string(),
            "[43. 2021-05-06T19:17:10.020Z]: (bytes) 0x0102030a0b0c".to_string(),
        ],
    );
}

pub async fn exec(env: &dyn Environment, opts: LogsOpts, call_sender: &CallSender) -> DfxResult {
    let log = env.get_logger();

    let callee_canister = opts.canister.as_str();
    let canister_id_store = env.get_canister_id_store()?;

    let canister_id = Principal::from_text(callee_canister)
        .or_else(|_| canister_id_store.get(callee_canister))?;

    fetch_root_key_if_needed(env).await?;

    let logs = canister::get_canister_logs(env, canister_id, call_sender).await?;

    info!(log, "{}", format_canister_logs(logs).join("\n"));

    Ok(())
}