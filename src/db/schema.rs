// @generated automatically by Diesel CLI.

diesel::table! {
    posts (id) {
        id -> Integer,
        user_id -> Integer,
        space_id -> Integer,
        title -> Varchar,
        uri -> Varchar,
        body -> Text,
        published -> Bool,
        updated_at -> Nullable<Datetime>,
    }
}

diesel::table! {
    sessions (id) {
        id -> Integer,
        user_id -> Integer,
        session_id -> Varchar,
        start_date -> Nullable<Datetime>,
        end_date -> Nullable<Datetime>,
    }
}

diesel::table! {
    spaces (id) {
        id -> Integer,
        name -> Varchar,
        long_name -> Varchar,
        owner_id -> Integer,
        is_public -> Bool,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        username -> Varchar,
        password -> Varchar,
        is_admin -> Bool,
    }
}

diesel::joinable!(posts -> spaces (space_id));
diesel::joinable!(posts -> users (user_id));
diesel::joinable!(sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    posts,
    sessions,
    spaces,
    users,
);
