// @generated automatically by Diesel CLI.

diesel::table! {
    books (id) {
        id -> Integer,
        path -> Text,
        title -> Text,
        cover -> Text,
        rating -> Integer,
    }
}

diesel::table! {
    collections (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    folders (id) {
        id -> Integer,
        path -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    books,
    collections,
    folders,
);
