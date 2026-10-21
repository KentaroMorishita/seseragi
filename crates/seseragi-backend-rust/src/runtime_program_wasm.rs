// Pure program ABI. No user value or continuation crosses a host import.
// Rust unwinding is NOT assumed for a WASM trap; the JS owner retires it.
#[derive(Clone)]
enum ProgramCode {
    Unit,
    Bool,
    Int,
    Text,
    Tuple(Vec<usize>),
    Maybe(usize),
    Either(usize, usize),
    Function(usize, usize),
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
        ProgramCode::Text => {
            let length = program_u32(input)? as usize;
            let value = std::str::from_utf8(program_take(input, length)?)
                .map_err(|_| "program ABI: invalid UTF-8")?;
            V::text(value)
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
        ProgramCode::Function(_, _) => return Err("program ABI: host callbacks are unsupported"),
    })
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
            let (tag, codec) = match value.get().0.as_str() {
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
#[no_mangle]
pub extern "C" fn ssrg_init() -> u32 {
    let fresh = PROGRAM.with(|s| !s.borrow().initialized && !s.borrow().closed);
    if !fresh {
        return program_result(Err("program ABI: instance already initialized or closed"));
    }
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
