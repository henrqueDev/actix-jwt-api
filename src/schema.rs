// @generated automatically by Diesel CLI.

diesel::table! {
    models (id) {
        id -> Int4,
        #[max_length = 128]
        name -> Varchar,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    products (id) {
        id -> Int4,
        #[max_length = 255]
        sku -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        description -> Text,
        price -> Float4,
        weight -> Float4,
        dimension_height -> Float4,
        dimension_width -> Float4,
        dimension_depth -> Float4,
        model_id -> Int4,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    user_types (id) {
        id -> Int4,
        #[max_length = 32]
        type_name -> Varchar,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
        email -> Varchar,
        password -> Nullable<Varchar>,
        user_type_id -> Int4,
        two_factor_secret -> Nullable<Text>,
        two_factor_recovery_code -> Nullable<Text>,
        two_factor_confirmed_at -> Nullable<Timestamptz>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::joinable!(products -> models (model_id));
diesel::joinable!(users -> user_types (user_type_id));

diesel::allow_tables_to_appear_in_same_query!(
    models,
    products,
    user_types,
    users,
);
