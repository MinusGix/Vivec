// TODO: this is not a full impl of all ctda related fields
// missing: CITC, CIS1, CIS2, and collections that automatically group them together

use super::common::{write_field_header, FromField, FromFieldError, GeneralField, FIELDH_SIZE};
use crate::{
    impl_static_data_size, impl_static_type_named,
    parse::{single, take, PResult, Parse, ParseError},
    records::common::{ConversionError, FormId},
    util::Writable,
};
use std::io::Write;

pub type FunctionIndex = u16;

// TODO: it might be interesting to have RunOn hold the reference if it's of the Reference variant

/// [reference].[function]([param_1], [param_2]) [operator] [value
#[derive(Debug, Clone)]
pub struct CTDA {
    /// operator and flags
    pub op_data: OperatorData,
    /// ?
    pub unknown: [u8; 3],
    pub comp_value: ComparisonValue,
    /// Index into the function list
    pub function_index: FunctionIndex,
    pub padding: u16,
    pub parameters: Parameters,
    pub run_on: RunOn,
    /// Function reference. Zero if no reference is needed (run_on != Reference)
    pub reference: FormId,
    /// uesp: always -1
    pub unknown2: i32,
}
impl FromField<'_> for CTDA {
    fn from_field(field: GeneralField<'_>) -> PResult<Self, FromFieldError> {
        let (data, op_data) = OperatorData::parse(field.data)?;
        let (data, unknown) = take(data, 3)?;
        let unknown = [unknown[0], unknown[1], unknown[2]];
        let (data, comp_value) = ComparisonValue::parse(data, op_data.flags)?;
        let (data, function_index) = u16::parse(data)?;
        let (data, padding) = u16::parse(data)?;
        let (data, parameters) = Parameters::parse(data, function_index)?;
        let (data, run_on) = RunOn::parse(data)?;
        let (data, reference) = FormId::parse(data)?;
        let (data, unknown2) = i32::parse(data)?;
        Ok((
            data,
            Self {
                op_data,
                unknown,
                comp_value,
                function_index,
                padding,
                parameters,
                run_on,
                reference,
                unknown2,
            },
        ))
    }
}
impl_static_type_named!(CTDA, b"CTDA");
impl_static_data_size!(
    CTDA,
    FIELDH_SIZE +
	OperatorData::static_data_size() +
	(u8::static_data_size() * 3) + // unknown
	ComparisonValue::static_data_size() +
	u16::static_data_size() + // function index
    u16::static_data_size() + // padding
    Parameters::static_data_size() +
	RunOn::static_data_size() +
	FormId::static_data_size() +
	i32::static_data_size()
);
impl Writable for CTDA {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        write_field_header(self, w)?;
        self.op_data.write_to(w)?;
        self.unknown[0].write_to(w)?;
        self.unknown[1].write_to(w)?;
        self.unknown[2].write_to(w)?;
        self.comp_value.write_to(w)?;
        self.function_index.write_to(w)?;
        self.padding.write_to(w)?;
        self.parameters.write_to(w)?;
        self.run_on.write_to(w)?;
        self.reference.write_to(w)?;
        self.unknown2.write_to(w)
    }
}

// Repr 3 bits, upper
// The actual ''operator'' is a full byte, but the lower 5 bits are used for flags
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Operator {
    /// 0, ==
    Equal,
    /// 1, !=
    NotEqual,
    /// 2, >
    GreaterThan,
    /// 3, >=
    GreaterThanEqual,
    /// 4, <
    LessThan,
    /// 5, <=
    LessThanEqual,
}
type OperatorError = ConversionError<u8>;
impl Operator {
    /// This takes in the *original* bits, so the upper 3 bits are the operator.
    pub fn from_byte(v: u8) -> Result<Self, OperatorError> {
        // Only the upper 3 bits matter
        let v = (v & 0b11100000) >> 5;
        Operator::from_value(v)
    }

    /// This takes in the value after it's been turned into a 3bit integer
    pub fn from_value(v: u8) -> Result<Self, OperatorError> {
        Ok(match v {
            0 => Operator::Equal,
            1 => Operator::NotEqual,
            2 => Operator::GreaterThan,
            3 => Operator::GreaterThanEqual,
            4 => Operator::LessThan,
            5 => Operator::LessThanEqual,
            x => return Err(OperatorError::InvalidEnumerationValue(x)),
        })
    }

    pub fn code(&self) -> u8 {
        match self {
            Operator::Equal => 0,
            Operator::NotEqual => 1,
            Operator::GreaterThan => 2,
            Operator::GreaterThanEqual => 3,
            Operator::LessThan => 4,
            Operator::LessThanEqual => 5,
        }
    }

    /// Returns u8 with bits set in correct position
    pub fn bits(&self) -> u8 {
        self.code() << 5
    }
}
/// repr lower 5 bits
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Flags {
    /// 0b00001: OR (default is to AND conditions together)
    /// 0b00010: Parameters use aliases. Force function parameters to use quest alias data. Exclusive with 'use pack data'
    /// 0b00100: Use global
    /// 0b01000: Use pack data. Force func parameters to use pack data. Exclusive with 'use aliases'
    /// 0b10000: Swap subject and target? (uesp isn't sure on this one)
    pub flags: u8,
}
impl Flags {
    pub fn from_byte(v: u8) -> Self {
        Flags {
            flags: v & 0b00011111,
        }
    }

