//! A bounded reader for the Protobuf wire format. It never allocates for the input it reads,
//! never indexes a slice, and skips every field it does not know.

use crate::consts::{
    FIXED32_BYTES, FIXED64_BYTES, GROUP_OPENED_DEPTH, MAX_GROUP_DEPTH, TAG_WIRE_BITS,
    TAG_WIRE_MASK, VARINT_CONTINUE, VARINT_MAX_BYTES, VARINT_PAYLOAD_BITS, VARINT_PAYLOAD_MASK,
    WIRE_END_GROUP, WIRE_FIXED32, WIRE_FIXED64, WIRE_LEN, WIRE_START_GROUP, WIRE_VARINT,
};
use crate::error::Corruption;

/// The value of one field that the decoder cares about.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Value<'a> {
    /// A varint.
    Varint(u64),
    /// Bytes with a length in front: a text, a nested message or packed numbers.
    Bytes(&'a [u8]),
}

/// Reads fields one after the other. Fixed-width fields and groups are skipped silently, so
/// only varints and values with a length in front reach the caller.
pub(crate) struct Fields<'a> {
    rest: &'a [u8],
}

impl<'a> Fields<'a> {
    /// Reads the fields of the message in `bytes`.
    pub(crate) const fn new(bytes: &'a [u8]) -> Self {
        Self { rest: bytes }
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], Corruption> {
        let (head, tail) = self
            .rest
            .split_at_checked(count)
            .ok_or(Corruption::Truncated)?;
        self.rest = tail;
        Ok(head)
    }

    fn varint(&mut self) -> Result<u64, Corruption> {
        let mut value = u64::MIN;
        for index in 0..VARINT_MAX_BYTES {
            let byte = *self.take(1)?.first().ok_or(Corruption::Truncated)?;
            let payload = u64::from(byte & VARINT_PAYLOAD_MASK);
            value |= payload
                .checked_shl(index * VARINT_PAYLOAD_BITS)
                .unwrap_or(0);
            if byte & VARINT_CONTINUE == 0 {
                return Ok(value);
            }
        }
        Err(Corruption::VarintTooLong)
    }

    fn length(&mut self) -> Result<&'a [u8], Corruption> {
        let size = usize::try_from(self.varint()?).map_err(|_| Corruption::Truncated)?;
        self.take(size)
    }

    /// Skips a group that was just opened, up to its matching end.
    fn skip_group(&mut self) -> Result<(), Corruption> {
        let mut depth = GROUP_OPENED_DEPTH;
        while depth > 0 {
            let tag = self.varint()?;
            match wire_type(tag) {
                WIRE_START_GROUP => {
                    depth += 1;
                    if depth > MAX_GROUP_DEPTH {
                        return Err(Corruption::TooDeep);
                    }
                }
                WIRE_END_GROUP => depth -= 1,
                other => self.skip_value(other)?,
            }
        }
        Ok(())
    }

    fn skip_value(&mut self, wire: u8) -> Result<(), Corruption> {
        match wire {
            WIRE_VARINT => self.varint().map(drop),
            WIRE_FIXED64 => self.take(FIXED64_BYTES).map(drop),
            WIRE_FIXED32 => self.take(FIXED32_BYTES).map(drop),
            WIRE_LEN => self.length().map(drop),
            WIRE_START_GROUP => self.skip_group(),
            _ => Err(Corruption::WireType),
        }
    }

    fn next_field(&mut self) -> Result<Option<(u32, Value<'a>)>, Corruption> {
        while !self.rest.is_empty() {
            let tag = self.varint()?;
            let field = u32::try_from(tag >> TAG_WIRE_BITS).map_err(|_| Corruption::WireType)?;
            match wire_type(tag) {
                WIRE_VARINT => return Ok(Some((field, Value::Varint(self.varint()?)))),
                WIRE_LEN => return Ok(Some((field, Value::Bytes(self.length()?)))),
                other => self.skip_value(other)?,
            }
        }
        Ok(None)
    }
}

fn wire_type(tag: u64) -> u8 {
    u8::try_from(tag & TAG_WIRE_MASK).unwrap_or(u8::MAX)
}

impl<'a> Iterator for Fields<'a> {
    type Item = Result<(u32, Value<'a>), Corruption>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.next_field().transpose();
        if item.as_ref().is_some_and(Result::is_err) {
            self.rest = &[];
        }
        item
    }
}

/// The varints of a packed repeated field.
pub(crate) fn packed_varints(bytes: &[u8]) -> impl Iterator<Item = Result<u64, Corruption>> + '_ {
    let mut reader = Fields::new(bytes);
    std::iter::from_fn(move || {
        if reader.rest.is_empty() {
            return None;
        }
        let item = reader.varint();
        if item.is_err() {
            reader.rest = &[];
        }
        Some(item)
    })
}
