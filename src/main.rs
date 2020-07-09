use bstr::{BStr, BString, ByteSlice, ByteVec, B};
use derive_more::From;
use groups::{
    common::{FromGeneralGroup, FromTopGroup, FromTopGroupError, GeneralGroup, GroupType},
    Group,
};
use parse::{many, take, PResult, ParseError};
use records::common::{FromRecord, FromRecordError, GeneralRecord, TypeNamed};
use util::{DataSize, StaticDataSize, Writable};

mod groups;
mod parse;
mod records;
mod util;

#[derive(Debug, Clone)]
pub enum GeneralTop<'data> {
    Record(GeneralRecord<'data>),
    Group(GeneralGroup<'data>),
}

#[derive(Debug, Clone, From)]
enum GeneralError<'data> {
    TopGroup(FromTopGroupError<'data>),
    Record(FromRecordError<'data>),
    ParseError(ParseError<'data>),
}

fn parse_top_level<'data>(data: &'data [u8]) -> PResult<GeneralTop<'data>, GeneralError<'data>> {
    let (_, name) = take(data, 4)?;
    // GRUPs have different format than records, and parsing them as records would be dreadfully incorrect.
    if name == b"GRUP" {
        let (data, group) = GeneralGroup::parse(data)?;
        Ok((data, GeneralTop::Group(group)))
    } else {
        let (data, record) = GeneralRecord::parse(data)?;
        Ok((data, GeneralTop::Record(record)))
    }
}

#[derive(Debug, Clone)]
pub enum Top<'data> {
    Record(records::Record<'data>),
    // TODO: custom group types?
    Group(groups::Group<'data>),
}
impl<'data> Writable for Top<'data> {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: std::io::Write,
    {
        match self {
            Top::Record(record) => record.write_to(w),
            Top::Group(group) => group.write_to(w),
        }
    }
}
impl<'data> DataSize for Top<'data> {
    fn data_size(&self) -> usize {
        match self {
            Top::Record(record) => record.data_size(),
            Top::Group(group) => group.data_size(),
        }
    }
}

fn parse_file(data: &[u8]) -> PResult<Vec<Top>, GeneralError> {
    let (data, general_top) = many(data, parse_top_level)?;

    println!(
        "After parsing general top level, there was {} bytes left",
        data.len()
    );

    let mut spec_top = Vec::new();

    // Note: we parse record fields as if the order doesn't matter, but It probably does, but we can't be sure it does :(
    for top in general_top {
        match top {
            GeneralTop::Record(record) => {
                if record
                    .common_info
                    .flags
                    .is(records::common::record_flag::COMPRESSED)
                {
                    println!("{} is compressed", record.type_name);
                }

                spec_top.push(Top::Record(match record.type_name.as_ref() {
                    b"TES4" => records::tes4::TES4Record::from_record(record)?.1.into(),
                    b"AACT" => records::aact::AACTRecord::from_record(record)?.1.into(),
                    b"ADDN" => records::addn::ADDNRecord::from_record(record)?.1.into(),
                    b"ACHR" => records::achr::ACHRRecord::from_record(record)?.1.into(),
                    b"ACTI" => records::acti::ACTIRecord::from_record(record)?.1.into(),
                    _ => record.into(),
                }));
            }
            GeneralTop::Group(group) => spec_top.push(Top::Group(match group.group_type {
                GroupType::Top(_) => {
                    let group = groups::common::TopGroup::from_general_group(group);
                    match group.label.as_ref() {
                        b"AACT" => groups::aact::AACTGroup::from_top_group(group)?.1.into(),
                        b"ACTI" => groups::acti::ACTIGroup::from_top_group(group)?.1.into(),
                        _ => group.into(),
                    }
                }
                _ => group.into(),
            })),
        }
    }

    Ok((data, spec_top))
}

fn main() {
    println!("Starting");
    let data = std::fs::read("./ex/Dawnguard.esm").expect("Failed to read data from file");
    let (_data, result) = parse_file(data.as_slice()).expect("Failed to parse");
    {
        use records::Record;
        for entry in result.iter() {
            match entry {
                Top::Record(record) => match record {
                    Record::TES4(tes4) => println!("TES4: {:?}", tes4),
                    Record::AACT(aact) => println!("AACT: {:?}", aact),
                    Record::ADDN(addn) => println!("ADDN: {:?}", addn),
                    Record::ACHR(achr) => println!("ACHR: {:?}", achr),
                    Record::ACTI(acti) => println!("ACTI: {:?}", acti),
                    Record::Unknown(record) => println!("U({}),", record.type_name()), // println!("Unknown record: {:?}", record),
                },
                Top::Group(group) => match group {
                    Group::AACT(group) => println!("AACT Group: {:#?} entries", group),
                    Group::ACTI(group) => println!("ACTI Group: {} entries", group.records.len()),
                    _ => print!("G, "),
                },
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equivalent_writeback() {
        let data = std::fs::read("./ex/Dawnguard.esm").expect("Failed to read data from file");
        let (_data, result) = parse_file(data.as_slice()).expect("Failed to parse");
        let mut stored = Vec::new();
        stored.reserve(data.len());
        println!("Writing top level data");
        let mut start_i: usize = 0;
        for top in result {
            top.write_to(&mut stored).unwrap();
            let mut i = start_i;
            let end = stored.len();
            loop {
                if i == end {
                    break;
                }

                if data[i] != stored[i] {
                    panic!(
                        "(i={}) Data[i] = ({:x?}) did not equal Stored[i] = ({:x?}), while dealing with: {:?}",
                        i, data[i], stored[i], top
                    );
                }

                i += 1;
            }
            start_i = end;
        }
        println!("Wrote data.");
        println!("Original data size: {}", data.len());
        println!("New      data size: {}", stored.len());
        // Perform a full equality check
        if data.len() == stored.len() {
            println!("Equivalent sizes.");
            for i in 0..data.len() {
                if data[i] != stored[i] {
                    panic!(
                        "Data[{}] ({:x?}) did not equal Stored[{}] ({:x?})",
                        i, data[i], i, stored[i]
                    );
                }
            }
            println!("The two files are equal!");
        } else {
            println!("They did not have equivalent sizes :(");
        }
    }
}
