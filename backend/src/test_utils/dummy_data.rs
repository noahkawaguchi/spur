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

// pub mod prompt_with_author {
//     use crate::models::prompt::PromptWithAuthor;

//     pub fn number1() -> PromptWithAuthor {
//         PromptWithAuthor {
//             id: 444,
//             author_username: String::from("any_username"),
//             body: String::from("any body here"),
//         }
//     }

//     pub fn number2() -> PromptWithAuthor {
//         PromptWithAuthor {
//             id: 6,
//             author_username: String::from("some_username123"),
//             body: String::from("How is it going?"),
//         }
//     }

//     pub fn number3() -> PromptWithAuthor {
//         PromptWithAuthor {
//             id: 554_258,
//             author_username: String::from("bobby_bob7"),
//             body: String::from("Hello whirled"),
//         }
//     }

//     pub fn all() -> [PromptWithAuthor; 3] { [number1(), number2(), number3()] }
// }

// pub mod post_with_prompt {
//     use crate::models::{post::PostWithPrompt, prompt::PromptWithAuthor};

//     pub fn number1() -> PostWithPrompt {
//         PostWithPrompt {
//             id: 8,
//             author_username: String::from("hellman"),
//             prompt: PromptWithAuthor {
//                 id: 25,
//                 author_username: String::from("janice"),
//                 body: String::from("Hello man"),
//             },
//             body: String::from("Hello man, what's up?"),
//         }
//     }

//     pub fn number2() -> PostWithPrompt {
//         PostWithPrompt {
//             id: 222,
//             author_username: String::from("abc321"),
//             prompt: PromptWithAuthor {
//                 id: 2552,
//                 author_username: String::from("another_username"),
//                 body: String::from("Another prompt body"),
//             },
//             body: String::from("Another post body"),
//         }
//     }

//     pub fn number3() -> PostWithPrompt {
//         PostWithPrompt {
//             id: 7,
//             author_username: String::from("creative_person"),
//             prompt: PromptWithAuthor {
//                 id: 9990,
//                 author_username: String::from("somewhat_creative"),
//                 body: String::from("What makes you creative?"),
//             },
//             body: String::from("I'm always creative"),
//         }
//     }

//     pub fn all() -> [PostWithPrompt; 3] { [number1(), number2(), number3()] }
// }