    pub fn or(&self) -> bool {
        (self.flags & 0b1) != 0
    }

    pub fn use_aliases(&self) -> bool {
        (self.flags & 0b10) != 0
    }

    pub fn use_global(&self) -> bool {
        (self.flags & 0b100) != 0
    }

    pub fn use_pack_data(&self) -> bool {
        (self.flags & 0b1000) != 0
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct OperatorData {
    pub operator: Operator,
    pub flags: Flags,
}
impl OperatorData {
    pub fn from_byte(v: u8) -> Result<Self, OperatorError> {
        Ok(Self {
            operator: Operator::from_byte(v)?,
            flags: Flags::from_byte(v),
        })
    }
    pub fn parse(data: &[u8]) -> PResult<Self> {
        let (data, v) = single(data)?;
        OperatorData::from_byte(v)
            .map_err(|x| match x {
                OperatorError::InvalidEnumerationValue(_) => ParseError::InvalidEnumerationValue,
            })
            .map(|x| (data, x))
    }
}
impl_static_data_size!(OperatorData, u8::static_data_size());
impl Writable for OperatorData {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        (self.flags.flags & self.operator.bits()).write_to(w)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ComparisonValue {
    /// Value to compare against
    Float(f32),
    /// GLOB formid
    Glob(FormId),
}
impl ComparisonValue {
    fn parse(data: &[u8], flags: Flags) -> PResult<Self> {
        Ok(if flags.use_global() {
            let (data, formid) = FormId::parse(data)?;
            (data, ComparisonValue::Glob(formid))
        } else {
            let (data, float) = f32::parse(data)?;
            (data, ComparisonValue::Float(float))
        })
    }
}
impl_static_data_size!(
    ComparisonValue,
    FormId::static_data_size().max(f32::static_data_size())
);
impl Writable for ComparisonValue {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        match self {
            ComparisonValue::Float(x) => x.write_to(w),
            ComparisonValue::Glob(x) => x.write_to(w),
        }
    }
}

// TODO: the function parameters are more complex than this..
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Parameters {
    Normal { first: u32, second: u32 },
}
impl Parameters {
    // we ignore the function index for now, just parsing it always as two u32s
    pub fn parse(data: &[u8], _findex: FunctionIndex) -> PResult<Self> {
        let (data, first) = u32::parse(data)?;
        let (data, second) = u32::parse(data)?;
        Ok((data, Parameters::Normal { first, second }))
    }
}
impl Writable for Parameters {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        match self {
            Parameters::Normal { first, second } => {
                first.write_to(w)?;
                second.write_to(w)
            }
        }
    }
}
impl_static_data_size!(Parameters, u64::static_data_size());

/// The method of applying the condition
/// repr: u32
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RunOn {
    /// 0
    Subject,
    /// 1
    Target,
    /// 2, related to reference field
    Reference,
    /// 3
    CombatTarget,
    /// 4, use a reference linked to another reference
    LinkedReferenced,
    /// 5, use quest alias data
    QuestAlias,
    /// 6
    PackageData,
    /// 7, use radiant event data
    EventData,
}
type RunOnError = ConversionError<u32>;
impl RunOn {
    pub fn from_u32(v: u32) -> Result<Self, RunOnError> {
        Ok(match v {
            0 => RunOn::Subject,
            1 => RunOn::Target,
            2 => RunOn::Reference,
            3 => RunOn::CombatTarget,
            4 => RunOn::LinkedReferenced,
            5 => RunOn::QuestAlias,
            6 => RunOn::PackageData,
            7 => RunOn::EventData,
            x => return Err(RunOnError::InvalidEnumerationValue(x)),
        })
    }

    pub fn parse(data: &[u8]) -> PResult<Self> {
        let (data, v) = u32::parse(data)?;
        Self::from_u32(v).map(|x| (data, x)).map_err(|e| match e {
            RunOnError::InvalidEnumerationValue(_) => ParseError::InvalidEnumerationValue,
        })
    }

    pub fn code(&self) -> u32 {
        match self {
            RunOn::Subject => 0,
            RunOn::Target => 1,
            RunOn::Reference => 2,
            RunOn::CombatTarget => 3,
            RunOn::LinkedReferenced => 4,
            RunOn::QuestAlias => 5,
            RunOn::PackageData => 6,
            RunOn::EventData => 7,
        }
    }
}
impl_static_data_size!(RunOn, u32::static_data_size());
impl Writable for RunOn {
    fn write_to<T>(&self, w: &mut T) -> std::io::Result<()>
    where
        T: Write,
    {
        self.code().write_to(w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_size_output;
    #[test]
    fn test_ctda() {
        let ctda = CTDA {
            op_data: OperatorData {
                operator: Operator::Equal,
                flags: Flags::from_byte(0),
            },
            unknown: [4, 5, 6],
            comp_value: ComparisonValue::Float(4.3),
            function_index: 0,
            padding: 0,
            parameters: Parameters::Normal {
                first: 0x0,
                second: 0x1,
            },
            run_on: RunOn::Target,
            reference: FormId::new(0),
            unknown2: -1,
        };
        assert_size_output!(ctda);
    }
}
