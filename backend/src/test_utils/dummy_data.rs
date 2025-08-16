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

    pub fn all() -> [User; 4] { [number1(), number2(), number3(), number4()] }
}
