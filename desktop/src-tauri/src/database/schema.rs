// @generated automatically by Diesel CLI.

diesel::table! {
    /// Representation of the `books` table.
    ///
    /// (Automatically generated by Diesel.)
    books (id) {
        /// The `id` column of the `books` table.
        ///
        /// Its SQL type is `Integer`.
        ///
        /// (Automatically generated by Diesel.)
        id -> Integer,
        /// The `path` column of the `books` table.
        ///
        /// Its SQL type is `Text`.
        ///
        /// (Automatically generated by Diesel.)
        path -> Text,
        /// The `title` column of the `books` table.
        ///
        /// Its SQL type is `Text`.
        ///
        /// (Automatically generated by Diesel.)
        title -> Text,
        /// The `cover` column of the `books` table.
        ///
        /// Its SQL type is `Text`.
        ///
        /// (Automatically generated by Diesel.)
        cover -> Text,
        /// The `rating` column of the `books` table.
        ///
        /// Its SQL type is `Integer`.
        ///
        /// (Automatically generated by Diesel.)
        rating -> Integer,
    }
}

diesel::table! {
    /// Representation of the `collections` table.
    ///
    /// (Automatically generated by Diesel.)
    collections (id) {
        /// The `id` column of the `collections` table.
        ///
        /// Its SQL type is `Integer`.
        ///
        /// (Automatically generated by Diesel.)
        id -> Integer,
        /// The `name` column of the `collections` table.
        ///
        /// Its SQL type is `Text`.
        ///
        /// (Automatically generated by Diesel.)
        name -> Text,
    }
}

diesel::table! {
    /// Representation of the `folders` table.
    ///
    /// (Automatically generated by Diesel.)
    folders (id) {
        /// The `id` column of the `folders` table.
        ///
        /// Its SQL type is `Integer`.
        ///
        /// (Automatically generated by Diesel.)
        id -> Integer,
        /// The `path` column of the `folders` table.
        ///
        /// Its SQL type is `Text`.
        ///
        /// (Automatically generated by Diesel.)
        path -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(books, collections, folders,);
