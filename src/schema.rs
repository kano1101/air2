table! {
    categories (id) {
        id -> Integer,
        name -> Varchar,
    }
}

table! {
    histories (id) {
        id -> Integer,
        item_id -> Integer,
        price -> Integer,
        purchased_at -> Varchar,
    }
}

table! {
    items (id) {
        id -> Integer,
        category_id -> Integer,
        hash -> Varchar,
        name -> Varchar,
    }
}

joinable!(histories -> items (item_id));
joinable!(items -> categories (category_id));

allow_tables_to_appear_in_same_query!(
    categories,
    histories,
    items,
);
