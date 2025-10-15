use crate::time_utils::anchor_offset;
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

struct SeedFriendship {
    lesser_id: i32,
    greater_id: i32,
    lesser_requested: bool,
    requested_at: DateTime<Utc>,
    confirmed_at: Option<DateTime<Utc>>,
}

/// Inserts seed friendships into the database. Assumes users with IDs 1, 2, 3, 4, 5, and 6 already
/// exist.
///
/// User 1 is the "Spurt" character, and is everyone's friend to start.
///
/// Also creates the following human relationships from the perspective of user 2:
///
/// - User 3: I requested, they accepted.
/// - User 4: They requested, I accepted.
/// - User 5: They requested, I have not accepted.
/// - User 6: No relation.
///
/// Also creates an accepted friendship requested from 6 to 3.
pub async fn seed(pool: &PgPool) -> Result<()> {
    // Regular human relationships
    let mut friendships = vec![
        SeedFriendship {
            lesser_id: 2,
            greater_id: 3,
            lesser_requested: true,
            requested_at: anchor_offset(3, 0, 0)?,
            confirmed_at: Some(anchor_offset(5, 14, 0)?),
        },
        SeedFriendship {
            lesser_id: 2,
            greater_id: 4,
            lesser_requested: false,
            requested_at: anchor_offset(6, 5, 20)?,
            confirmed_at: Some(anchor_offset(24, 0, 5)?),
        },
        SeedFriendship {
            lesser_id: 2,
            greater_id: 5,
            lesser_requested: false,
            requested_at: anchor_offset(120, 12, 0)?,
            confirmed_at: None,
        },
        SeedFriendship {
            lesser_id: 3,
            greater_id: 6,
            lesser_requested: false,
            requested_at: anchor_offset(222, 2, 2)?,
            confirmed_at: Some(anchor_offset(501, 8, 15)?),
        },
    ];

    // Everyone's friendships with Spurt
    for id in 2..=6 {
        friendships.push(SeedFriendship {
            lesser_id: 1,
            greater_id: id,
            lesser_requested: true,
            requested_at: anchor_offset(0, 0, 0)?,
            confirmed_at: Some(anchor_offset(0, 0, 0)?),
        });
    }

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
