use crate::error::AppError;
use sea_orm::{DbErr, RuntimeErr, sqlx};

pub fn map_db_error(err: DbErr) -> AppError {
    if let Some(mapped) = try_map_sqlx_error(&err) {
        return mapped;
    }
    AppError::InternalServerError { source: err.into() }
}

fn try_map_sqlx_error(err: &DbErr) -> Option<AppError> {
    let runtime_err = match err {
        DbErr::Exec(RuntimeErr::SqlxError(sqlx_err)) => sqlx_err,
        DbErr::Query(RuntimeErr::SqlxError(sqlx_err)) => sqlx_err,
        _ => return None,
    };

    let db_error = match runtime_err.as_ref() {
        sqlx::Error::Database(db_error) => db_error.as_ref(),
        _ => return None,
    };

    match db_error.code().as_deref() {
        Some("23505") => Some(map_unique_violation(db_error)),
        Some("23502") => Some(map_not_null_violation(db_error)),
        Some("23503") => Some(map_foreign_key_violation(db_error)),
        _ => None,
    }
}

fn map_unique_violation(db_error: &dyn sqlx::error::DatabaseError) -> AppError {
    let pg_error = db_error.try_downcast_ref::<sqlx::postgres::PgDatabaseError>();
    let entity = pg_error
        .and_then(|pg| pg.table())
        .map(|t| t.to_string())
        .or_else(|| db_error.constraint().map(|c| c.to_string()))
        .unwrap_or_else(|| "record".to_string());
    let (field, value) = parse_detail_key_value(
        pg_error.and_then(|pg| pg.detail()),
        pg_error.and_then(|pg| pg.column()),
    );
    AppError::Duplicate {
        entity,
        field,
        value,
    }
}

fn map_not_null_violation(db_error: &dyn sqlx::error::DatabaseError) -> AppError {
    let pg_error = db_error.try_downcast_ref::<sqlx::postgres::PgDatabaseError>();
    let field = pg_error
        .and_then(|pg| pg.column())
        .map(|c| c.to_string())
        .unwrap_or_else(|| "field".to_string());

    AppError::BadRequest(format!("'{field}' is required"))
}

fn map_foreign_key_violation(db_error: &dyn sqlx::error::DatabaseError) -> AppError {
    let pg_error = db_error.try_downcast_ref::<sqlx::postgres::PgDatabaseError>();
    let (field, value) = parse_detail_key_value(
        pg_error.and_then(|pg| pg.detail()),
        pg_error.and_then(|pg| pg.column()),
    );

    AppError::BadRequest(format!(
        "Referenced record for '{field}' with value '{value}' does not exist"
    ))
}

fn parse_detail_key_value(detail: Option<&str>, column: Option<&str>) -> (String, String) {
    if let Some(detail) = detail {
        if let Some(key_start) = detail.find("Key (") {
            let after_key = &detail[key_start + 5..];
            if let Some(field_end) = after_key.find(')') {
                let field = after_key[..field_end].to_string();
                if let Some(eq_idx) = after_key.find(")=") {
                    let after_eq = &after_key[eq_idx + 2..];
                    if after_eq.starts_with('(') {
                        let after_paren = &after_eq[1..];
                        if let Some(value_end) = after_paren.find(')') {
                            let value = after_paren[..value_end].to_string();
                            return (field, value);
                        }
                    } else if let Some(value_end) = after_eq.find(')') {
                        let value = after_eq[..value_end].to_string();
                        return (field, value);
                    }
                }
                return (field, String::new());
            }
        }
    }

    let fallback_field = column
        .map(|col| col.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    (fallback_field, String::new())
}
