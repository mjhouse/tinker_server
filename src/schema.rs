// @generated automatically by Diesel CLI.

diesel::table! {
    movements (id) {
        id -> Int4,
        player -> Int4,
        vector -> Array<Nullable<Float8>>,
        created -> Timestamp,
    }
}
