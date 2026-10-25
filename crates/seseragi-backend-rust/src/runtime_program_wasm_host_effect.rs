// Typed host Effect suspension over the existing resumable NativeMachine.
// Separate transport keeps pure-v1/providerless output intact. Admission
// excludes language panics; unexpected engine traps retire the instance
// without claiming Rust unwinding or native resource cleanup.
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
    Effect(Option<usize>, Option<usize>),
    HostEffect(usize, Option<usize>, Option<usize>),
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
    if !matches!(value, V::Fun(_) | V::Effect(_)) {
        return Err("program ABI: expected checked callable or Effect result");
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
        ProgramCode::HostEffect(_, _, _) => program_host_decode(program_u32(input)?, codec)?,
        ProgramCode::Function(_, _) | ProgramCode::Effect(_, _) => {
            return Err("program ABI: host callbacks and Effect inputs are unsupported")
        }
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
        (ProgramCode::Effect(_, _), value @ V::Effect(_)) => {
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
    if PROGRAM_TRACK_REFS {
        // Cell captures must be tracked before globals and decoded host leases
        // can allocate values. Admission limits cells to the closed data codec.
        enable_cycle_collection();
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
    if PROGRAM_RUN.with(|run| run.borrow().is_some()) {
        // A raw caller must finish cancellation/cleanup before dropping roots.
        return usize::MAX;
    }
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
    PROGRAM_HOSTS.with(|hosts| hosts.borrow_mut().clear());
    LIVE.with(Cell::get)
}

struct ProgramRun {
    machine: NativeMachine,
    failure: Option<usize>,
    success: Option<usize>,
    waiting: Option<ProgramHostRequest>,
}
thread_local! {
    static PROGRAM_RUN: RefCell<Option<ProgramRun>> = const { RefCell::new(None) };
}

#[no_mangle]
pub extern "C" fn ssrg_start(token: u32) -> u32 {
    let result = (|| {
        if PROGRAM_RUN.with(|run| run.borrow().is_some()) {
            return Err("program ABI: another root Effect is running");
        }
        let (plan, codec) = PROGRAM.with(|state| {
            let state = state.borrow();
            if !state.initialized || state.closed {
                return Err("program ABI: closed instance");
            }
            let root = state
                .callbacks
                .get(&token)
                .ok_or("program ABI: detached Effect")?;
            Ok((root.value.clone(), root.codec))
        })?;
        let codes = program_codecs();
        let ProgramCode::Effect(failure, success) = codes[codec] else {
            return Err("program ABI: root is not an Effect");
        };
        if !matches!(plan, V::Effect(_)) {
            return Err("program ABI: invalid checked Effect plan");
        }
        let run = ProgramRun {
            machine: NativeMachine::new(NativeContext::default(), plan),
            failure,
            success,
            waiting: None,
        };
        PROGRAM_RUN.with(|slot| *slot.borrow_mut() = Some(run));
        Ok(Vec::new())
    })();
    program_result(result)
}

#[no_mangle]
pub extern "C" fn ssrg_cancel() -> u32 {
    PROGRAM_RUN.with(|run| {
        if let Some(run) = run.borrow_mut().as_mut() {
            // Keep all continuation frames, captured values and registered
            // finalizers. The machine observes cancellation at a checkpoint.
            run.machine.context.cancelled = true;
            0
        } else {
            1
        }
    })
}

fn program_outcome(
    outcome: NativeOutcome,
    failure: Option<usize>,
    success: Option<usize>,
) -> ProgramResult<Vec<u8>> {
    let codes = program_codecs();
    let mut out = Vec::new();
    match outcome {
        Ok(value) => {
            program_bytes(&mut out, &[1])?;
            program_encode(
                value,
                success.ok_or("program ABI: Never success produced a value")?,
                &codes,
                &mut out,
            )?;
        }
        Err(NativeExit::Failure(error)) => {
            program_bytes(&mut out, &[2])?;
            program_encode(
                error,
                failure.ok_or("program ABI: Never failure produced a value")?,
                &codes,
                &mut out,
            )?;
        }
        Err(NativeExit::Defect(messages)) => {
            program_bytes(&mut out, &[3])?;
            let count = u32::try_from(messages.len())
                .map_err(|_| "program ABI: too many Defect messages")?;
            program_bytes(&mut out, &count.to_le_bytes())?;
            for message in messages {
                let bytes = message.as_bytes();
                let count = u32::try_from(bytes.len())
                    .map_err(|_| "program ABI: Defect message exceeds boundary limit")?;
                program_bytes(&mut out, &count.to_le_bytes())?;
                program_bytes(&mut out, bytes)?;
            }
        }
        Err(NativeExit::Cancelled) => program_bytes(&mut out, &[4])?,
    }
    Ok(out)
}

#[no_mangle]
pub extern "C" fn ssrg_poll(quantum: u32) -> u32 {
    let result = (|| {
        if !(1..=1024).contains(&quantum) {
            return Err("program ABI: invalid Effect quantum");
        }
        // No TLS borrow crosses a source callback or a result-root registration.
        let mut run = PROGRAM_RUN
            .with(|slot| slot.borrow_mut().take())
            .ok_or("program ABI: no running Effect")?;
        if let Some(request) = &run.waiting {
            let result = program_host_request(request);
            PROGRAM_RUN.with(|slot| *slot.borrow_mut() = Some(run));
            return result;
        }
        for _ in 0..quantum {
            match run.machine.turn() {
                NativeTurn::Runnable => {
                    if let Some(mut request) =
                        PROGRAM_HOST_PENDING.with(|pending| pending.borrow_mut().take())
                    {
                        // Native leaves yield at their checkpoint BEFORE the
                        // placeholder can reach Bind/Register/Scope frames.
                        request.masked = run.machine.context.masked
                            || run
                                .machine
                                .stack
                                .iter()
                                .any(|frame| matches!(frame, NativeFrame::Register(_)));
                        match program_host_request(&request) {
                            Ok(bytes) => {
                                run.waiting = Some(request);
                                PROGRAM_RUN.with(|slot| *slot.borrow_mut() = Some(run));
                                return Ok(bytes);
                            }
                            Err(message) => run
                                .machine
                                .resume(Err(NativeExit::Defect(vec![message.into()]))),
                        }
                    }
                }
                NativeTurn::CloseScope(original, previous_mask) => {
                    run.machine.state = Some(run.machine.context.cleanup(
                        &mut run.machine.stack,
                        original,
                        Vec::new(),
                        previous_mask,
                    ));
                }
                NativeTurn::Complete(outcome) => {
                    return program_outcome(outcome, run.failure, run.success)
                }
                NativeTurn::Operation(_, _) | NativeTurn::FileSystem(_, _) => {
                    // Full target admission prevents these turns. Preserve the
                    // scope stack even for a violated internal contract.
                    run.machine.resume(Err(NativeExit::Defect(vec![
                        "program ABI: unadmitted host operation".into(),
                    ])));
                }
            }
        }
        PROGRAM_RUN.with(|slot| *slot.borrow_mut() = Some(run));
        Ok(vec![0])
    })();
    program_result(result)
}

#[no_mangle]
pub extern "C" fn ssrg_runs() -> u32 {
    PROGRAM_RUN.with(|run| u32::from(run.borrow().is_some()))
}

// Each source callback and each cold invocation shares this managed marker.
// A weak lease never keeps a host reference alive by itself.
type ProgramHostLease = Weak<Node<BTreeMap<String, V>>>;
struct ProgramHostRequest {
    token: u32,
    codec: usize,
    argument: V,
    _lease: V,
    masked: bool,
}
thread_local! {
    static PROGRAM_HOSTS: RefCell<BTreeMap<u32, ProgramHostLease>> = const { RefCell::new(BTreeMap::new()) };
    static PROGRAM_HOST_PENDING: RefCell<Option<ProgramHostRequest>> = const { RefCell::new(None) };
}
fn program_host_decode(token: u32, codec: usize) -> ProgramResult<V> {
    if token == 0 || PROGRAM_HOSTS.with(|hosts| hosts.borrow().contains_key(&token)) {
        return Err("program ABI: invalid/reused host callback token");
    }
    let marker = Shared::new(BTreeMap::from([
        ("token".into(), V::Int(i64::from(token))),
        ("codec".into(), V::Int(codec as i64)),
    ]));
    let weak = Rc::downgrade(
        marker
            .0
            .as_ref()
            .ok_or("program ABI: missing callback lease")?,
    );
    PROGRAM_HOSTS.with(|hosts| hosts.borrow_mut().insert(token, weak));
    Ok(V::fun(vec![V::Record(marker)], |captures, argument| {
        V::leaf(vec![captures[0].clone(), argument], |captures, _| {
            let V::Int(token) = captures[0].field("token") else {
                return Err(NativeExit::Defect(vec![
                    "program ABI: invalid host token lease".into(),
                ]));
            };
            let V::Int(codec) = captures[0].field("codec") else {
                return Err(NativeExit::Defect(vec![
                    "program ABI: invalid host codec lease".into(),
                ]));
            };
            PROGRAM_HOST_PENDING.with(|pending| {
                *pending.borrow_mut() = Some(ProgramHostRequest {
                    token: token as u32,
                    codec: codec as usize,
                    argument: captures[1].clone(),
                    _lease: captures[0].clone(),
                    masked: false,
                })
            });
            Ok(V::Unit)
        })
    }))
}
fn program_host_request(request: &ProgramHostRequest) -> ProgramResult<Vec<u8>> {
    let codes = program_codecs();
    let ProgramCode::HostEffect(parameter, _, _) = codes[request.codec] else {
        return Err("program ABI: invalid host request codec");
    };
    let mut output = vec![5];
    program_bytes(&mut output, &request.token.to_le_bytes())?;
    program_bytes(&mut output, &(request.codec as u32).to_le_bytes())?;
    program_bytes(&mut output, &[u8::from(request.masked)])?;
    program_encode(request.argument.clone(), parameter, &codes, &mut output)?;
    Ok(output)
}
fn program_host_outcome(kind: u32, bytes: &[u8], codec: usize) -> ProgramResult<NativeOutcome> {
    let codes = program_codecs();
    let ProgramCode::HostEffect(_, failure, success) = codes[codec] else {
        return Err("program ABI: invalid host response codec");
    };
    let mut input = bytes;
    let outcome = match kind {
        1 => Ok(program_decode(
            &mut input,
            success.ok_or("host callback returned Never success")?,
            &codes,
        )?),
        2 => Err(NativeExit::Failure(program_decode(
            &mut input,
            failure.ok_or("host callback returned Never failure")?,
            &codes,
        )?)),
        3 => {
            let count = program_u32(&mut input)?;
            if count == 0 || count as usize > input.len() / 4 {
                return Err("invalid host Defect count");
            }
            let mut messages = Vec::new();
            for _ in 0..count {
                let size = program_u32(&mut input)? as usize;
                messages.push(
                    std::str::from_utf8(program_take(&mut input, size)?)
                        .map_err(|_| "invalid host Defect UTF-8")?
                        .to_owned(),
                );
            }
            Err(NativeExit::Defect(messages))
        }
        4 => Err(NativeExit::Cancelled),
        _ => return Err("invalid host response tag"),
    };
    if !input.is_empty() {
        return Err("trailing host response bytes");
    }
    Ok(outcome)
}
#[no_mangle]
pub extern "C" fn ssrg_host_reply(kind: u32) -> u32 {
    let bytes = PROGRAM.with(|state| state.borrow().input.clone());
    PROGRAM_RUN.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(run) = slot.as_mut() else { return 1 };
        let Some(request) = run.waiting.take() else {
            return 1;
        };
        let mut outcome = if kind == 4 && (request.masked || !run.machine.context.cancelled) {
            Err("host callback cancelled outside an unmasked cancellation request")
        } else {
            program_host_outcome(kind, &bytes, request.codec)
        }
        .unwrap_or_else(|message| Err(NativeExit::Defect(vec![message.into()])));
        // Acquisition succeeds into Register BEFORE observing cancellation.
        // Unmasked host success is a cancellation boundary of its own.
        if !request.masked && run.machine.context.cancelled && outcome.is_ok() {
            outcome = Err(NativeExit::Cancelled);
        }
        run.machine.resume(outcome);
        0
    })
}
#[no_mangle]
pub extern "C" fn ssrg_hosts() -> u32 {
    let bytes = PROGRAM_HOSTS.with(|hosts| {
        let mut hosts = hosts.borrow_mut();
        hosts.retain(|_, lease| lease.strong_count() > 0);
        let mut bytes = Vec::new();
        program_bytes(&mut bytes, &(hosts.len() as u32).to_le_bytes())?;
        for token in hosts.keys() {
            program_bytes(&mut bytes, &token.to_le_bytes())?;
        }
        Ok(bytes)
    });
    program_result(bytes)
}
