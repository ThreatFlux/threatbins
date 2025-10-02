use office_sdk::OfficeDocument;
use office_sdk::xlsx::XlsxDocument;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = XlsxDocument::new();

    let sales_data = vec![
        vec![
            "Product".to_string(),
            "Price".to_string(),
            "Quantity".to_string(),
            "Total".to_string(),
        ],
        vec![
            "Laptop".to_string(),
            "1200.00".to_string(),
            "5".to_string(),
            "6000.00".to_string(),
        ],
        vec![
            "Mouse".to_string(),
            "25.00".to_string(),
            "20".to_string(),
            "500.00".to_string(),
        ],
        vec![
            "Keyboard".to_string(),
            "75.00".to_string(),
            "10".to_string(),
            "750.00".to_string(),
        ],
    ];

    doc.add_worksheet("Sales", &sales_data)?;

    let employee_data = vec![
        vec![
            "Name".to_string(),
            "Department".to_string(),
            "Salary".to_string(),
        ],
        vec![
            "John Doe".to_string(),
            "Engineering".to_string(),
            "75000".to_string(),
        ],
        vec![
            "Jane Smith".to_string(),
            "Marketing".to_string(),
            "65000".to_string(),
        ],
        vec![
            "Bob Johnson".to_string(),
            "Sales".to_string(),
            "55000".to_string(),
        ],
    ];

    doc.add_worksheet("Employees", &employee_data)?;

    doc.save("example.xlsx")?;
    println!("XLSX document created successfully: example.xlsx");

    let loaded_doc = XlsxDocument::from_file("example.xlsx")?;

    println!("Sales worksheet data:");
    let sales = loaded_doc.get_worksheet_data("Sales")?;
    for (i, row) in sales.iter().enumerate() {
        println!("Row {}: {:?}", i + 1, row);
    }

    println!("\nEmployees worksheet data:");
    let employees = loaded_doc.get_worksheet_data("Employees")?;
    for (i, row) in employees.iter().enumerate() {
        println!("Row {}: {:?}", i + 1, row);
    }

    Ok(())
}
