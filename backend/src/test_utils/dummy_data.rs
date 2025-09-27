pub mod user {
    use crate::models::user::User;
    use chrono::{Days, Months, Utc};

    pub fn number1() -> User {
        User {
            id: 41,
            name: String::from("Friendly Good"),
            email: String::from("good@friend.co"),
            username: String::from("my_friend_5"),
            password_hash: String::from("ab5iub$@1i&g"),
            created_at: Utc::now(),
        }
    }

    pub fn number2() -> User {
        User {
            id: 42,
            name: String::from("Gillian Jill"),
            email: String::from("gillian@jill.org"),
            username: String::from("jill_plus_ian"),
            password_hash: String::from("aab52i4n&$"),
            created_at: Utc::now()
                .checked_sub_days(Days::new(1))
                .expect("failed to subtract one day from now"),
        }
    }

    pub fn number3() -> User {
        User {
            id: 43,
            name: String::from("Harold Old"),
            email: String::from("harold@old.jp"),
            username: String::from("old_hare"),
            password_hash: String::from("ljb42b50%&$@"),
            created_at: Utc::now()
                .checked_sub_months(Months::new(1))
                .expect("failed to subtract one month from now"),
        }
    }

    pub fn number4() -> User {
        User {
            id: 44,
            name: String::from("Greg Egg"),
            email: String::from("egg_greg@egg.gg"),
            username: String::from("greg_the_egg"),
            password_hash: String::from("5%2b@$$@bu"),
            created_at: Utc::now()
                .checked_sub_months(Months::new(6))
                .expect("failed to subtract six months from now"),
        }
    }

    // pub fn all() -> [User; 4] { [number1(), number2(), number3(), number4()] }
}

pub mod post {
    use crate::{models::post::Post, test_utils::dummy_data::post_with_author};

    pub fn three_dummies() -> [Post; 3] {
        let [p1, p2, p3] = post_with_author::three_dummies();
        [p1.into(), p2.into(), p3.into()]
    }
}

pub mod post_with_author {
    use crate::models::post::PostWithAuthor;
    use chrono::{TimeZone, Utc};

    pub fn three_dummies() -> [PostWithAuthor; 3] {
        [
            PostWithAuthor {
                id: 24,
                author_id: Some(255),
                parent_id: Some(42),
                body: Some(String::from("cool post body")),
                created_at: Utc.timestamp_millis_opt(29_489_571).unwrap(),
                edited_at: None,
                archived_at: None,
                deleted_at: None,
                author_username: Some(String::from("jack54444mack")),
            },
            PostWithAuthor {
                id: 999,
                author_id: Some(2431),
                parent_id: Some(94),
                body: Some(String::from("one two three test post")),
                created_at: Utc.timestamp_millis_opt(249_982_133).unwrap(),
                edited_at: Some(Utc.timestamp_millis_opt(444_843_343).unwrap()),
                archived_at: None,
                deleted_at: None,
                author_username: Some(String::from("helmet_man")),
            },
            PostWithAuthor {
                id: 1324,
                author_id: Some(44),
                parent_id: Some(5432),
                body: Some(String::from("hello from the world 🗺️")),
                created_at: Utc.timestamp_millis_opt(294_424).unwrap(),
                edited_at: None,
                archived_at: Some(Utc.timestamp_millis_opt(442_422).unwrap()),
                deleted_at: Some(Utc.timestamp_millis_opt(99_942_901).unwrap()),
                author_username: Some(String::from("aunt_flo")),
            },
        ]
    }
}
