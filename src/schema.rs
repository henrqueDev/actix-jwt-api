// @generated automatically by Diesel CLI.

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
