// SPDX-License-Identifier: MPL-2.0

use csv::ReaderBuilder;
use std::collections::HashMap;
use std::{fs, path::Path};
use xmla_ssas_rs::xmla::{
    XmlaDataset, XmlaDiscoverResponse, parse_discover_response, parse_execute_response,
};

fn load_csv_matrix(csv: &str) -> Result<Vec<Vec<String>>, csv::Error> {
    ReaderBuilder::new()
        .has_headers(false)
        .from_reader(csv.as_bytes())
        .deserialize::<Vec<String>>()
        .collect()
}

fn compare_dataset_with_csv(dataset: &XmlaDataset, expected_csv: &str) -> Result<(), csv::Error> {
    let expected_matrix = load_csv_matrix(expected_csv)?;
    let expected_column_names = expected_matrix.first().expect("header row expected");

    let axis0 = dataset.axis(0).expect("axis0 expected");
    let axis1 = dataset.axis(1).expect("axis1 expected");
    let axis1_tuple = axis1.tuples().first().expect("axis1 tuple expected");
    let with_rows_axis = axis1.name() == "Axis1";
    let dimensions = if with_rows_axis {
        axis1_tuple
            .members()
            .iter()
            .map(|m| m.hierarchy())
            .collect::<Vec<&str>>()
    } else {
        Vec::new()
    };
    let measures = axis0
        .tuples()
        .iter()
        .map(|t| t.members().first().expect("member expected").unique_name())
        .collect::<Vec<&str>>();
    let dataset_column_names = dimensions
        .into_iter()
        .chain(measures)
        .map(|s| s.into())
        .collect::<Vec<String>>();
    assert_eq!(&dataset_column_names, expected_column_names);

    let expected_row_count = expected_matrix.len() - 1;
    assert_eq!(dataset.row_count(), expected_row_count);

    for row in 0..dataset.row_count() {
        let dataset_dimensions = if with_rows_axis {
            axis1.tuples()[row]
                .members()
                .iter()
                .map(|m| m.caption().to_string())
                .collect::<Vec<String>>()
        } else {
            Vec::new()
        };
        let dataset_row_values = (0..dataset.column_count())
            .map(|c| {
                dataset
                    .cell_formatted_value_at(c as u32, row as u32)
                    .unwrap_or("")
                    .to_string()
            })
            .collect::<Vec<String>>();
        let dataset_row = dataset_dimensions
            .into_iter()
            .chain(dataset_row_values)
            .collect::<Vec<String>>();
        assert_eq!(
            &dataset_row,
            expected_matrix.get(row + 1).expect("row expected")
        );
    }
    Ok(())
}

fn compare_discover_with_csv(
    response: &XmlaDiscoverResponse,
    expected_csv: &str,
) -> Result<(), csv::Error> {
    let expected_matrix = load_csv_matrix(expected_csv)?;
    let expected_attributes = expected_matrix.first().expect("header row expected");
    let rows: Vec<&HashMap<String, String>> = response.rows().collect();
    assert_eq!(rows.len(), expected_matrix.len() - 1);

    for column in 0..expected_attributes.len() {
        let attribute_name = expected_attributes.get(column).expect("column expected");
        for row in 0..rows.len() {
            let value = expected_matrix
                .get(row + 1)
                .expect("row expected")
                .get(column)
                .unwrap();
            let discover_row = rows.get(row).unwrap();
            assert_eq!(
                value,
                discover_row
                    .get(attribute_name)
                    .expect("attribute expected")
            );
        }
    }
    Ok(())
}

fn execute_case(response_path: &Path, response_xml: String) -> datatest_stable::Result<()> {
    let case_dir = response_path.parent().expect("parent directory expected");

    let expected_csv = fs::read_to_string(case_dir.join("expected.csv"))?;
    let dataset = parse_execute_response(&response_xml)?;
    compare_dataset_with_csv(&dataset, &expected_csv)?;

    Ok(())
}

fn discover_case(response_path: &Path, response_xml: String) -> datatest_stable::Result<()> {
    let case_dir = response_path.parent().expect("parent directory expected");

    let expected_csv = fs::read_to_string(case_dir.join("expected.csv"))?;
    let response = parse_discover_response(&response_xml)?;
    compare_discover_with_csv(&response, &expected_csv)?;

    Ok(())
}

datatest_stable::harness! {
    {
        test = execute_case,
        root = "tests/fixtures",
        pattern = r"^execute/.*/response\.xml$",
    },
    {
        test = discover_case,
        root = "tests/fixtures",
        pattern = r"^discover/.*/response\.xml$",
    },
}
