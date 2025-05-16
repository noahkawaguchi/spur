use anyhow::{Result, anyhow};

enum FriendshipState {
    Friends,
    PendingFromFirst,
    PendingFromSecond,
    Nil,
}

struct FriendshipRepo {
    pool: sqlx::PgPool,
}

impl FriendshipRepo {
    async fn new_request(
        &self,
        first_user_id: i32,
        second_user_id: i32,
        requester_id: i32,
    ) -> sqlx::Result<()> {
        let _ = sqlx::query!(
            "
            INSERT INTO friendships (first_user_id, second_user_id, requester_id) 
            VALUES ($1, $2, $3)
            ",
            first_user_id,
            second_user_id,
            requester_id,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn accept_request(&self, first_user_id: i32, second_user_id: i32) -> sqlx::Result<()> {
        let _ = sqlx::query!(
            "
            UPDATE friendships
            SET confirmed = TRUE, confirmed_at = CURRENT_TIMESTAMP
            WHERE first_user_id = $1 AND second_user_id = $2
            ",
            first_user_id,
            second_user_id,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_friendship_state(
        &self,
        first_user_id: i32,
        second_user_id: i32,
    ) -> Result<FriendshipState> {
        let row = sqlx::query!(
            "
            SELECT requester_id, confirmed FROM friendships
            WHERE first_user_id = $1 AND second_user_id = $2
            ",
            first_user_id,
            second_user_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        let state = match row {
            None => FriendshipState::Nil,
            Some(friends) if friends.confirmed => FriendshipState::Friends,
            Some(pair) if pair.requester_id == first_user_id => FriendshipState::PendingFromFirst,
            Some(pair) if pair.requester_id == second_user_id => FriendshipState::PendingFromSecond,
            Some(_) => return Err(anyhow!("unexpected values checking friendship state")),
        };

        Ok(state)
    }

    async fn get_friends(&self, id: i32) -> sqlx::Result<Vec<i32>> {
        let friends = sqlx::query!(
            "
            SELECT
                CASE
                    WHEN first_user_id = $1 THEN second_user_id
                    ELSE first_user_id
                END AS friend_id
            FROM friendships
            WHERE confirmed = TRUE
            AND (first_user_id = $1 OR second_user_id = $1)
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
            SELECT requester_id FROM friendships
            WHERE confirmed = FALSE
            AND (first_user_id = $1 OR second_user_id = $1)
            AND requester_id != $1
            ",
            id,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(requesters.into_iter().map(|r| r.requester_id).collect())
    }
}
