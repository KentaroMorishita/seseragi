// Pure program ABI. No user value or continuation crosses a host import.
// Rust unwinding is NOT assumed for a WASM trap; the JS owner retires it.
#[derive(Clone)]
enum ProgramCode {
    Unit,
    Bool,
    Int,
    Text,
    Float,
    Char,
    BigInt,
    Decimal,
    Tuple(Vec<usize>),
    Maybe(usize),
    Either(usize, usize),
    Function(usize, usize),
    Array(usize),
    List(usize),
    Record(Vec<(&'static str, usize)>),
    Adt(Vec<(&'static str, Option<usize>)>),
    // A newtype that crosses as its payload; WASM keeps the owner wrapper.
    Newtype(&'static str, usize),
    Parameter,
}
struct ProgramRoot {
    value: V,
    codec: usize,
}
struct ProgramState {
    initialized: bool,
    closed: bool,
    next: u32,
    callbacks: BTreeMap<u32, ProgramRoot>,
    input: Vec<u8>,
    output: Vec<u8>,
}
impl Default for ProgramState {
    fn default() -> Self {
        Self {
            initialized: false,
            closed: false,
            next: 1,
            callbacks: BTreeMap::new(),
            input: Vec::new(),
            output: Vec::new(),
        }
    }
}
thread_local! {
    static PROGRAM: RefCell<ProgramState> = RefCell::new(ProgramState::default());
}
const PROGRAM_MAX_BYTES: usize = 16 * 1024 * 1024;
const PROGRAM_HOST_VALUE: &str = "program/host::Value";
type ProgramResult<T> = Result<T, &'static str>;

fn program_register(value: V, codec: usize) -> ProgramResult<u32> {
    if !matches!(value, V::Fun(_)) {
        return Err("program ABI: expected checked function result");
    }
    PROGRAM.with(|state| {
        let mut state = state.borrow_mut();
        if !state.initialized || state.closed {
            return Err("program ABI: closed instance");
        }
        let token = state.next;
        // Never recycle a token. A stale host callback cannot alias a new root.
        state.next = token
            .checked_add(1)
            .ok_or("program ABI: token space exhausted")?;
        state.callbacks.insert(token, ProgramRoot { value, codec });
        Ok(token)
    })
}
fn program_take<'a>(input: &mut &'a [u8], count: usize) -> ProgramResult<&'a [u8]> {
    let result = input.get(..count).ok_or("program ABI: truncated input")?;
    *input = &input[count..];
    Ok(result)
}
fn program_u32(input: &mut &[u8]) -> ProgramResult<u32> {
    Ok(u32::from_le_bytes(
        program_take(input, 4)?.try_into().unwrap(),
    ))
}
// The fewest bytes one value of `codec` occupies on the wire.
fn program_width(codec: usize, codes: &[ProgramCode]) -> usize {
    program_width_at(codec, codes, 0)
}
// A struct can contain itself without indirection, such as
// `struct Loop { next: Loop }`; no value of it exists, so stop at a depth no
// acyclic codec reaches and count the rest as zero.
fn program_width_at(codec: usize, codes: &[ProgramCode], depth: usize) -> usize {
    if depth > codes.len() {
        return 0;
    }
    match &codes[codec] {
        ProgramCode::Unit => 0,
        ProgramCode::Bool | ProgramCode::Maybe(_) | ProgramCode::Either(_, _) => 1,
        ProgramCode::Int | ProgramCode::Float | ProgramCode::Decimal => 8,
        ProgramCode::Tuple(elements) => elements
            .iter()
            .map(|c| program_width_at(*c, codes, depth + 1))
            .sum(),
        ProgramCode::Record(fields) => fields
            .iter()
            .map(|(_, c)| program_width_at(*c, codes, depth + 1))
            .sum(),
        ProgramCode::Newtype(_, value) => program_width_at(*value, codes, depth + 1),
        _ => 4,
    }
}
fn program_decode_items(
    input: &mut &[u8],
    element: usize,
    codes: &[ProgramCode],
) -> ProgramResult<Vec<V>> {
    let count = program_u32(input)? as usize;
    if count == 0 {
        return Ok(Vec::new());
    }
    // Each element needs its fixed width, so a longer count is invalid.
    let width = program_width(element, codes);
    if count > PROGRAM_MAX_ITEMS || (width > 0 && count > input.len() / width) {
        return Err("program ABI: invalid sequence length");
    }
    (0..count)
        .map(|_| program_decode(input, element, codes))
        .collect()
}
fn program_count(out: &mut Vec<u8>, count: usize) -> ProgramResult<()> {
    if count > PROGRAM_MAX_ITEMS {
        return Err("program ABI: sequence exceeds boundary limit");
    }
    let count = u32::try_from(count).map_err(|_| "program ABI: sequence exceeds boundary limit")?;
    program_bytes(out, &count.to_le_bytes())
}
fn program_decode(input: &mut &[u8], codec: usize, codes: &[ProgramCode]) -> ProgramResult<V> {
    Ok(match &codes[codec] {
        ProgramCode::Unit => V::Unit,
        ProgramCode::Bool => match program_take(input, 1)?[0] {
            0 => V::Bool(false),
            1 => V::Bool(true),
            _ => return Err("program ABI: invalid Bool"),
        },
        ProgramCode::Int => {
            let value = i64::from_le_bytes(program_take(input, 8)?.try_into().unwrap());
            if !(-9_007_199_254_740_991..=9_007_199_254_740_991).contains(&value) {
                return Err("program ABI: Int outside safe range");
            }
            V::Int(value)
        }
        ProgramCode::Float => V::Float(f64::from_le_bytes(
            program_take(input, 8)?.try_into().unwrap(),
        )),
        ProgramCode::Char => {
            V::Char(char::from_u32(program_u32(input)?).ok_or("program ABI: invalid Char")?)
        }
        ProgramCode::Text => {
            let length = program_u32(input)? as usize;
            let value = std::str::from_utf8(program_take(input, length)?)
                .map_err(|_| "program ABI: invalid UTF-8")?;
            V::text(value)
        }
        ProgramCode::BigInt => {
            let text = program_integer(input)?;
            V::BigInt(Shared::new(NativeBigInt::from_digits(
                text.starts_with('-'),
                text.trim_start_matches('-').as_bytes(),
                10,
            )))
        }
        ProgramCode::Decimal => {
            let coefficient = program_integer(input)?;
            let scale = program_integer(input)?;
            // Both targets keep one form per value: no trailing zero in the
            // coefficient, and zero only as 0 at scale 0.
            if (coefficient == "0" && scale != "0")
                || (coefficient != "0" && coefficient.ends_with('0'))
            {
                return Err("program ABI: non-canonical Decimal");
            }
            V::Decimal(Shared::new(NativeDecimal { coefficient, scale }))
        }
        ProgramCode::Tuple(elements) => V::Tuple(Shared::new(
            elements
                .iter()
                .map(|codec| program_decode(input, *codec, codes))
                .collect::<ProgramResult<Vec<_>>>()?,
        )),
        ProgramCode::Maybe(value) => match program_take(input, 1)?[0] {
            0 => V::adt("std/prelude::Nothing", V::Unit),
            1 => V::adt("std/prelude::Just", program_decode(input, *value, codes)?),
            _ => return Err("program ABI: invalid Maybe variant"),
        },
        ProgramCode::Either(left, right) => match program_take(input, 1)?[0] {
            0 => V::adt("std/prelude::Left", program_decode(input, *left, codes)?),
            1 => V::adt("std/prelude::Right", program_decode(input, *right, codes)?),
            _ => return Err("program ABI: invalid Either variant"),
        },
        ProgramCode::Array(element) => {
            V::Array(Shared::new(program_decode_items(input, *element, codes)?))
        }
        ProgramCode::List(element) => {
            V::list_from_values(program_decode_items(input, *element, codes)?)
        }
        ProgramCode::Record(fields) => {
            let mut values = BTreeMap::new();
            for (name, codec) in fields {
                values.insert((*name).to_owned(), program_decode(input, *codec, codes)?);
            }
            V::Record(Shared::new(values))
        }
        ProgramCode::Newtype(owner, value) => V::adt(owner, program_decode(input, *value, codes)?),
        // WASM only holds a type parameter's host value, by its token.
        ProgramCode::Parameter => {
            V::adt(PROGRAM_HOST_VALUE, V::Int(i64::from(program_u32(input)?)))
        }
        ProgramCode::Adt(variants) => {
            let (owner, payload) = variants
                .get(program_u32(input)? as usize)
                .ok_or("program ABI: invalid variant")?;
            let payload = match payload {
                Some(codec) => program_decode(input, *codec, codes)?,
                None => V::Unit,
            };
            V::adt(owner, payload)
        }
        ProgramCode::Function(_, _) => return Err("program ABI: host callbacks are unsupported"),
    })
}
// A canonical decimal integer: no sign on zero, no leading zero.
fn program_integer(input: &mut &[u8]) -> ProgramResult<String> {
    let length = program_u32(input)? as usize;
    let text = std::str::from_utf8(program_take(input, length)?)
        .map_err(|_| "program ABI: invalid UTF-8")?;
    let digits = text.strip_prefix('-').unwrap_or(text);
    if digits.is_empty()
        || !digits.bytes().all(|byte| byte.is_ascii_digit())
        || (digits.len() > 1 && digits.starts_with('0'))
        || text == "-0"
    {
        return Err("program ABI: invalid integer text");
    }
    Ok(text.to_owned())
}
fn program_text(out: &mut Vec<u8>, text: &str) -> ProgramResult<()> {
    let len = u32::try_from(text.len()).map_err(|_| "program ABI: text exceeds boundary limit")?;
    program_bytes(out, &len.to_le_bytes())?;
    program_bytes(out, text.as_bytes())
}
fn program_bytes(output: &mut Vec<u8>, bytes: &[u8]) -> ProgramResult<()> {
    if output
        .len()
        .checked_add(bytes.len())
        .is_none_or(|n| n > PROGRAM_MAX_BYTES)
    {
        return Err("program ABI: result exceeds boundary byte limit");
    }
    output.extend_from_slice(bytes);
    Ok(())
}
fn program_encode(
    value: V,
    codec: usize,
    codes: &[ProgramCode],
    out: &mut Vec<u8>,
) -> ProgramResult<()> {
    match (&codes[codec], value) {
        (ProgramCode::Unit, V::Unit) => Ok(()),
        (ProgramCode::Bool, V::Bool(value)) => program_bytes(out, &[u8::from(value)]),
        (ProgramCode::Int, V::Int(value))
            if (-9_007_199_254_740_991..=9_007_199_254_740_991).contains(&value) =>
        {
            program_bytes(out, &value.to_le_bytes())
        }
        (ProgramCode::Float, V::Float(value)) => program_bytes(out, &value.to_le_bytes()),
        (ProgramCode::Char, V::Char(value)) => program_bytes(out, &u32::from(value).to_le_bytes()),
        (ProgramCode::BigInt, V::BigInt(value)) => program_text(out, value.get().decimal()),
        (ProgramCode::Decimal, V::Decimal(value)) => {
            program_text(out, &value.get().coefficient)?;
            program_text(out, &value.get().scale)
        }
        (ProgramCode::Text, V::Text(value)) => {
            let bytes = value.get().as_bytes();
            let len = u32::try_from(bytes.len())
                .map_err(|_| "program ABI: text exceeds boundary limit")?;
            program_bytes(out, &len.to_le_bytes())?;
            program_bytes(out, bytes)
        }
        (ProgramCode::Tuple(elements), V::Tuple(values))
            if elements.len() == values.get().len() =>
        {
            for (value, codec) in values.get().iter().zip(elements) {
                program_encode(value.clone(), *codec, codes, out)?;
            }
            Ok(())
        }
        (ProgramCode::Maybe(_), V::Adt(value))
            if value.get().0 == "std/prelude::Nothing" && matches!(&value.get().1, V::Unit) =>
        {
            program_bytes(out, &[0])
        }
        (ProgramCode::Maybe(codec), V::Adt(value)) if value.get().0 == "std/prelude::Just" => {
            program_bytes(out, &[1])?;
            program_encode(value.get().1.clone(), *codec, codes, out)
        }
        (ProgramCode::Either(left, right), V::Adt(value)) => {
            let (tag, codec) = match value.get().0 {
                "std/prelude::Left" => (0, *left),
                "std/prelude::Right" => (1, *right),
                _ => return Err("program ABI: unexpected Either owner"),
            };
            program_bytes(out, &[tag])?;
            program_encode(value.get().1.clone(), codec, codes, out)
        }
        (ProgramCode::Function(_, _), value @ V::Fun(_)) => {
            let token = program_register(value, codec)?;
            program_bytes(out, &token.to_le_bytes())
        }
        (ProgramCode::Array(element), V::Array(values)) => {
            program_count(out, values.get().len())?;
            for value in values.get().iter() {
                program_encode(value.clone(), *element, codes, out)?;
            }
            Ok(())
        }
        (ProgramCode::List(element), list @ (V::ListEmpty | V::List(_))) => {
            let mut values = Vec::new();
            let mut cursor = &list;
            while let Some(cell) = cursor.list_cell() {
                values.push(cell.head.clone());
                cursor = &cell.tail;
            }
            program_count(out, values.len())?;
            for value in values {
                program_encode(value, *element, codes, out)?;
            }
            Ok(())
        }
        (ProgramCode::Record(fields), V::Record(values)) if values.get().len() == fields.len() => {
            for (name, codec) in fields {
                let value = values
                    .get()
                    .get(*name)
                    .ok_or("program ABI: result violates checked public type")?
                    .clone();
                program_encode(value, *codec, codes, out)?;
            }
            Ok(())
        }
        (ProgramCode::Parameter, V::Adt(value)) if value.get().0 == PROGRAM_HOST_VALUE => {
            match value.get().1 {
                V::Int(token) => program_bytes(
                    out,
                    &u32::try_from(token)
                        .map_err(|_| "program ABI: invalid host value")?
                        .to_le_bytes(),
                ),
                _ => Err("program ABI: invalid host value"),
            }
        }
        (ProgramCode::Newtype(owner, codec), V::Adt(value)) if value.get().0 == *owner => {
            program_encode(value.get().1.clone(), *codec, codes, out)
        }
        (ProgramCode::Adt(variants), V::Adt(value)) => {
            let index = variants
                .iter()
                .position(|(owner, _)| *owner == value.get().0)
                .ok_or("program ABI: unexpected variant owner")?;
            program_bytes(out, &(index as u32).to_le_bytes())?;
            match (variants[index].1, &value.get().1) {
                (Some(codec), payload) => program_encode(payload.clone(), codec, codes, out),
                (None, V::Unit) => Ok(()),
                (None, _) => Err("program ABI: result violates checked public type"),
            }
        }
        _ => Err("program ABI: result violates checked public type"),
    }
}
fn program_result(result: ProgramResult<Vec<u8>>) -> u32 {
    let (status, bytes) = match result {
        Ok(bytes) => (0, bytes),
        Err(message) => (1, message.as_bytes().to_vec()),
    };
    PROGRAM.with(|state| state.borrow_mut().output = bytes);
    status
}
thread_local! {
    static PROGRAM_PANIC: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}
// A Defect panics, and a panic aborts in WASM. Record the Defect and its
// source location first, so the loader can report them after the trap. The
// record is the location (`file:line:column`, or empty), a newline and the
// message.
fn program_record_panics() {
    std::panic::set_hook(Box::new(|info| {
        let payload = info.payload();
        let (source, message) =
            if let Some(diagnostic) = payload.downcast_ref::<NativeDiagnostic<String>>() {
                (
                    diagnostic.source.map(ToString::to_string),
                    diagnostic.value.clone(),
                )
            } else if let Some(message) = payload.downcast_ref::<&str>() {
                (None, (*message).to_owned())
            } else if let Some(message) = payload.downcast_ref::<String>() {
                (None, message.clone())
            } else {
                (None, "Seseragi Defect".to_owned())
            };
        let record = format!("{}\n{message}", source.unwrap_or_default());
        let _ = PROGRAM_PANIC.try_with(|panic| {
            if let Ok(mut panic) = panic.try_borrow_mut() {
                *panic = record.into_bytes();
            }
        });
    }));
}
#[no_mangle]
pub extern "C" fn ssrg_panic_ptr() -> usize {
    PROGRAM_PANIC.with(|panic| panic.borrow().as_ptr() as usize)
}
#[no_mangle]
pub extern "C" fn ssrg_panic_len() -> usize {
    PROGRAM_PANIC.with(|panic| panic.borrow().len())
}
#[no_mangle]
pub extern "C" fn ssrg_init() -> u32 {
    let fresh = PROGRAM.with(|s| !s.borrow().initialized && !s.borrow().closed);
    if !fresh {
        return program_result(Err("program ABI: instance already initialized or closed"));
    }
    program_record_panics();
    initialize_globals();
    PROGRAM.with(|state| state.borrow_mut().initialized = true);
    0
}
#[no_mangle]
pub extern "C" fn ssrg_input(length: u32) -> usize {
    PROGRAM.with(|state| {
        let mut state = state.borrow_mut();
        if state.closed || !state.initialized || length as usize > PROGRAM_MAX_BYTES {
            return 0;
        }
        state.input.resize(length as usize, 0);
        state.input.as_ptr() as usize
    })
}
#[no_mangle]
pub extern "C" fn ssrg_export(index: u32) -> u32 {
    let active = PROGRAM.with(|s| s.borrow().initialized && !s.borrow().closed);
    if !active {
        return program_result(Err("program ABI: closed instance"));
    }
    let result = program_export(index)
        .ok_or("program ABI: unknown export")
        .and_then(|(value, codec)| program_register(value, codec))
        .map(|token| token.to_le_bytes().to_vec());
    program_result(result)
}
#[no_mangle]
pub extern "C" fn ssrg_call(token: u32) -> u32 {
    let call = PROGRAM.with(|state| {
        let state = state.borrow();
        if state.closed {
            return Err("program ABI: closed instance");
        }
        let root = state
            .callbacks
            .get(&token)
            .ok_or("program ABI: detached callback")?;
        Ok((root.value.clone(), root.codec, state.input.clone()))
    });
    let result = call.and_then(|(function, codec, input)| {
        let codes = program_codecs();
        let ProgramCode::Function(parameter, result) = codes[codec] else {
            return Err("program ABI: root is not callable");
        };
        let mut bytes = input.as_slice();
        let argument = program_decode(&mut bytes, parameter, &codes)?;
        if !bytes.is_empty() {
            return Err("program ABI: trailing input bytes");
        }
        // No RefCell borrow or host callback survives this source stage.
        let value = function.apply(argument);
        let mut output = Vec::new();
        program_encode(value, result, &codes, &mut output)?;
        Ok(output)
    });
    program_result(result)
}
#[no_mangle]
pub extern "C" fn ssrg_detach(token: u32) {
    let value = PROGRAM.with(|state| state.borrow_mut().callbacks.remove(&token));
    drop(value);
}
#[no_mangle]
pub extern "C" fn ssrg_output_ptr() -> usize {
    PROGRAM.with(|state| state.borrow().output.as_ptr() as usize)
}
#[no_mangle]
pub extern "C" fn ssrg_output_len() -> usize {
    PROGRAM.with(|state| state.borrow().output.len())
}
#[no_mangle]
pub extern "C" fn ssrg_callbacks() -> usize {
    PROGRAM.with(|state| state.borrow().callbacks.len())
}
#[no_mangle]
pub extern "C" fn ssrg_live() -> usize {
    LIVE.with(Cell::get)
}
#[no_mangle]
pub extern "C" fn ssrg_dispose() -> usize {
    let values = PROGRAM.with(|state| {
        let mut state = state.borrow_mut();
        state.closed = true;
        state.input = Vec::new();
        state.output = Vec::new();
        std::mem::take(&mut state.callbacks)
    });
    drop(values);
    clear_globals();
    clear_dictionaries();
    LIVE.with(Cell::get)
}
