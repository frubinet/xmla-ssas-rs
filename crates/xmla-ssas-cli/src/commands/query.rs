// SPDX-License-Identifier: MPL-2.0

use super::connect;
use crate::args::QueryArgs;
use std::error::Error;
use std::fs;
use std::io;
use std::io::Write;
use xmla_ssas_rs::xmla::XmlaDataset;

pub(crate) fn run(args: QueryArgs) -> Result<(), Box<dyn Error>> {
    let mdx = match (args.mdx, args.file) {
        (Some(mdx), None) => mdx,
        (None, Some(path)) => fs::read_to_string(path)?,
        _ => unreachable!("Exactly one query input is required"),
    };
    let mut connection = connect(args.endpoint, args.credentials)?;
    let dataset = connection.execute(mdx, args.catalog)?;

    write_dataset_csv(&dataset, io::stdout().lock())
}

fn write_dataset_csv<W: Write>(dataset: &XmlaDataset, writer: W) -> Result<(), Box<dyn Error>> {
    if dataset.axes().iter().any(|axis| {
        axis.name().starts_with("Axis") && axis.name() != "Axis0" && axis.name() != "Axis1"
    }) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "query results with more than two axes are not supported",
        )
        .into());
    }

    let column_axis = dataset
        .axes()
        .iter()
        .find(|axis| axis.name() == "Axis0")
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "query result does not contain Axis0",
            )
        })?;
    let row_axis = dataset.axes().iter().find(|axis| axis.name() == "Axis1");

    let mut csv = csv::WriterBuilder::new()
        .terminator(csv::Terminator::Any(b'\n'))
        .from_writer(writer);

    let mut header = Vec::new();
    if let Some(first_row) = row_axis.and_then(|axis| axis.tuples().first()) {
        header.extend(
            first_row
                .members()
                .iter()
                .map(|member| member.hierarchy().to_owned()),
        );
    }
    header.extend(column_axis.tuples().iter().map(|tuple| {
        tuple
            .members()
            .iter()
            .map(|member| member.unique_name())
            .collect::<Vec<_>>()
            .join(" / ")
    }));
    csv.write_record(header)?;

    let row_count = row_axis.map_or(1, |axis| axis.tuples().len());
    for row_index in 0..row_count {
        let mut record = Vec::new();
        if let Some(tuple) = row_axis.and_then(|axis| axis.tuples().get(row_index)) {
            record.extend(
                tuple
                    .members()
                    .iter()
                    .map(|member| member.caption().to_owned()),
            );
        }

        let row = u32::try_from(row_index)?;
        for column_index in 0..column_axis.tuples().len() {
            let column = u32::try_from(column_index)?;
            record.push(
                dataset
                    .cell_formatted_value_at(column, row)
                    .unwrap_or_default()
                    .to_owned(),
            );
        }
        csv.write_record(record)?;
    }
    csv.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use xmla_ssas_rs::xmla::parse_execute_response;

    #[test]
    fn writes_one_axis_dataset_as_csv() -> Result<(), Box<dyn Error>> {
        let dataset = parse_execute_response(include_str!(
            "../../../../tests/fixtures/execute/total_sales_amount/response.xml"
        ))?;
        let mut output = Vec::new();

        write_dataset_csv(&dataset, &mut output)?;

        assert_eq!(
            std::str::from_utf8(&output)?,
            include_str!("../../../../tests/fixtures/execute/total_sales_amount/expected.csv")
        );
        Ok(())
    }

    #[test]
    fn writes_two_axis_dataset_as_csv() -> Result<(), Box<dyn Error>> {
        let dataset = parse_execute_response(include_str!(
            "../../../../tests/fixtures/execute/countries/response.xml"
        ))?;
        let mut output = Vec::new();

        write_dataset_csv(&dataset, &mut output)?;

        assert_eq!(
            std::str::from_utf8(&output)?,
            include_str!("../../../../tests/fixtures/execute/countries/expected.csv")
        );
        Ok(())
    }
}
