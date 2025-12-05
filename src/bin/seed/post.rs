use crate::time_utils::anchor_offset;
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

struct SeedPost {
    author_id: Option<i32>,
    parent_id: Option<i32>,
    body: Option<&'static str>,
    created_at: DateTime<Utc>,
    edited_at: Option<DateTime<Utc>>,
    archived_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
}

/// Inserts seed posts into the database. Assumes users with IDs 1, 2, 3, 4, 5, and 6 already exist.
///
/// User 1 (Spurt) will be the author of the initial root post inserted. Users 2 through 6 each
/// have a post in response to the root post. 3 then responds to 2's post, and then 2 responds back
/// to that response.
pub async fn seed(pool: &PgPool) -> Result<()> {
    let posts = [
        // Post 1
        SeedPost {
            author_id: Some(1), // Spurt
            parent_id: None,    // In the database, only one post can have a NULL parent ID
            body: Some(POST_BODIES[0]),
            created_at: anchor_offset(0, 0, 0)?,
            edited_at: None,
            archived_at: None,
            deleted_at: None,
        },
        // Post 2
        SeedPost {
            author_id: Some(2),
            parent_id: Some(1),
            body: Some(POST_BODIES[1]),
            created_at: anchor_offset(1000, 11, 11)?,
            edited_at: None,
            archived_at: None,
            deleted_at: None,
        },
        // Post 3
        SeedPost {
            author_id: Some(3),
            parent_id: Some(1),
            body: Some(POST_BODIES[2]),
            created_at: anchor_offset(1500, 4, 25)?,
            edited_at: Some(anchor_offset(1533, 4, 30)?),
            archived_at: None,
            deleted_at: None,
        },
        // Post 4
        SeedPost {
            author_id: Some(4),
            parent_id: Some(1),
            body: Some(POST_BODIES[3]),
            created_at: anchor_offset(2000, 0, 50)?,
            edited_at: None,
            archived_at: None,
            deleted_at: None,
        },
        // Post 5
        SeedPost {
            author_id: Some(5),
            parent_id: Some(1),
            body: Some(POST_BODIES[4]),
            created_at: anchor_offset(2100, 5, 52)?,
            edited_at: Some(anchor_offset(2150, 10, 25)?),
            archived_at: None,
            deleted_at: None,
        },
        // Post 6
        SeedPost {
            author_id: Some(6),
            parent_id: Some(1),
            body: Some(POST_BODIES[5]),
            created_at: anchor_offset(2244, 4, 44)?,
            edited_at: None,
            archived_at: None,
            deleted_at: None,
        },
        // Post 7
        SeedPost {
            author_id: Some(3),
            parent_id: Some(2),
            body: Some(POST_BODIES[6]),
            created_at: anchor_offset(2300, 15, 50)?,
            edited_at: None,
            archived_at: None,
            deleted_at: None,
        },
        // Post 8
        SeedPost {
            author_id: Some(2),
            parent_id: Some(7),
            body: Some(POST_BODIES[7]),
            created_at: anchor_offset(2345, 23, 0)?,
            edited_at: Some(anchor_offset(2350, 23, 5)?),
            archived_at: None,
            deleted_at: None,
        },
    ];

    for post in posts {
        sqlx::query!(
            "
            INSERT INTO post (author_id, parent_id, body, created_at,
                              edited_at, archived_at, deleted_at)
            VALUES ($1, $2, $3::text, $4, $5, $6, $7)
            ",
            post.author_id,
            post.parent_id,
            post.body,
            post.created_at,
            post.edited_at,
            post.archived_at,
            post.deleted_at,
        )
        .execute(pool)
        .await?;
    }

    log::info!("Seeded posts");

    Ok(())
}

