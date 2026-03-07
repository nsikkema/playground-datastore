use datastore::definition::{
    BasicDefinition, MapDefinition, ObjectDefinition, PropertyDefinition, StructDefinition,
    StructItemDefinition, TableDefinition,
};
use datastore::path;
use datastore::store::traits::ProxyStoreTrait;
use datastore::store::{Store, StorePath};

fn main() {
    // 1. Create a Store
    let store = Store::new(Default::default());

    // 2. Define a Struct
    // A Struct is a reusable nested structure with named fields.
    let address_def = StructDefinition::new(
        "Address",
        vec![
            (
                "street",
                StructItemDefinition::Basic(BasicDefinition::new_string("")),
            ),
            (
                "city",
                StructItemDefinition::Basic(BasicDefinition::new_string("")),
            ),
        ],
    )
    .unwrap();

    // 3. Define a Map
    // A Map stores multiple instances of a Struct, keyed by strings.
    let contacts_def = MapDefinition::new("Contacts", address_def.clone());

    // 4. Define a Table
    // A Table is a collection of rows with fixed columns.
    let inventory_def = TableDefinition::new(
        "Inventory",
        vec![
            ("item_id", BasicDefinition::new_string("")),
            ("quantity", BasicDefinition::new_number("0")),
        ],
    )
    .unwrap();

    // 5. Create an Object with these components
    let mut builder = ObjectDefinition::builder("Company Data");
    builder
        .add(
            "hq_address",
            PropertyDefinition::new("Headquarters", address_def),
        )
        .unwrap();
    builder
        .add(
            "branches",
            PropertyDefinition::new("Branch Offices", contacts_def),
        )
        .unwrap();
    builder
        .add(
            "stock",
            PropertyDefinition::new("Warehouse Stock", inventory_def),
        )
        .unwrap();
    let company_def = builder.finish();

    store
        .create_object(&"my_company".into(), &company_def)
        .unwrap();

    let mut company_proxy = store.get_object(&"my_company".into()).unwrap();
    let company_key = "my_company";

    // Access the 'hq_address' struct
    // We can now use the 'path!' macro for more ergonomic path construction.
    let street_path = path!(company_key / "hq_address" / "street");

    let mut street_proxy = store.get_basic(&street_path).unwrap();
    street_proxy.set_value("123 Main St");
    street_proxy.push().unwrap();

    println!("HQ Street: {}", street_proxy.get_value().unwrap());

    // 7. Interact with the Map
    // Maps allow inserting new entries that follow the defined Struct schema.
    let branches_proxy = company_proxy.get_container("branches").unwrap();

    // Insert a new branch "london"
    branches_proxy.insert_map_entry("london").unwrap();

    // Now we can access the 'london' branch fields using a tuple for ergonomics.
    let london_city_path: StorePath = ("my_company", "branches", "london", "city").into();

    let mut london_city_proxy = store.get_basic(&london_city_path).unwrap();
    london_city_proxy.set_value("London");
    london_city_proxy.push().unwrap();

    println!(
        "Branch 'london' city: {}",
        london_city_proxy.get_value().unwrap()
    );

    // 8. Interact with the Table
    let mut table_proxy = company_proxy.get_table("stock").unwrap();

    // Add some rows
    table_proxy.append_row();
    table_proxy
        .set_cell(0, "item_id", "widget_a".into())
        .unwrap();
    table_proxy.set_cell(0, "quantity", "50".into()).unwrap();

    table_proxy.append_row();
    table_proxy
        .set_cell(1, "item_id", "widget_b".into())
        .unwrap();
    table_proxy.set_cell(1, "quantity", "25".into()).unwrap();

    table_proxy.push().unwrap();

    println!("Stock table rows: {}", table_proxy.row_count());
    for i in 0..table_proxy.row_count() {
        let row = table_proxy.row(i).unwrap();
        println!(
            "  Row {}: ID={}, Qty={}",
            i, row["item_id"], row["quantity"]
        );
    }
}
