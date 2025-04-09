// @generated automatically by Diesel CLI.

diesel::table! {
    product_categories (id) {
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
        product_category_id -> Int4,
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
        two_factor_secret -> Nullable<Text>,
        two_factor_recovery_code -> Nullable<Text>,
        two_factor_confirmed_at -> Nullable<Timestamptz>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::joinable!(products -> product_categories (product_category_id));

diesel::allow_tables_to_appear_in_same_query!(
    product_categories,
    products,
    users,
);
