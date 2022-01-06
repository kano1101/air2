table! {
    histories (id) {
        id -> Integer,
        item_id -> Nullable<Integer>,
        price -> Nullable<Integer>,
        purchased_at -> Nullable<Varchar>,
    }
}

table! {
    items (id) {
        id -> Integer,
        hash -> Varchar,
        name -> Varchar,
    }
}

joinable!(histories -> items (item_id));

allow_tables_to_appear_in_same_query!(
    histories,
    items,
);
