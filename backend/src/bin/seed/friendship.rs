use anyhow::{Context, Result};
use chrono::{DateTime, Days, Utc};
use sqlx::PgPool;

struct SeedFriendship {
    lesser_id: i32,
    greater_id: i32,
    lesser_requested: bool,
    requested_at: DateTime<Utc>,
    confirmed_at: Option<DateTime<Utc>>,
}

/// Inserts seed friendships into the database. Assumes users with IDs 1, 2, 3, 4, and 5 already
/// exist.
///
/// Creates the following relationships from the perspective of user 1:
///
/// - User 2: I requested, they accepted.
/// - User 3: They requested, I accepted.
/// - User 4: They requested, I have not accepted.
/// - User 5: No relation.
///
/// Also creates an accepted friendship requested from 5 to 2.
pub async fn seed(pool: &PgPool) -> Result<()> {
    let friendships = [
        SeedFriendship {
            lesser_id: 1,
            greater_id: 2,
            lesser_requested: true,
            requested_at: days_ago(3)?,
            confirmed_at: Some(Utc::now()),
        },
        SeedFriendship {
            lesser_id: 1,
            greater_id: 3,
            lesser_requested: false,
            requested_at: days_ago(24)?,
            confirmed_at: Some(days_ago(6)?),
        },
        SeedFriendship {
            lesser_id: 1,
            greater_id: 4,
            lesser_requested: false,
            requested_at: days_ago(120)?,
            confirmed_at: None,
        },
        SeedFriendship {
            lesser_id: 2,
            greater_id: 5,
            lesser_requested: false,
            requested_at: days_ago(501)?,
            confirmed_at: Some(days_ago(222)?),
        },
    ];

    for friendship in friendships {
        sqlx::query!(
            "INSERT INTO friendship VALUES ($1, $2, $3, $4, $5)",
            friendship.lesser_id,
            friendship.greater_id,
            friendship.lesser_requested,
            friendship.requested_at,
            friendship.confirmed_at,
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

fn days_ago(n: u64) -> Result<DateTime<Utc>> {
    Utc::now()
        .checked_sub_days(Days::new(n))
        .context("Failed to perform DateTime arithmetic")
}
