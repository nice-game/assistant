table! {
    ai_requests (id) {
        id -> Int4,
        session_id -> Int4,
        query -> Text,
        query_created -> Timestamptz,
        reply -> Nullable<Text>,
        reply_created -> Nullable<Timestamptz>,
    }
}

table! {
    sessions (id) {
        id -> Int4,
        uuid -> Uuid,
        user_id -> Nullable<Int4>,
        expired -> Bool,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Text,
        password -> Bytea,
        salt -> Bytea,
    }
}

joinable!(ai_requests -> sessions (session_id));
joinable!(sessions -> users (user_id));

allow_tables_to_appear_in_same_query!(
    ai_requests,
    sessions,
    users,
);
