use crate::services::friendship_svc::FriendshipStore;

pub enum FriendshipStatus {
    /// The two users are confirmed friends.
    Friends,
    /// There is a pending request from the user with the contained ID.
    PendingFrom(i32),
    /// There is no existing relationship between the two users.
    Nil,
}

pub struct FriendshipRepo {
    pool: sqlx::PgPool,
}

impl FriendshipStore for FriendshipRepo {
    async fn new_request(
        &self,
        first_id: i32,
        second_id: i32,
        requester_id: i32,
    ) -> sqlx::Result<()> {
        let _ = sqlx::query!(
            "
            INSERT INTO friendships (first_id, second_id, requester_first) 
            VALUES ($1, $2, $3)
            ",
            first_id,
            second_id,
            requester_id == first_id,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn accept_request(&self, first_id: i32, second_id: i32) -> sqlx::Result<()> {
        let _ = sqlx::query!(
            "
            UPDATE friendships
            SET confirmed = TRUE, confirmed_at = CURRENT_TIMESTAMP
            WHERE first_id = $1 AND second_id = $2
            ",
            first_id,
            second_id,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_status(&self, first_id: i32, second_id: i32) -> sqlx::Result<FriendshipStatus> {
        let row = sqlx::query!(
            "
            SELECT requester_first, confirmed FROM friendships
            WHERE first_id = $1 AND second_id = $2
            ",
            first_id,
            second_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        let status = match row {
            None => FriendshipStatus::Nil,
            Some(friends) if friends.confirmed => FriendshipStatus::Friends,
            Some(pair) if pair.requester_first => FriendshipStatus::PendingFrom(first_id),
            Some(_) => FriendshipStatus::PendingFrom(second_id),
        };

        Ok(status)
    }

    async fn get_friends(&self, id: i32) -> sqlx::Result<Vec<i32>> {
        let friends = sqlx::query!(
            "
            SELECT
                CASE
                    WHEN first_id = $1 THEN second_id
                    ELSE first_id
                END AS friend_id
            FROM friendships
            WHERE confirmed
            AND (first_id = $1 OR second_id = $1)
            ",
            id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(friends.into_iter().filter_map(|f| f.friend_id).collect())
    }

    async fn get_requests(&self, id: i32) -> sqlx::Result<Vec<i32>> {
        let requesters = sqlx::query!(
            "
            SELECT first_id AS requester_id FROM friendships
            WHERE NOT confirmed
            AND second_id = $1
            AND requester_first

            UNION ALL

            SELECT second_id AS requester_id FROM friendships
            WHERE NOT confirmed
            AND first_id = $1
            AND NOT requester_first
            ",
            id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(requesters
            .into_iter()
            .filter_map(|r| r.requester_id)
            .collect())
    }
}