const POST_BODIES: [&str; 8] = [
    // Post 1 (the root post)
    "ROOT POST: Welcome to Spur! Happy posting!",
    // Post 2
    "I’ve gravitated toward Rust because it gives me the control of systems programming without the constant anxiety of undefined behavior. The ownership model might feel strict at first, but once it “clicks,” you start to appreciate how it forces you to think clearly about lifetimes and data flow. I used to spend hours tracking down memory leaks or race conditions in C++, but with Rust, the compiler does that heavy lifting. It’s like having a very stern but extremely competent coworker who won’t let you push broken code to production.",
    // Post 3
    // cspell:disable-next
    "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Phasellus sit amet sapien ac eros hendrerit elementum. Praesent id velit vitae arcu convallis facilisis. Suspendisse nec lorem at nulla commodo condimentum. Mauris vel metus porttitor, fringilla velit id, tincidunt leo. Integer vehicula nulla a libero vestibulum, vel cursus sem viverra. Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia curae; Sed imperdiet lacus a massa varius, sit amet scelerisque felis sagittis.",
    // Post 4
    // cspell:disable-next
    "Integer varius, elit vel commodo fermentum, purus turpis facilisis justo, nec dictum lacus nisl non mi. Cras gravida, lorem ac facilisis egestas, nunc lorem efficitur mauris, ut ullamcorper ante metus ac risus. Maecenas at nisi ac nisl ultricies cursus. Aliquam erat volutpat. \n\nVivamus viverra quam eu ligula fermentum, non feugiat justo vestibulum. Donec auctor metus id lorem pharetra, vitae imperdiet odio cursus. Etiam gravida, nisl id feugiat tempor, justo lorem iaculis leo, nec porta sem nisl eget justo.",
    // Post 5
    // cspell:disable-next
    "Curabitur malesuada, nunc ut tincidunt tincidunt, justo nulla tincidunt magna, nec varius urna leo id tortor. Vestibulum ac turpis non justo tincidunt gravida. In sit amet convallis elit. Nulla facilisi. Fusce feugiat ultricies erat, id interdum augue lacinia a. Suspendisse faucibus ante ut nulla ullamcorper mattis. \n\nSed in libero et lectus consequat volutpat. Cras dapibus, nisi id dictum ultricies, arcu augue malesuada purus, vel tincidunt magna nunc ac diam. Pellentesque et urna sit amet lectus maximus rhoncus. Integer eu dolor et massa vehicula convallis nec id justo.",
    // Post 6
    // cspell:disable-next
    "Aenean rhoncus sagittis lectus, vitae luctus mi maximus ac. Suspendisse at libero eget ex tincidunt egestas. Cras non diam quis sapien sodales vehicula. Nulla vel sem et mi faucibus blandit. Sed sit amet orci ut justo mattis rhoncus. \n\nMorbi tincidunt malesuada nisl, at egestas odio sodales non. Proin ac facilisis risus, a gravida libero. Vivamus id risus fermentum, lacinia mi vel, convallis sem. Integer vehicula lacinia nunc a pulvinar. Nam vestibulum urna quis nunc vestibulum condimentum. \n\nDuis ut metus a leo pretium feugiat. Sed sagittis purus id sapien efficitur, a malesuada odio hendrerit. Nullam at lacus at eros hendrerit mattis. Nunc nec justo lorem. Pellentesque habitant morbi tristique senectus et netus et malesuada fames ac turpis egestas.",
    // Post 7
    "I can see why that appeals to you, but to me, Rust sometimes feels like it sacrifices productivity for safety. The borrow checker is powerful, sure, but I’ve had situations where fighting it took longer than just writing the code in a higher-level language. For example, in Python or Go, I can iterate rapidly on prototypes without worrying about the compiler blocking my progress every five minutes. I think there’s a trade-off that Rust enthusiasts sometimes gloss over. \n\nThat said, I do admire the level of performance and safety it delivers. The type system is elegant, and zero-cost abstractions are no joke. When I worked on a small concurrent data processing tool, Rust’s thread safety guarantees saved me from subtle synchronization bugs that would have been a nightmare in C. So while I still find it cumbersome at times, I can’t deny that it earns its reputation.",
    // Post 8
    "You’re absolutely right that Rust demands more up-front effort—it’s not a “move fast and break things” language. But I’ve found that once you get through the learning curve, development actually accelerates because you spend so much less time debugging. I’d argue that the compiler errors are more like real-time code reviews: they don’t just tell you something’s wrong; they often explain why and how to fix it. That kind of feedback loop makes me a better programmer in the long run. \n\nPrototyping in Rust used to be painful, but with tools like cargo watch, faster build times, and libraries like anyhow for flexible error handling, it’s gotten a lot smoother. I can now get from idea to working prototype surprisingly quickly, and the result is something I don’t have to rewrite later for performance or safety reasons. That’s not something I can say for most languages. \n\nUltimately, I think it comes down to intent. If I’m building a quick one-off script, sure, I’ll reach for Python. But if I’m creating something that’s meant to last—something where correctness and performance matter—Rust feels like an investment that keeps paying off. The confidence I have in the resulting code is worth every minute of wrestling with the borrow checker.",
];
