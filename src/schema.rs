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
        name -> Nullable<Varchar>,
        description -> Nullable<Text>,
        price -> Money,
        weight -> Float4,
        dimension_height -> Nullable<Float8>,
        dimension_width -> Nullable<Float8>,
        dimension_depth -> Nullable<Float8>,
        category_id -> Int4,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
        email -> Varchar,
        password -> Varchar,
        two_factor_secret -> Nullable<Text>,
        two_factor_recovery_code -> Nullable<Text>,
        two_factor_confirmed_at -> Nullable<Timestamptz>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::joinable!(products -> product_categories (category_id));

diesel::allow_tables_to_appear_in_same_query!(
    product_categories,
    products,
    users,
);
